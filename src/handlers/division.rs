use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::division::Division,
    error::{AppError, AppResult},
    server::AppState,
    services::division::{CreateDivisionParams, UpdateDivisionParams},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDivisionRequest {
    pub name: String,
    pub description: String,
    pub budget_code: String,
    pub payroll_id: Uuid,
    pub parent_division_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDivisionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub budget_code: Option<String>,
    pub payroll_id: Option<Uuid>,
    #[serde(default, deserialize_with = "deserialize_option_option")]
    #[schema(value_type = Option<Uuid>)]
    pub parent_division_id: Option<Option<Uuid>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DivisionResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub budget_code: String,
    pub payroll_id: Uuid,
    pub parent_division_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct DivisionPathParams {
    pub id: Uuid,
}

impl From<Division> for DivisionResponse {
    fn from(value: Division) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            budget_code: value.budget_code,
            payroll_id: value.payroll_id,
            parent_division_id: value.parent_division_id,
        }
    }
}

impl CreateDivisionRequest {
    fn into_params(self) -> CreateDivisionParams {
        CreateDivisionParams {
            name: self.name,
            description: self.description,
            budget_code: self.budget_code,
            payroll_id: self.payroll_id,
            parent_division_id: self.parent_division_id,
        }
    }
}

impl UpdateDivisionRequest {
    fn into_params(self) -> UpdateDivisionParams {
        UpdateDivisionParams {
            name: self.name,
            description: self.description,
            budget_code: self.budget_code,
            payroll_id: self.payroll_id,
            parent_division_id: self.parent_division_id,
        }
    }
}

fn deserialize_option_option<'de, D>(deserializer: D) -> Result<Option<Option<Uuid>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[utoipa::path(
    post,
    path = "/divisions",
    request_body = CreateDivisionRequest,
    responses(
        (status = 201, description = "Division created", body = DivisionResponse)
    ),
    tag = "Divisions"
)]
pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateDivisionRequest>,
) -> AppResult<(StatusCode, Json<DivisionResponse>)> {
    let division = state
        .division_service()
        .create(payload.into_params())
        .await?;

    Ok((StatusCode::CREATED, Json(division.into())))
}

#[utoipa::path(
    get,
    path = "/divisions",
    responses(
        (status = 200, description = "List divisions", body = [DivisionResponse])
    ),
    tag = "Divisions"
)]
pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<DivisionResponse>>> {
    let divisions = state.division_service().list().await?;
    let response = divisions.into_iter().map(DivisionResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/divisions/{id}",
    params(DivisionPathParams),
    responses(
        (status = 200, description = "Get division", body = DivisionResponse),
        (status = 404, description = "Division not found")
    ),
    tag = "Divisions"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<DivisionPathParams>,
) -> AppResult<Json<DivisionResponse>> {
    let id = params.id;
    let division = state
        .division_service()
        .get(id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("division `{id}` not found")))?;

    Ok(Json(division.into()))
}

#[utoipa::path(
    put,
    path = "/divisions/{id}",
    params(DivisionPathParams),
    request_body = UpdateDivisionRequest,
    responses(
        (status = 200, description = "Division updated", body = DivisionResponse),
        (status = 404, description = "Division not found")
    ),
    tag = "Divisions"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<DivisionPathParams>,
    Json(payload): Json<UpdateDivisionRequest>,
) -> AppResult<Json<DivisionResponse>> {
    let id = params.id;
    let division = state
        .division_service()
        .update(id, payload.into_params())
        .await?
        .ok_or_else(|| AppError::not_found(format!("division `{id}` not found")))?;

    Ok(Json(division.into()))
}

#[utoipa::path(
    delete,
    path = "/divisions/{id}",
    params(DivisionPathParams),
    responses(
        (status = 204, description = "Division deleted"),
        (status = 404, description = "Division not found")
    ),
    tag = "Divisions"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<DivisionPathParams>,
) -> AppResult<StatusCode> {
    let id = params.id;
    let removed = state.division_service().delete(id).await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("division `{id}` not found")))
    }
}
