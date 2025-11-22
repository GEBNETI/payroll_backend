use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{Map, Value as JsonValue, json};
use surrealdb::{
    Connection, Surreal,
    engine::any::Any,
    sql::{Id, Thing},
};
use uuid::Uuid;

use crate::{
    domain::employee::Employee,
    error::{AppError, AppResult},
    services::employee::{EmployeeRepository, UpdateEmployeeParams},
};

const EMPLOYEE_TABLE: &str = "employee";

#[derive(Clone)]
pub struct SurrealEmployeeRepository<C>
where
    C: Connection,
{
    client: Surreal<C>,
}

impl<C> SurrealEmployeeRepository<C>
where
    C: Connection,
{
    pub fn new(client: Surreal<C>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<C> EmployeeRepository for SurrealEmployeeRepository<C>
where
    C: Connection + Clone + Send + Sync + 'static,
{
    #[allow(clippy::too_many_arguments)]
    async fn insert(
        &self,
        id: Uuid,
        id_number: String,
        last_name: String,
        first_name: String,
        address: String,
        phone: String,
        place_of_birth: String,
        date_of_birth: NaiveDate,
        nationality: String,
        marital_status: String,
        gender: String,
        hire_date: NaiveDate,
        leaving_date: Option<NaiveDate>,
        clasification: String,
        job_id: Uuid,
        bank_id: Uuid,
        bank_account: String,
        status: String,
        hours: i32,
        division_id: Uuid,
        payroll_id: Uuid,
    ) -> AppResult<Employee> {
        let record: Option<EmployeeRecord> = self
            .client
            .create((EMPLOYEE_TABLE, id.to_string()))
            .content(json!({
                "id_number": id_number,
                "last_name": last_name,
                "first_name": first_name,
                "address": address,
                "phone": phone,
                "place_of_birth": place_of_birth,
                "date_of_birth": date_of_birth.to_string(),
                "nationality": nationality,
                "marital_status": marital_status,
                "gender": gender,
                "hire_date": hire_date.to_string(),
                "leaving_date": leaving_date.map(|date| date.to_string()),
                "clasification": clasification,
                "job_id": job_id,
                "bank_id": bank_id,
                "bank_account": bank_account,
                "status": status,
                "hours": hours,
                "division_id": division_id,
                "payroll_id": payroll_id,
            }))
            .await?;

        record
            .map(record_to_domain)
            .transpose()?
            .ok_or_else(|| AppError::internal("database did not return created employee"))
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Employee>> {
        let record: Option<EmployeeRecord> =
            self.client.select((EMPLOYEE_TABLE, id.to_string())).await?;

        record.map(record_to_domain).transpose()
    }

    async fn fetch_by_division(&self, division_id: Uuid) -> AppResult<Vec<Employee>> {
        let records: Vec<EmployeeRecord> = self.client.select(EMPLOYEE_TABLE).await?;
        records
            .into_iter()
            .filter(|record| record.division_id == division_id.to_string())
            .map(record_to_domain)
            .collect()
    }

    async fn update(&self, id: Uuid, updates: UpdateEmployeeParams) -> AppResult<Option<Employee>> {
        let payload = build_update_payload(updates)?;
        let record: Option<EmployeeRecord> = self
            .client
            .update((EMPLOYEE_TABLE, id.to_string()))
            .merge(payload)
            .await?;

        record.map(record_to_domain).transpose()
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        let record: Option<EmployeeRecord> =
            self.client.delete((EMPLOYEE_TABLE, id.to_string())).await?;
        Ok(record.is_some())
    }
}

#[derive(Debug, Deserialize)]
struct EmployeeRecord {
    id: Thing,
    id_number: String,
    last_name: String,
    first_name: String,
    address: String,
    phone: String,
    place_of_birth: String,
    date_of_birth: String,
    nationality: String,
    marital_status: String,
    gender: String,
    hire_date: String,
    leaving_date: Option<String>,
    clasification: String,
    job_id: String,
    bank_id: String,
    bank_account: String,
    status: String,
    hours: i32,
    division_id: String,
    payroll_id: String,
}

fn record_to_domain(record: EmployeeRecord) -> AppResult<Employee> {
    let id = match record.id.id {
        Id::String(value) => Uuid::parse_str(&value)
            .map_err(|_| AppError::internal("stored employee id is not a UUID"))?,
        Id::Uuid(value) => uuid::Uuid::from(value),
        _ => {
            return Err(AppError::internal(
                "stored employee identifier is not a supported format",
            ));
        }
    };

    let division_id = Uuid::parse_str(&record.division_id)
        .map_err(|_| AppError::internal("stored division id is not a UUID"))?;
    let payroll_id = Uuid::parse_str(&record.payroll_id)
        .map_err(|_| AppError::internal("stored payroll id is not a UUID"))?;
    let job_id = Uuid::parse_str(&record.job_id)
        .map_err(|_| AppError::internal("stored job id is not a UUID"))?;
    let bank_id = Uuid::parse_str(&record.bank_id)
        .map_err(|_| AppError::internal("stored bank id is not a UUID"))?;
    let date_of_birth = parse_date(&record.date_of_birth, "date of birth")?;
    let hire_date = parse_date(&record.hire_date, "hire date")?;
    let leaving_date = match record.leaving_date {
        Some(value) => Some(parse_date(&value, "leaving date")?),
        None => None,
    };

    Ok(Employee::new(
        id,
        record.id_number,
        record.last_name,
        record.first_name,
        record.address,
        record.phone,
        record.place_of_birth,
        date_of_birth,
        record.nationality,
        record.marital_status,
        record.gender,
        hire_date,
        leaving_date,
        record.clasification,
        job_id,
        bank_id,
        record.bank_account,
        record.status,
        record.hours,
        division_id,
        payroll_id,
    ))
}

fn parse_date(value: &str, field: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::internal(format!("stored {field} is not a valid date")))
}

fn build_update_payload(updates: UpdateEmployeeParams) -> AppResult<JsonValue> {
    let mut object = Map::new();

    if let Some(id_number) = updates.id_number {
        object.insert("id_number".to_string(), JsonValue::String(id_number));
    }

    if let Some(last_name) = updates.last_name {
        object.insert("last_name".to_string(), JsonValue::String(last_name));
    }

    if let Some(first_name) = updates.first_name {
        object.insert("first_name".to_string(), JsonValue::String(first_name));
    }

    if let Some(address) = updates.address {
        object.insert("address".to_string(), JsonValue::String(address));
    }

    if let Some(phone) = updates.phone {
        object.insert("phone".to_string(), JsonValue::String(phone));
    }

    if let Some(place_of_birth) = updates.place_of_birth {
        object.insert(
            "place_of_birth".to_string(),
            JsonValue::String(place_of_birth),
        );
    }

    if let Some(date_of_birth) = updates.date_of_birth {
        object.insert(
            "date_of_birth".to_string(),
            JsonValue::String(date_of_birth.to_string()),
        );
    }

    if let Some(nationality) = updates.nationality {
        object.insert("nationality".to_string(), JsonValue::String(nationality));
    }

    if let Some(marital_status) = updates.marital_status {
        object.insert(
            "marital_status".to_string(),
            JsonValue::String(marital_status),
        );
    }

    if let Some(gender) = updates.gender {
        object.insert("gender".to_string(), JsonValue::String(gender));
    }

    if let Some(hire_date) = updates.hire_date {
        object.insert(
            "hire_date".to_string(),
            JsonValue::String(hire_date.to_string()),
        );
    }

    if let Some(leaving_date) = updates.leaving_date {
        match leaving_date {
            Some(value) => {
                object.insert(
                    "leaving_date".to_string(),
                    JsonValue::String(value.to_string()),
                );
            }
            None => {
                object.insert("leaving_date".to_string(), JsonValue::Null);
            }
        }
    }

    if let Some(clasification) = updates.clasification {
        object.insert(
            "clasification".to_string(),
            JsonValue::String(clasification),
        );
    }

    if let Some(job_id) = updates.job_id {
        object.insert("job_id".to_string(), JsonValue::String(job_id.to_string()));
    }

    if let Some(bank_id) = updates.bank_id {
        object.insert(
            "bank_id".to_string(),
            JsonValue::String(bank_id.to_string()),
        );
    }

    if let Some(bank_account) = updates.bank_account {
        object.insert("bank_account".to_string(), JsonValue::String(bank_account));
    }

    if let Some(status) = updates.status {
        object.insert("status".to_string(), JsonValue::String(status));
    }

    if let Some(hours) = updates.hours {
        object.insert("hours".to_string(), JsonValue::from(hours));
    }

    if object.is_empty() {
        return Err(AppError::internal("no fields supplied for employee update"));
    }

    Ok(JsonValue::Object(object))
}

pub type SurrealAnyEmployeeRepository = SurrealEmployeeRepository<Any>;
