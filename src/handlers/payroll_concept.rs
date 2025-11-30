use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::payroll_concept::{PayrollConcept, PayrollConceptScope, PayrollConceptType},
    error::{AppError, AppResult},
    server::AppState,
    services::payroll_concept::{CreatePayrollConceptParams, UpdatePayrollConceptParams},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePayrollConceptRequest {
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub concept_type: PayrollConceptType,
    pub scope: PayrollConceptScope,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePayrollConceptRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub concept_type: Option<PayrollConceptType>,
    pub scope: Option<PayrollConceptScope>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PayrollConceptResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub concept_type: PayrollConceptType,
    pub scope: PayrollConceptScope,
    pub payroll_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollConceptCollectionPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollConceptPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub concept_id: Uuid,
}

impl From<PayrollConcept> for PayrollConceptResponse {
    fn from(value: PayrollConcept) -> Self {
        Self {
            id: value.id,
            code: value.code,
            name: value.name,
            concept_type: value.concept_type,
            scope: value.scope,
            payroll_id: value.payroll_id,
        }
    }
}

impl CreatePayrollConceptRequest {
    fn into_params(self) -> CreatePayrollConceptParams {
        CreatePayrollConceptParams {
            code: self.code,
            name: self.name,
            concept_type: self.concept_type,
            scope: self.scope,
        }
    }
}

impl UpdatePayrollConceptRequest {
    fn into_params(self) -> UpdatePayrollConceptParams {
        UpdatePayrollConceptParams {
            code: self.code,
            name: self.name,
            concept_type: self.concept_type,
            scope: self.scope,
        }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts",
    params(PayrollConceptCollectionPathParams),
    request_body = CreatePayrollConceptRequest,
    responses(
        (status = 201, description = "Payroll concept created", body = PayrollConceptResponse)
    ),
    tag = "Payroll Concepts",
    operation_id = "create_payroll_concept"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptCollectionPathParams>,
    Json(payload): Json<CreatePayrollConceptRequest>,
) -> AppResult<(StatusCode, Json<PayrollConceptResponse>)> {
    let concept = state
        .payroll_concept_service()
        .create(
            params.organization_id,
            params.payroll_id,
            payload.into_params(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(concept.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts",
    params(PayrollConceptCollectionPathParams),
    responses(
        (status = 200, description = "List payroll concepts", body = [PayrollConceptResponse])
    ),
    tag = "Payroll Concepts",
    operation_id = "list_payroll_concepts"
)]
pub async fn list(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptCollectionPathParams>,
) -> AppResult<Json<Vec<PayrollConceptResponse>>> {
    let concepts = state
        .payroll_concept_service()
        .list(params.organization_id, params.payroll_id)
        .await?;
    let response = concepts
        .into_iter()
        .map(PayrollConceptResponse::from)
        .collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}",
    params(PayrollConceptPathParams),
    responses(
        (status = 200, description = "Get payroll concept", body = PayrollConceptResponse),
        (status = 404, description = "Payroll concept not found")
    ),
    tag = "Payroll Concepts",
    operation_id = "get_payroll_concept"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptPathParams>,
) -> AppResult<Json<PayrollConceptResponse>> {
    let concept = state
        .payroll_concept_service()
        .get(params.organization_id, params.payroll_id, params.concept_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "payroll concept `{}` not found for payroll `{}`",
                params.concept_id, params.payroll_id
            ))
        })?;

    Ok(Json(concept.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}",
    params(PayrollConceptPathParams),
    request_body = UpdatePayrollConceptRequest,
    responses(
        (status = 200, description = "Payroll concept updated", body = PayrollConceptResponse),
        (status = 404, description = "Payroll concept not found")
    ),
    tag = "Payroll Concepts",
    operation_id = "update_payroll_concept"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptPathParams>,
    Json(payload): Json<UpdatePayrollConceptRequest>,
) -> AppResult<Json<PayrollConceptResponse>> {
    let concept = state
        .payroll_concept_service()
        .update(
            params.organization_id,
            params.payroll_id,
            params.concept_id,
            payload.into_params(),
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "payroll concept `{}` not found for payroll `{}`",
                params.concept_id, params.payroll_id
            ))
        })?;

    Ok(Json(concept.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}",
    params(PayrollConceptPathParams),
    responses(
        (status = 204, description = "Payroll concept deleted"),
        (status = 404, description = "Payroll concept not found")
    ),
    tag = "Payroll Concepts",
    operation_id = "delete_payroll_concept"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptPathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .payroll_concept_service()
        .delete(params.organization_id, params.payroll_id, params.concept_id)
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "payroll concept `{}` not found for payroll `{}`",
            params.concept_id, params.payroll_id
        )))
    }
}
