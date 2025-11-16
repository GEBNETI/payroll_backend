use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::job::Job,
    error::{AppError, AppResult},
    server::AppState,
    services::job::{CreateJobParams, UpdateJobParams},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateJobRequest {
    pub job_title: String,
    pub salary: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateJobRequest {
    pub job_title: Option<String>,
    pub salary: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobResponse {
    pub id: Uuid,
    pub job_title: String,
    pub salary: f64,
    pub payroll_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct JobCollectionPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub struct JobPathParams {
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub job_id: Uuid,
}

impl From<Job> for JobResponse {
    fn from(value: Job) -> Self {
        Self {
            id: value.id,
            job_title: value.job_title,
            salary: value.salary,
            payroll_id: value.payroll_id,
        }
    }
}

impl CreateJobRequest {
    fn into_params(self) -> CreateJobParams {
        CreateJobParams {
            job_title: self.job_title,
            salary: self.salary,
        }
    }
}

impl UpdateJobRequest {
    fn into_params(self) -> UpdateJobParams {
        UpdateJobParams {
            job_title: self.job_title,
            salary: self.salary,
        }
    }
}

#[utoipa::path(
    post,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/jobs",
    params(JobCollectionPathParams),
    request_body = CreateJobRequest,
    responses(
        (status = 201, description = "Job created", body = JobResponse)
    ),
    tag = "Jobs",
    operation_id = "create_job"
)]
pub async fn create(
    State(state): State<AppState>,
    Path(params): Path<JobCollectionPathParams>,
    Json(payload): Json<CreateJobRequest>,
) -> AppResult<(StatusCode, Json<JobResponse>)> {
    let job = state
        .job_service()
        .create(
            params.organization_id,
            params.payroll_id,
            payload.into_params(),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(job.into())))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/jobs",
    params(JobCollectionPathParams),
    responses(
        (status = 200, description = "List jobs", body = [JobResponse])
    ),
    tag = "Jobs",
    operation_id = "list_jobs"
)]
pub async fn list(
    State(state): State<AppState>,
    Path(params): Path<JobCollectionPathParams>,
) -> AppResult<Json<Vec<JobResponse>>> {
    let jobs = state
        .job_service()
        .list(params.organization_id, params.payroll_id)
        .await?;
    let response = jobs.into_iter().map(JobResponse::from).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/jobs/{job_id}",
    params(JobPathParams),
    responses(
        (status = 200, description = "Get job", body = JobResponse),
        (status = 404, description = "Job not found")
    ),
    tag = "Jobs",
    operation_id = "get_job"
)]
pub async fn get(
    State(state): State<AppState>,
    Path(params): Path<JobPathParams>,
) -> AppResult<Json<JobResponse>> {
    let job = state
        .job_service()
        .get(params.organization_id, params.payroll_id, params.job_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "job `{}` not found for payroll `{}`",
                params.job_id, params.payroll_id
            ))
        })?;

    Ok(Json(job.into()))
}

#[utoipa::path(
    put,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/jobs/{job_id}",
    params(JobPathParams),
    request_body = UpdateJobRequest,
    responses(
        (status = 200, description = "Job updated", body = JobResponse),
        (status = 404, description = "Job not found")
    ),
    tag = "Jobs",
    operation_id = "update_job"
)]
pub async fn update(
    State(state): State<AppState>,
    Path(params): Path<JobPathParams>,
    Json(payload): Json<UpdateJobRequest>,
) -> AppResult<Json<JobResponse>> {
    let job = state
        .job_service()
        .update(
            params.organization_id,
            params.payroll_id,
            params.job_id,
            payload.into_params(),
        )
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!(
                "job `{}` not found for payroll `{}`",
                params.job_id, params.payroll_id
            ))
        })?;

    Ok(Json(job.into()))
}

#[utoipa::path(
    delete,
    path = "/organizations/{organization_id}/payrolls/{payroll_id}/jobs/{job_id}",
    params(JobPathParams),
    responses(
        (status = 204, description = "Job deleted"),
        (status = 404, description = "Job not found")
    ),
    tag = "Jobs",
    operation_id = "delete_job"
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(params): Path<JobPathParams>,
) -> AppResult<StatusCode> {
    let removed = state
        .job_service()
        .delete(params.organization_id, params.payroll_id, params.job_id)
        .await?;

    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(format!(
            "job `{}` not found for payroll `{}`",
            params.job_id, params.payroll_id
        )))
    }
}
