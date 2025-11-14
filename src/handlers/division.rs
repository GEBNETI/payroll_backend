use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::{
    domain::division::Division,
    error::{AppError, AppResult},
    server::AppState,
    services::division::{CreateDivisionParams, UpdateDivisionParams},
};

#[derive(Debug, Deserialize)]
pub struct CreateDivisionRequest {
    pub name: String,
    pub description: String,
    pub budget_code: String,
    pub payroll_id: Uuid,
    pub parent_division_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDivisionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub budget_code: Option<String>,
    pub payroll_id: Option<Uuid>,
    #[serde(default, deserialize_with = "deserialize_option_option")]
    pub parent_division_id: Option<Option<Uuid>>,
}

#[derive(Debug, Serialize)]
pub struct DivisionResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub budget_code: String,
    pub payroll_id: Uuid,
    pub parent_division_id: Option<Uuid>,
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

pub async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<DivisionResponse>>> {
    let divisions = state.division_service().list().await?;
    let response = divisions.into_iter().map(DivisionResponse::from).collect();
    Ok(Json(response))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<DivisionResponse>> {
    let division = state
        .division_service()
        .get(id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("division `{id}` not found")))?;

    Ok(Json(division.into()))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDivisionRequest>,
) -> AppResult<Json<DivisionResponse>> {
    let division = state
        .division_service()
        .update(id, payload.into_params())
        .await?
        .ok_or_else(|| AppError::not_found(format!("division `{id}` not found")))?;

    Ok(Json(division.into()))
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    let removed = state.division_service().delete(id).await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!("division `{id}` not found")))
    }
}
