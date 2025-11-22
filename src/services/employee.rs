use std::sync::Arc;

use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    domain::employee::Employee,
    error::{AppError, AppResult},
    services::{
        bank::BankService, division::DivisionService, job::JobService, payroll::PayrollService,
    },
};

#[derive(Debug, Clone)]
pub struct CreateEmployeeParams {
    pub id_number: String,
    pub last_name: String,
    pub first_name: String,
    pub address: String,
    pub phone: String,
    pub place_of_birth: String,
    pub date_of_birth: NaiveDate,
    pub nationality: String,
    pub marital_status: String,
    pub gender: String,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub clasification: String,
    pub job_id: Uuid,
    pub bank_id: Uuid,
    pub bank_account: String,
    pub status: String,
    pub hours: i32,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateEmployeeParams {
    pub id_number: Option<String>,
    pub last_name: Option<String>,
    pub first_name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub place_of_birth: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub nationality: Option<String>,
    pub marital_status: Option<String>,
    pub gender: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<Option<NaiveDate>>,
    pub clasification: Option<String>,
    pub job_id: Option<Uuid>,
    pub bank_id: Option<Uuid>,
    pub bank_account: Option<String>,
    pub status: Option<String>,
    pub hours: Option<i32>,
}

#[async_trait]
pub trait EmployeeRepository: Send + Sync {
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
        termination_date: Option<NaiveDate>,
        clasification: String,
        job_id: Uuid,
        bank_id: Uuid,
        bank_account: String,
        status: String,
        hours: i32,
        division_id: Uuid,
        payroll_id: Uuid,
    ) -> AppResult<Employee>;

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Employee>>;

    async fn fetch_by_division(&self, division_id: Uuid) -> AppResult<Vec<Employee>>;

    async fn update(&self, id: Uuid, updates: UpdateEmployeeParams) -> AppResult<Option<Employee>>;

    async fn delete(&self, id: Uuid) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct EmployeeService {
    repository: Arc<dyn EmployeeRepository>,
    division_service: Arc<DivisionService>,
    payroll_service: Arc<PayrollService>,
    job_service: Arc<JobService>,
    bank_service: Arc<BankService>,
}

impl EmployeeService {
    pub fn new(
        repository: Arc<dyn EmployeeRepository>,
        division_service: Arc<DivisionService>,
        payroll_service: Arc<PayrollService>,
        job_service: Arc<JobService>,
        bank_service: Arc<BankService>,
    ) -> Self {
        Self {
            repository,
            division_service,
            payroll_service,
            job_service,
            bank_service,
        }
    }

