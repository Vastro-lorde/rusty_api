//! Swagger documentation setup using utoipa
use crate::api::routes::{auth, employees};
use crate::domain::models::{AuthResponse, CreateEmployeePayload, Employee, OtpRequestPayload, OtpVerifyPayload};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::request_otp,
        auth::verify_otp,
        employees::list_employees,
        employees::create_employee,
        crate::api::routes::bulk::upload_csv
    ),
    components(
        schemas(
            Employee,
            CreateEmployeePayload,
            OtpRequestPayload,
            OtpVerifyPayload,
            AuthResponse,
            crate::api::routes::bulk::BulkUploadResponse,
            crate::api::routes::bulk::BulkUploadForm
        )
    ),
    tags(
        (name = "Authentication", description = "OTP Auth endpoints using email"),
        (name = "Employees", description = "Employee management endpoints")
    )
)]
pub struct ApiDoc;
