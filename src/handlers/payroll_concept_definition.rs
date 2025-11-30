use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::payroll_concept_definition::PayrollConceptDefinition,
    error::{AppError, AppResult},
    server::AppState,
    services::payroll_concept_definition::{
        CreatePayrollConceptDefinitionParams, UpdatePayrollConceptDefinitionParams,
    },
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePayrollConceptDefinitionRequest {
    pub formula: String,
    pub condition: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePayrollConceptDefinitionRequest {
    pub formula: Option<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PayrollConceptDefinitionResponse {
    pub id: Uuid,
    pub payroll_concept_id: Uuid,
    pub formula: String,
    pub condition: String,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct PayrollConceptDefinitionPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub concept_id: Uuid,
}

impl From<PayrollConceptDefinition> for PayrollConceptDefinitionResponse {
    fn from(value: PayrollConceptDefinition) -> Self {
        Self {
            id: value.id,
            payroll_concept_id: value.payroll_concept_id,
            formula: value.formula,
            condition: value.condition,
        }
    }
}

impl CreatePayrollConceptDefinitionRequest {
    fn into_params(self) -> CreatePayrollConceptDefinitionParams {
        CreatePayrollConceptDefinitionParams {
            formula: self.formula,
            condition: self.condition,
        }
    }
}

impl UpdatePayrollConceptDefinitionRequest {
    fn into_params(self) -> UpdatePayrollConceptDefinitionParams {
        UpdatePayrollConceptDefinitionParams {
            formula: self.formula,
            condition: self.condition,
        }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition",
    params(PayrollConceptDefinitionPathParams),
    request_body = CreatePayrollConceptDefinitionRequest,
    responses(
        (status = 201, description = "Payroll concept definition created", body = PayrollConceptDefinitionResponse)
    ),
    tag = "Payroll Concepts",
    operation_id = "create_payroll_concept_definition"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptDefinitionPathParams>,
    Json(payload): Json<CreatePayrollConceptDefinitionRequest>,
) -> AppResult<(StatusCode, Json<PayrollConceptDefinitionResponse>)> {
    let definition = state
        .payroll_concept_definition_service()
        .create(
            params.organization_id,
            params.payroll_id,
            params.concept_id,
            payload.into_params(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(definition.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition",
    params(PayrollConceptDefinitionPathParams),
    responses(
        (status = 200, description = "Get payroll concept definition", body = PayrollConceptDefinitionResponse),
        (status = 404, description = "Definition not found")
    ),
    tag = "Payroll Concepts",
    operation_id = "get_payroll_concept_definition"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptDefinitionPathParams>,
) -> AppResult<Json<PayrollConceptDefinitionResponse>> {
    let definition = state
        .payroll_concept_definition_service()
        .get(params.organization_id, params.payroll_id, params.concept_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "definition not found for payroll concept `{}`",
                params.concept_id
            ))
        })?;

    Ok(Json(definition.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition",
    params(PayrollConceptDefinitionPathParams),
    request_body = UpdatePayrollConceptDefinitionRequest,
    responses(
        (status = 200, description = "Payroll concept definition updated", body = PayrollConceptDefinitionResponse),
        (status = 404, description = "Definition not found")
    ),
    tag = "Payroll Concepts",
    operation_id = "update_payroll_concept_definition"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptDefinitionPathParams>,
    Json(payload): Json<UpdatePayrollConceptDefinitionRequest>,
) -> AppResult<Json<PayrollConceptDefinitionResponse>> {
    let definition = state
        .payroll_concept_definition_service()
        .update(
            params.organization_id,
            params.payroll_id,
            params.concept_id,
            payload.into_params(),
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "definition not found for payroll concept `{}`",
                params.concept_id
            ))
        })?;

    Ok(Json(definition.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/concepts/{concept_id}/definition",
    params(PayrollConceptDefinitionPathParams),
    responses(
        (status = 204, description = "Payroll concept definition deleted"),
        (status = 404, description = "Definition not found")
    ),
    tag = "Payroll Concepts",
    operation_id = "delete_payroll_concept_definition"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<PayrollConceptDefinitionPathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .payroll_concept_definition_service()
        .delete(params.organization_id, params.payroll_id, params.concept_id)
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "definition not found for payroll concept `{}`",
            params.concept_id
        )))
    }
}
