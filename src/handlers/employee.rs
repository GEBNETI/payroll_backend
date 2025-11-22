use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::employee::Employee,
    error::{AppError, AppResult},
    server::AppState,
    services::employee::{CreateEmployeeParams, UpdateEmployeeParams},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEmployeeRequest {
    pub id_number: String,
    pub last_name: String,
    pub first_name: String,
    pub address: String,
    pub phone: String,
    pub place_of_birth: String,
    #[schema(value_type = String, format = Date)]
    pub date_of_birth: NaiveDate,
    pub nationality: String,
    pub marital_status: String,
    pub gender: String,
    #[schema(value_type = String, format = Date)]
    pub hire_date: NaiveDate,
    #[schema(value_type = Option<String>, format = Date)]
    pub leaving_date: Option<NaiveDate>,
    pub clasification: String,
    pub job_id: Uuid,
    pub bank_id: Uuid,
    pub bank_account: String,
    pub status: String,
    pub hours: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEmployeeRequest {
    pub id_number: Option<String>,
    pub last_name: Option<String>,
    pub first_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub place_of_birth: Option<String>,
    #[schema(value_type = Option<String>, format = Date)]
    pub date_of_birth: Option<NaiveDate>,
    pub nationality: Option<String>,
    pub marital_status: Option<String>,
    pub gender: Option<String>,
    #[schema(value_type = Option<String>, format = Date)]
    pub hire_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "deserialize_option_option")]
    #[schema(value_type = Option<String>, format = Date)]
    pub leaving_date: Option<Option<NaiveDate>>,
    pub clasification: Option<String>,
    pub job_id: Option<Uuid>,
    pub bank_id: Option<Uuid>,
    pub bank_account: Option<String>,
    pub status: Option<String>,
    pub hours: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmployeeResponse {
    pub id: Uuid,
    pub id_number: String,
    pub last_name: String,
    pub first_name: String,
    pub address: String,
    pub phone: String,
    pub place_of_birth: String,
    #[schema(value_type = String, format = Date)]
    pub date_of_birth: NaiveDate,
    pub nationality: String,
    pub marital_status: String,
    pub gender: String,
    #[schema(value_type = String, format = Date)]
    pub hire_date: NaiveDate,
    #[schema(value_type = Option<String>, format = Date)]
    pub leaving_date: Option<NaiveDate>,
    pub clasification: String,
    pub job_id: Uuid,
    pub bank_id: Uuid,
    pub bank_account: String,
    pub status: String,
    pub hours: i32,
    pub division_id: Uuid,
    pub payroll_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct EmployeeCollectionPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub division_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct EmployeePathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub division_id: Uuid,
    pub employee_id: Uuid,
}

impl From<Employee> for EmployeeResponse {
    fn from(value: Employee) -> Self {
        Self {
            id: value.id,
            id_number: value.id_number,
            last_name: value.last_name,
            first_name: value.first_name,
            address: value.address,
            phone: value.phone,
            place_of_birth: value.place_of_birth,
            date_of_birth: value.date_of_birth,
            nationality: value.nationality,
            marital_status: value.marital_status,
            gender: value.gender,
            hire_date: value.hire_date,
            leaving_date: value.leaving_date,
            clasification: value.clasification,
            job_id: value.job_id,
            bank_id: value.bank_id,
            bank_account: value.bank_account,
            status: value.status,
            hours: value.hours,
            division_id: value.division_id,
            payroll_id: value.payroll_id,
        }
    }
}

impl CreateEmployeeRequest {
    fn into_params(self) -> CreateEmployeeParams {
        CreateEmployeeParams {
            id_number: self.id_number,
            last_name: self.last_name,
            first_name: self.first_name,
            address: self.address,
            phone: self.phone,
            place_of_birth: self.place_of_birth,
            date_of_birth: self.date_of_birth,
            nationality: self.nationality,
            marital_status: self.marital_status,
            gender: self.gender,
            hire_date: self.hire_date,
            leaving_date: self.leaving_date,
            clasification: self.clasification,
            job_id: self.job_id,
            bank_id: self.bank_id,
            bank_account: self.bank_account,
            status: self.status,
            hours: self.hours,
        }
    }
}

