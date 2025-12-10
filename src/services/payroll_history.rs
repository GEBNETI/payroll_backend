use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::{
    domain::payroll_history::{NewPayrollHistoryData, PayrollHistory, PayrollHistoryStatus},
    error::{AppError, AppResult},
    services::{organization::OrganizationService, payroll::PayrollService},
};

#[derive(Debug, Clone)]
pub struct CreatePayrollHistoryParams {
    pub title: String,
    pub period: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: PayrollHistoryStatus,
    pub total_employees: i32,
    pub total_earnings: f64,
    pub total_deductions: f64,
}

#[derive(Debug, Clone, Default)]
pub struct UpdatePayrollHistoryParams {
    pub title: Option<String>,
    pub period: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub status: Option<PayrollHistoryStatus>,
    pub total_employees: Option<i32>,
    pub total_earnings: Option<f64>,
    pub total_deductions: Option<f64>,
}

#[async_trait]
pub trait PayrollHistoryRepository: Send + Sync {
    async fn insert(&self, data: NewPayrollHistoryData) -> AppResult<PayrollHistory>;

    async fn fetch(&self, id: Uuid) -> AppResult<Option<PayrollHistory>>;

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<PayrollHistory>>;

    async fn update(
        &self,
        id: Uuid,
        params: UpdatePayrollHistoryParams,
    ) -> AppResult<Option<PayrollHistory>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct PayrollHistoryService {
    repository: Arc<dyn PayrollHistoryRepository>,
    payroll_service: Arc<PayrollService>,
    organization_service: Arc<OrganizationService>,
}

impl PayrollHistoryService {
    pub fn new(
        repository: Arc<dyn PayrollHistoryRepository>,
        payroll_service: Arc<PayrollService>,
        organization_service: Arc<OrganizationService>,
    ) -> Self {
        Self {
            repository,
            payroll_service,
            organization_service,
        }
    }

    pub async fn create(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        params: CreatePayrollHistoryParams,
    ) -> AppResult<PayrollHistory> {
        let payroll = self
            .payroll_service
            .get(organization_id, payroll_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "payroll `{payroll_id}` not found for organization `{organization_id}`"
                ))
            })?;

        let organization = self
            .payroll_service
            .ensure_belongs_to_organization(organization_id, payroll_id)
            .await
            .map(|_| organization_id)?;

        let title = Self::normalize_field(&params.title, "title")?;
        let period = Self::normalize_field(&params.period, "period")?;
        Self::validate_date_range(params.start_date, params.end_date)?;
        Self::validate_total_employees(params.total_employees)?;
        Self::validate_amount(params.total_earnings, "total_earnings")?;
        Self::validate_amount(params.total_deductions, "total_deductions")?;

        let id = Uuid::new_v4();
        let created_at = Utc::now();

        // Get organization name - we need to fetch it
        let organization_name = self.get_organization_name(organization).await?;

        let data = NewPayrollHistoryData {
            id,
            title,
            period,
            start_date: params.start_date,
            end_date: params.end_date,
            created_at,
            organization_id,
            organization_name,
            payroll_id,
            payroll_name: payroll.name,
            status: params.status,
            total_employees: params.total_employees,
            total_earnings: params.total_earnings,
            total_deductions: params.total_deductions,
        };

        self.repository.insert(data).await
    }

    pub async fn get(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<Option<PayrollHistory>> {
        self.payroll_service
            .ensure_belongs_to_organization(organization_id, payroll_id)
            .await?;

        let history = self.repository.fetch(history_id).await?;
        Ok(history.filter(|h| h.payroll_id == payroll_id && h.organization_id == organization_id))
    }

    pub async fn list(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
    ) -> AppResult<Vec<PayrollHistory>> {
        self.payroll_service
            .ensure_belongs_to_organization(organization_id, payroll_id)
            .await?;

        let mut histories = self.repository.fetch_by_payroll(payroll_id).await?;
        histories.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(histories)
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
        params: UpdatePayrollHistoryParams,
    ) -> AppResult<Option<PayrollHistory>> {
        if params.title.is_none()
            && params.period.is_none()
            && params.start_date.is_none()
            && params.end_date.is_none()
            && params.status.is_none()
            && params.total_employees.is_none()
            && params.total_earnings.is_none()
            && params.total_deductions.is_none()
        {
            return Err(AppError::validation("no fields supplied for update"));
        }

        let existing = match self.get(organization_id, payroll_id, history_id).await? {
            Some(h) => h,
            None => return Ok(None),
        };

        let start_date = params.start_date.unwrap_or(existing.start_date);
        let end_date = params.end_date.unwrap_or(existing.end_date);
        Self::validate_date_range(start_date, end_date)?;

        let updates = UpdatePayrollHistoryParams {
            title: params
                .title
                .as_deref()
                .map(|v| Self::normalize_field(v, "title"))
                .transpose()?,
            period: params
                .period
                .as_deref()
                .map(|v| Self::normalize_field(v, "period"))
                .transpose()?,
            start_date: params.start_date,
            end_date: params.end_date,
            status: params.status,
            total_employees: params
                .total_employees
                .map(|v| Self::validate_total_employees(v).map(|_| v))
                .transpose()?,
            total_earnings: params
                .total_earnings
                .map(|v| Self::validate_amount(v, "total_earnings").map(|_| v))
                .transpose()?,
            total_deductions: params
                .total_deductions
                .map(|v| Self::validate_amount(v, "total_deductions").map(|_| v))
                .transpose()?,
        };

        self.repository.update(history_id, updates).await
    }

    pub async fn delete(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<bool> {
        if self
            .get(organization_id, payroll_id, history_id)
            .await?
            .is_none()
        {
            return Ok(false);
        }

        self.repository.delete(history_id).await
    }

    pub async fn ensure_exists(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<PayrollHistory> {
        self.get(organization_id, payroll_id, history_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "payroll history `{history_id}` not found for payroll `{payroll_id}`"
                ))
            })
    }

    async fn get_organization_name(&self, organization_id: Uuid) -> AppResult<String> {
        let organization = self
            .organization_service
            .get(organization_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!("organization `{organization_id}` not found"))
            })?;
        Ok(organization.name)
    }

    fn normalize_field(value: &str, field: &str) -> AppResult<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(format!("{field} cannot be empty")));
        }
        Ok(trimmed.to_string())
    }

    fn validate_date_range(start_date: NaiveDate, end_date: NaiveDate) -> AppResult<()> {
        if end_date < start_date {
            return Err(AppError::validation("end_date cannot be before start_date"));
        }
        Ok(())
    }

    fn validate_total_employees(value: i32) -> AppResult<()> {
        if value < 0 {
            return Err(AppError::validation("total_employees cannot be negative"));
        }
        Ok(())
    }

    fn validate_amount(value: f64, field: &str) -> AppResult<()> {
        if !value.is_finite() {
            return Err(AppError::validation(format!(
                "{field} must be a valid number"
            )));
        }
        if value < 0.0 {
            return Err(AppError::validation(format!("{field} cannot be negative")));
        }
        Ok(())
    }
}
