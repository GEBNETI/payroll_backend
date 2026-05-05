use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Map, Value as JsonValue};
use surrealdb::{engine::any::Any, types::{RecordId, SurrealValue}, Connection, Surreal};
use uuid::Uuid;

use crate::{
    domain::payroll_history::{NewPayrollHistoryData, PayrollHistory, PayrollHistoryStatus},
    error::{parse_thing_id, parse_uuid_field, AppError, AppResult},
    services::payroll_history::{PayrollHistoryRepository, UpdatePayrollHistoryParams},
};

const TABLE: &str = "payroll_history";

#[derive(Clone)]
pub struct SurrealPayrollHistoryRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealPayrollHistoryRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> PayrollHistoryRepository for SurrealPayrollHistoryRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    async fn insert(&self, data: NewPayrollHistoryData) -> AppResult<PayrollHistory> {
        let id = data.id;
        let _: Option<JsonValue> = self
            .client
            .create((TABLE, id.to_string()))
            .content(json!({
                "title": data.title,
                "period": data.period,
                "start_date": data.start_date.to_string(),
                "end_date": data.end_date.to_string(),
                "created_at": data.created_at.to_rfc3339(),
                "organization_id": data.organization_id.to_string(),
                "organization_name": data.organization_name,
                "payroll_id": data.payroll_id.to_string(),
                "payroll_name": data.payroll_name,
                "status": status_to_string(&data.status),
                "total_employees": data.total_employees,
                "total_earnings": data.total_earnings,
                "total_deductions": data.total_deductions,
            }))
            .await?;

        self.fetch(id)
            .await?
            .ok_or_else(|| AppError::internal("database did not return created payroll history"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<PayrollHistory>> {
        let rid = RecordId::new(TABLE, id.to_string());
        let mut response = self
            .client
            .query("SELECT * FROM payroll_history WHERE id = $rid")
            .bind(("rid", rid))
            .await?;

        let result: Result<Option<PayrollHistoryRecord>, _> = response.take(0);
        match result {
            Ok(record) => record.map(record_to_domain).transpose(),
            Err(e) if e.to_string().contains("does not exist") => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<PayrollHistory>> {
        let mut response = self
            .client
            .query("SELECT * FROM payroll_history WHERE payroll_id = $payroll_id")
            .bind(("payroll_id", payroll_id.to_string()))
            .await?;

        let result: Result<Vec<PayrollHistoryRecord>, _> = response.take(0);
        match result {
            Ok(records) => records.into_iter().map(record_to_domain).collect(),
            Err(e) if e.to_string().contains("does not exist") => Ok(vec![]),
            Err(e) => Err(AppError::from(e)),
        }
    }

    async fn update(
        &self,
        id: Uuid,
        params: UpdatePayrollHistoryParams,
    ) -> AppResult<Option<PayrollHistory>> {
        let payload = build_update_payload(params)?;

        let _: Option<JsonValue> = self
            .client
            .update((TABLE, id.to_string()))
            .merge(payload)
            .await?;

        self.fetch(id).await
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<JsonValue> = self
            .client
            .delete((TABLE, id.to_string()))
            .await?;

        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize, SurrealValue)]
struct PayrollHistoryRecord {
    id: RecordId,
    title: String,
    period: String,
    start_date: String,
    end_date: String,
    created_at: String,
    organization_id: String,
    organization_name: String,
    payroll_id: String,
    payroll_name: String,
    status: String,
    total_employees: i32,
    total_earnings: f64,
    total_deductions: f64,
}

fn record_to_domain(record: PayrollHistoryRecord) -> AppResult<PayrollHistory> {
    let id = parse_thing_id(&record.id.key, "stored payroll history id")?;
    let organization_id = parse_uuid_field(
        &record.organization_id,
        "stored payroll history organization_id",
    )?;
    let payroll_id = parse_uuid_field(&record.payroll_id, "stored payroll history payroll_id")?;
    let start_date = parse_date(&record.start_date, "start_date")?;
    let end_date = parse_date(&record.end_date, "end_date")?;
    let created_at = parse_datetime(&record.created_at, "created_at")?;
    let status = parse_status(&record.status)?;

    Ok(PayrollHistory::new(NewPayrollHistoryData {
        id,
        title: record.title,
        period: record.period,
        start_date,
        end_date,
        created_at,
        organization_id,
        organization_name: record.organization_name,
        payroll_id,
        payroll_name: record.payroll_name,
        status,
        total_employees: record.total_employees,
        total_earnings: record.total_earnings,
        total_deductions: record.total_deductions,
    }))
}

fn parse_date(value: &str, field: &str) -> AppResult<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::internal(format!("stored {field} is not a valid date")))
}

fn parse_datetime(value: &str, field: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| AppError::internal(format!("stored {field} is not a valid datetime")))
}

fn parse_status(value: &str) -> AppResult<PayrollHistoryStatus> {
    match value {
        "draft" => Ok(PayrollHistoryStatus::Draft),
        "finalized" => Ok(PayrollHistoryStatus::Finalized),
        "archived" => Ok(PayrollHistoryStatus::Archived),
        _ => Err(AppError::internal(format!(
            "unknown payroll history status: {value}"
        ))),
    }
}

fn status_to_string(status: &PayrollHistoryStatus) -> &'static str {
    match status {
        PayrollHistoryStatus::Draft => "draft",
        PayrollHistoryStatus::Finalized => "finalized",
        PayrollHistoryStatus::Archived => "archived",
    }
}

fn build_update_payload(params: UpdatePayrollHistoryParams) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(title) = params.title {
        object.insert("title".to_string(), JsonValue::String(title));
    }
    if let Some(period) = params.period {
        object.insert("period".to_string(), JsonValue::String(period));
    }
    if let Some(start_date) = params.start_date {
        object.insert("start_date".to_string(), JsonValue::String(start_date.to_string()));
    }
    if let Some(end_date) = params.end_date {
        object.insert("end_date".to_string(), JsonValue::String(end_date.to_string()));
    }
    if let Some(status) = params.status {
        object.insert("status".to_string(), JsonValue::String(status_to_string(&status).to_string()));
    }
    if let Some(total_employees) = params.total_employees {
        object.insert("total_employees".to_string(), JsonValue::from(total_employees));
    }
    if let Some(total_earnings) = params.total_earnings {
        object.insert("total_earnings".to_string(), JsonValue::from(total_earnings));
    }
    if let Some(total_deductions) = params.total_deductions {
        object.insert("total_deductions".to_string(), JsonValue::from(total_deductions));
    }

    if object.is_empty() {
        return Err(AppError::internal(
            "no fields supplied for payroll history update",
        ));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyPayrollHistoryRepository = SurrealPayrollHistoryRepository<Any>;