impl UpdateEmployeeRequest {
    fn into_params(self) -> UpdateEmployeeParams {
        UpdateEmployeeParams {
            id_number: self.id_number,
            last_name: self.last_name,
            first_name: self.first_name,
            address: self.address,
            phone: self.phone,
            place_of_birth: self.place_of_birth,
            date_of_birth: self.date_of_birth,
            nationality: self.nationality,
            marital_status: self.marital_status,
            gender: self.gender,
            hire_date: self.hire_date,
            leaving_date: self.leaving_date,
            clasification: self.clasification,
            job_id: self.job_id,
            bank_id: self.bank_id,
            bank_account: self.bank_account,
            status: self.status,
            hours: self.hours,
        }
    }
}

fn deserialize_option_option<'de, D>(deserializer: D) -> Result<Option<Option<NaiveDate>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees",
    params(EmployeeCollectionPathParams),
    request_body = CreateEmployeeRequest,
    responses(
        (status = 201, description = "Employee created", body = EmployeeResponse)
    ),
    tag = "Employees",
    operation_id = "create_employee"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<EmployeeCollectionPathParams>,
    Json(payload): Json<CreateEmployeeRequest>,
) -> AppResult<(StatusCode, Json<EmployeeResponse>)> {
    let employee = state
        .employee_service()
        .create(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            payload.into_params(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(employee.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees",
    params(EmployeeCollectionPathParams),
    responses(
        (status = 200, description = "List employees", body = [EmployeeResponse])
    ),
    tag = "Employees",
    operation_id = "list_employees"
)]
pub async fn list(
    State(state): State<AppState>,
    Path(params): Path<EmployeeCollectionPathParams>,
) -> AppResult<Json<Vec<EmployeeResponse>>> {
    let employees = state
        .employee_service()
        .list(
            params.organization_id,
            params.payroll_id,
            params.division_id,
        )
        .await?;
    let response = employees.into_iter().map(EmployeeResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}",
    params(EmployeePathParams),
    responses(
        (status = 200, description = "Get employee", body = EmployeeResponse),
        (status = 404, description = "Employee not found")
    ),
    tag = "Employees",
    operation_id = "get_employee"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<EmployeePathParams>,
) -> AppResult<Json<EmployeeResponse>> {
    let employee = state
        .employee_service()
        .get(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            params.employee_id,
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "employee `{}` not found for division `{}` in payroll `{}`",
                params.employee_id, params.division_id, params.payroll_id
            ))
        })?;

    Ok(Json(employee.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}",
    params(EmployeePathParams),
    request_body = UpdateEmployeeRequest,
    responses(
        (status = 200, description = "Employee updated", body = EmployeeResponse),
        (status = 404, description = "Employee not found")
    ),
    tag = "Employees",
    operation_id = "update_employee"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<EmployeePathParams>,
    Json(payload): Json<UpdateEmployeeRequest>,
) -> AppResult<Json<EmployeeResponse>> {
    let employee = state
        .employee_service()
        .update(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            params.employee_id,
            payload.into_params(),
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "employee `{}` not found for division `{}` in payroll `{}`",
                params.employee_id, params.division_id, params.payroll_id
            ))
        })?;

    Ok(Json(employee.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/divisions/{division_id}/employees/{employee_id}",
    params(EmployeePathParams),
    responses(
        (status = 204, description = "Employee deleted"),
        (status = 404, description = "Employee not found")
    ),
    tag = "Employees",
    operation_id = "delete_employee"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<EmployeePathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .employee_service()
        .delete(
            params.organization_id,
            params.payroll_id,
            params.division_id,
            params.employee_id,
        )
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "employee `{}` not found for division `{}` in payroll `{}`",
            params.employee_id, params.division_id, params.payroll_id
        )))
    }
}
