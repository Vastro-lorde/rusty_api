//! Employee CRUD Endpoints
use crate::{
    app_state::AppState,
    domain::{
        errors::AppError,
        models::{CreateEmployeePayload, Employee},
    },
};
use axum::{extract::State, Json};

/// Retrieve all active employees from the database.
#[utoipa::path(
    get,
    path = "/employees",
    tag = "Employees",
    responses(
        (status = 200, description = "List of all active employees", body = [Employee])
    )
)]
pub async fn list_employees(State(state): State<AppState>) -> Result<Json<Vec<Employee>>, AppError> {
    // Execute a real SQL query using sqlx!
    let employees = sqlx::query_as::<_, Employee>("SELECT * FROM employees WHERE active = 1")
        .fetch_all(&state.db_pool)
        .await?;

    Ok(Json(employees))
}

/// Creates a new employee and inserts them into the SQLite database.
#[utoipa::path(
    post,
    path = "/employees",
    tag = "Employees",
    request_body(content = CreateEmployeePayload, description = "The employee details"),
    responses(
        (status = 200, description = "Employee successfully created", body = Employee)
    )
)]
pub async fn create_employee(
    State(state): State<AppState>,
    Json(payload): Json<CreateEmployeePayload>,
) -> Result<Json<Employee>, AppError> {
    // Insert into DB and return the inserted row
    let employee = sqlx::query_as::<_, Employee>(
        r#"
        INSERT INTO employees (name, department, active) 
        VALUES (?1, ?2, 1) 
        RETURNING id, name, department, active
        "#
    )
    .bind(&payload.name)
    .bind(&payload.department)
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(employee))
}
