//! CSV Bulk Upload Endpoints using Tokio Concurrency
use crate::{
    app_state::AppState,
    domain::errors::AppError,
};
use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

#[derive(Deserialize, Debug)]
pub struct CsvRecord {
    pub name: String,
    pub department: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct BulkUploadResponse {
    pub total_inserted: usize,
    pub chunks_processed: usize,
}

/// This struct is purely for generating the Swagger UI file upload button
#[derive(utoipa::ToSchema)]
pub struct BulkUploadForm {
    /// The CSV file to upload
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

#[utoipa::path(
    post,
    path = "/employees/bulk",
    tag = "Employees",
    request_body(content = inline(BulkUploadForm), content_type = "multipart/form-data", description = "CSV file with columns: name, department"),
    responses(
        (status = 200, description = "Bulk upload successful", body = BulkUploadResponse)
    )
)]
pub async fn upload_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<BulkUploadResponse>, AppError> {
    let mut raw_csv_data = Vec::new();

    // 1. Extract the raw bytes from the multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Internal(e.to_string()))? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| AppError::Internal(e.to_string()))?;
            raw_csv_data.extend_from_slice(&data);
            break; // We only care about the first file
        }
    }

    if raw_csv_data.is_empty() {
        return Err(AppError::Internal("No file uploaded".to_string()));
    }

    // 2. Parse the CSV using the `csv` crate
    tracing::info!("Parsing CSV payload...");
    let mut reader = csv::Reader::from_reader(raw_csv_data.as_slice());
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: CsvRecord = result.map_err(|e| AppError::Internal(format!("Invalid CSV format: {}", e)))?;
        records.push(record);
    }

    let total_records = records.len();
    if total_records == 0 {
        return Ok(Json(BulkUploadResponse { total_inserted: 0, chunks_processed: 0 }));
    }

    // 3. Chunk the records into groups of 500 to process them concurrently!
    // We clone the SqlitePool (which is essentially an Arc under the hood, so it's very cheap)
    let pool = state.db_pool.clone();
    let chunk_size = 500;
    let mut tasks: Vec<JoinHandle<Result<usize, AppError>>> = Vec::new();

    tracing::info!("Spawning Tokio tasks to process {} records concurrently", total_records);

    // We chunk the array
    for chunk in records.chunks(chunk_size) {
        // We must clone the chunk because it's moving into a new thread
        let chunk_owned = chunk.iter().map(|c| CsvRecord {
            name: c.name.clone(),
            department: c.department.clone(),
        }).collect::<Vec<_>>();
        
        let pool_clone = pool.clone();

        // 4. Spawn a Tokio Async Task! 
        // This runs the insertion concurrently in the background across your CPU cores.
        let task = tokio::spawn(async move {
            // Let's use a Database Transaction for safety within this chunk!
            // If one insert fails, the whole chunk rolls back.
            let mut tx = pool_clone.begin().await?;

            let mut inserted = 0;
            for record in chunk_owned {
                sqlx::query(
                    "INSERT INTO employees (name, department, active) VALUES (?1, ?2, 1)"
                )
                .bind(&record.name)
                .bind(&record.department)
                .execute(&mut *tx)
                .await?;
                inserted += 1;
            }

            // Commit the transaction
            tx.commit().await?;
            Ok(inserted)
        });

        tasks.push(task);
    }

    // 5. Wait for all background tasks to finish
    let mut total_inserted = 0;
    let chunks_processed = tasks.len();

    for task in tasks {
        match task.await {
            Ok(Ok(inserted)) => total_inserted += inserted,
            Ok(Err(e)) => tracing::error!("A chunk failed to insert: {:?}", e),
            Err(e) => tracing::error!("Tokio thread panicked: {:?}", e),
        }
    }

    tracing::info!("Finished processing {} chunks", chunks_processed);

    Ok(Json(BulkUploadResponse {
        total_inserted,
        chunks_processed,
    }))
}