    pub async fn create(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        params: CreateEmployeeParams,
    ) -> AppResult<Employee> {
        let division = self
            .division_service
            .get(organization_id, payroll_id, division_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "division `{division_id}` not found for payroll `{payroll_id}` in organization `{organization_id}`"
                ))
            })?;

        self.ensure_job_belongs(organization_id, payroll_id, params.job_id)
            .await?;
        self.ensure_bank_belongs(organization_id, params.bank_id)
            .await?;

        let id_number = Self::normalize_field(&params.id_number, "id number")?;
        let last_name = Self::normalize_field(&params.last_name, "last name")?;
        let first_name = Self::normalize_field(&params.first_name, "first name")?;
        let address = Self::normalize_field(&params.address, "address")?;
        let phone = Self::normalize_field(&params.phone, "phone")?;
        let place_of_birth = Self::normalize_field(&params.place_of_birth, "place of birth")?;
        let nationality = Self::normalize_field(&params.nationality, "nationality")?;
        let marital_status = Self::normalize_field(&params.marital_status, "marital status")?;
        let gender = Self::normalize_field(&params.gender, "gender")?;
        let clasification = Self::normalize_field(&params.clasification, "clasification")?;
        let bank_account = Self::normalize_field(&params.bank_account, "bank account")?;
        let status = Self::normalize_field(&params.status, "status")?;
        let hours = Self::validate_hours(params.hours)?;
        let hire_date = params.hire_date;
        let termination_date = Self::validate_termination_date(hire_date, params.termination_date)?;

        let id = Uuid::new_v4();
        self.repository
            .insert(
                id,
                id_number,
                last_name,
                first_name,
                address,
                phone,
                place_of_birth,
                params.date_of_birth,
                nationality,
                marital_status,
                gender,
                hire_date,
                termination_date,
                clasification,
                params.job_id,
                params.bank_id,
                bank_account,
                status,
                hours,
                division.id,
                payroll_id,
            )
            .await
    }

    pub async fn get(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
    ) -> AppResult<Option<Employee>> {
        self.ensure_division_accessible(organization_id, payroll_id, division_id)
            .await?;
        let employee = self.repository.fetch(employee_id).await?;
        Ok(employee.filter(|employee| {
            employee.division_id == division_id && employee.payroll_id == payroll_id
        }))
    }

    pub async fn list(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
    ) -> AppResult<Vec<Employee>> {
        self.ensure_division_accessible(organization_id, payroll_id, division_id)
            .await?;
        let mut employees = self.repository.fetch_by_division(division_id).await?;
        employees.sort_by(|a, b| {
            a.last_name
                .cmp(&b.last_name)
                .then_with(|| a.first_name.cmp(&b.first_name))
        });
        Ok(employees)
    }

    pub async fn update(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
        params: UpdateEmployeeParams,
    ) -> AppResult<Option<Employee>> {
        if params.id_number.is_none()
            && params.last_name.is_none()
            && params.first_name.is_none()
            && params.address.is_none()
            && params.phone.is_none()
            && params.place_of_birth.is_none()
            && params.date_of_birth.is_none()
            && params.nationality.is_none()
            && params.marital_status.is_none()
            && params.gender.is_none()
            && params.hire_date.is_none()
            && params.termination_date.is_none()
            && params.clasification.is_none()
            && params.job_id.is_none()
            && params.bank_id.is_none()
            && params.bank_account.is_none()
            && params.status.is_none()
            && params.hours.is_none()
        {
            return Err(AppError::validation("no fields supplied for update"));
        }

        let employee = match self
            .get(organization_id, payroll_id, division_id, employee_id)
            .await?
        {
            Some(employee) => employee,
            None => return Ok(None),
        };

        if let Some(job_id) = params.job_id {
            self.ensure_job_belongs(organization_id, payroll_id, job_id)
                .await?;
        }

        if let Some(bank_id) = params.bank_id {
            self.ensure_bank_belongs(organization_id, bank_id).await?;
        }

        let hire_date = params.hire_date.unwrap_or(employee.hire_date);
        let termination_date = match params.termination_date {
            Some(value) => Some(Self::validate_termination_date(hire_date, value)?),
            None => None,
        };

        let updates = UpdateEmployeeParams {
            id_number: params
                .id_number
                .as_deref()
                .map(|value| Self::normalize_field(value, "id number"))
                .transpose()?,
            last_name: params
                .last_name
                .as_deref()
                .map(|value| Self::normalize_field(value, "last name"))
                .transpose()?,
            first_name: params
                .first_name
                .as_deref()
                .map(|value| Self::normalize_field(value, "first name"))
                .transpose()?,
            address: params
                .address
                .as_deref()
                .map(|value| Self::normalize_field(value, "address"))
                .transpose()?,
            phone: params
                .phone
                .as_deref()
                .map(|value| Self::normalize_field(value, "phone"))
                .transpose()?,
            place_of_birth: params
                .place_of_birth
                .as_deref()
                .map(|value| Self::normalize_field(value, "place of birth"))
                .transpose()?,
            date_of_birth: params.date_of_birth,
            nationality: params
                .nationality
                .as_deref()
                .map(|value| Self::normalize_field(value, "nationality"))
                .transpose()?,
            marital_status: params
                .marital_status
                .as_deref()
                .map(|value| Self::normalize_field(value, "marital status"))
                .transpose()?,
            gender: params
                .gender
                .as_deref()
                .map(|value| Self::normalize_field(value, "gender"))
                .transpose()?,
            hire_date: params.hire_date,
            termination_date,
            clasification: params
                .clasification
                .as_deref()
                .map(|value| Self::normalize_field(value, "clasification"))
                .transpose()?,
            job_id: params.job_id,
            bank_id: params.bank_id,
            bank_account: params
                .bank_account
                .as_deref()
                .map(|value| Self::normalize_field(value, "bank account"))
                .transpose()?,
            status: params
                .status
                .as_deref()
                .map(|value| Self::normalize_field(value, "status"))
                .transpose()?,
            hours: params.hours.map(Self::validate_hours).transpose()?,
        };

        self.repository.update(employee_id, updates).await
    }

    pub async fn delete(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
        employee_id: Uuid,
    ) -> AppResult<bool> {
        if self
            .get(organization_id, payroll_id, division_id, employee_id)
            .await?
            .is_none()
        {
            return Ok(false);
        }

        self.repository.delete(employee_id).await
    }

    async fn ensure_division_accessible(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        division_id: Uuid,
    ) -> AppResult<()> {
        self.payroll_service
            .ensure_belongs_to_organization(organization_id, payroll_id)
            .await?;
        match self
            .division_service
            .get(organization_id, payroll_id, division_id)
            .await?
        {
            Some(_) => Ok(()),
            None => Err(AppError::not_found(format!(
                "division `{division_id}` not found for payroll `{payroll_id}` in organization `{organization_id}`"
            ))),
        }
    }

    async fn ensure_job_belongs(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        job_id: Uuid,
    ) -> AppResult<()> {
        match self
            .job_service
            .get(organization_id, payroll_id, job_id)
            .await?
        {
            Some(job) if job.payroll_id == payroll_id => Ok(()),
            _ => Err(AppError::not_found(format!(
                "job `{job_id}` not found for payroll `{payroll_id}`"
            ))),
        }
    }

    async fn ensure_bank_belongs(&self, organization_id: Uuid, bank_id: Uuid) -> AppResult<()> {
        match self.bank_service.get(organization_id, bank_id).await? {
            Some(bank) if bank.organization_id == organization_id => Ok(()),
            _ => Err(AppError::not_found(format!(
                "bank `{bank_id}` not found for organization `{organization_id}`"
            ))),
        }
    }

    fn normalize_field(value: &str, field: &str) -> AppResult<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(format!("{field} cannot be empty")));
        }

        Ok(trimmed.to_string())
    }

    fn validate_hours(value: i32) -> AppResult<i32> {
        if value < 0 {
            return Err(AppError::validation("hours cannot be negative"));
        }

        Ok(value)
    }

    fn validate_termination_date(
        hire_date: NaiveDate,
        termination_date: Option<NaiveDate>,
    ) -> AppResult<Option<NaiveDate>> {
        if let Some(date) = termination_date {
            if date < hire_date {
                return Err(AppError::validation(
                    "termination date cannot be before hire date",
                ));
            }
            Ok(Some(date))
        } else {
            Ok(None)
        }
    }
}
