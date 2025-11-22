use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use nomina::{
    domain::{
        bank::Bank, division::Division, employee::Employee, job::Job, organization::Organization,
        payroll::Payroll,
    },
    error::AppResult,
    services::{
        bank::BankRepository,
        division::DivisionRepository,
        employee::{EmployeeRepository, UpdateEmployeeParams},
        job::JobRepository,
        organization::OrganizationRepository,
        payroll::PayrollRepository,
    },
};

#[derive(Default)]
pub struct InMemoryOrganizationRepository {
    store: RwLock<HashMap<Uuid, Organization>>,
}

#[derive(Default)]
pub struct InMemoryBankRepository {
    store: RwLock<HashMap<Uuid, Bank>>,
}

#[async_trait]
impl OrganizationRepository for InMemoryOrganizationRepository {
    async fn insert(&self, id: Uuid, name: String) -> AppResult<Organization> {
        let organization = Organization::new(id, name);
        self.store
            .write()
            .await
            .insert(organization.id, organization.clone());
        Ok(organization)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Organization>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_all(&self) -> AppResult<Vec<Organization>> {
        Ok(self.store.read().await.values().cloned().collect())
    }

    async fn update(&self, id: Uuid, name: Option<String>) -> AppResult<Option<Organization>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(name) = name {
                existing.name = name;
            }
            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}

#[async_trait]
impl BankRepository for InMemoryBankRepository {
    async fn insert(&self, id: Uuid, name: String, organization_id: Uuid) -> AppResult<Bank> {
        let bank = Bank::new(id, name, organization_id);
        self.store.write().await.insert(bank.id, bank.clone());
        Ok(bank)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Bank>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Bank>> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|bank| bank.organization_id == organization_id)
            .cloned()
            .collect())
    }

    async fn update(&self, id: Uuid, name: Option<String>) -> AppResult<Option<Bank>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(name) = name {
                existing.name = name;
            }
            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}

#[derive(Default)]
pub struct InMemoryPayrollRepository {
    store: RwLock<HashMap<Uuid, Payroll>>,
}

#[async_trait]
impl PayrollRepository for InMemoryPayrollRepository {
    async fn insert(
        &self,
        id: Uuid,
        name: String,
        description: String,
        organization_id: Uuid,
    ) -> AppResult<Payroll> {
        let payroll = Payroll::new(id, name, description, organization_id);
        self.store.write().await.insert(payroll.id, payroll.clone());
        Ok(payroll)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Payroll>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_by_organization(&self, organization_id: Uuid) -> AppResult<Vec<Payroll>> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|payroll| payroll.organization_id == organization_id)
            .cloned()
            .collect())
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> AppResult<Option<Payroll>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(name) = name {
                existing.name = name;
            }
            if let Some(description) = description {
                existing.description = description;
            }

            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}

#[derive(Default)]
pub struct InMemoryDivisionRepository {
    store: RwLock<HashMap<Uuid, Division>>,
}

#[async_trait]
impl DivisionRepository for InMemoryDivisionRepository {
    async fn insert(
        &self,
        id: Uuid,
        name: String,
        description: String,
        budget_code: String,
        payroll_id: Uuid,
        parent_division_id: Option<Uuid>,
    ) -> AppResult<Division> {
        let division = Division::new(
            id,
            name,
            description,
            budget_code,
            payroll_id,
            parent_division_id,
        );
        self.store
            .write()
            .await
            .insert(division.id, division.clone());
        Ok(division)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Division>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Division>> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|division| division.payroll_id == payroll_id)
            .cloned()
            .collect())
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        budget_code: Option<String>,
        parent_division_id: Option<Option<Uuid>>,
    ) -> AppResult<Option<Division>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(name) = name {
                existing.name = name;
            }
            if let Some(description) = description {
                existing.description = description;
            }
            if let Some(budget_code) = budget_code {
                existing.budget_code = budget_code;
            }
            if let Some(parent) = parent_division_id {
                existing.parent_division_id = parent;
            }

            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}

#[derive(Default)]
pub struct InMemoryJobRepository {
    store: RwLock<HashMap<Uuid, Job>>,
}

#[async_trait]
impl JobRepository for InMemoryJobRepository {
    async fn insert(
        &self,
        id: Uuid,
        job_title: String,
        salary: f64,
        payroll_id: Uuid,
    ) -> AppResult<Job> {
        let job = Job::new(id, job_title, salary, payroll_id);
        self.store.write().await.insert(job.id, job.clone());
        Ok(job)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Job>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_by_payroll(&self, payroll_id: Uuid) -> AppResult<Vec<Job>> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|job| job.payroll_id == payroll_id)
            .cloned()
            .collect())
    }

    async fn update(
        &self,
        id: Uuid,
        job_title: Option<String>,
        salary: Option<f64>,
    ) -> AppResult<Option<Job>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(job_title) = job_title {
                existing.job_title = job_title;
            }
            if let Some(salary) = salary {
                existing.salary = salary;
            }
            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}

#[derive(Default)]
pub struct InMemoryEmployeeRepository {
    store: RwLock<HashMap<Uuid, Employee>>,
}

#[async_trait]
impl EmployeeRepository for InMemoryEmployeeRepository {
    async fn insert(
        &self,
        id: Uuid,
        id_number: String,
        last_name: String,
        first_name: String,
        address: String,
        phone: String,
        place_of_birth: String,
        date_of_birth: chrono::NaiveDate,
        nationality: String,
        marital_status: String,
        gender: String,
        hire_date: chrono::NaiveDate,
        termination_date: Option<chrono::NaiveDate>,
        clasification: String,
        job_id: Uuid,
        bank_id: Uuid,
        bank_account: String,
        status: String,
        hours: i32,
        division_id: Uuid,
        payroll_id: Uuid,
    ) -> AppResult<Employee> {
        let employee = Employee::new(
            id,
            id_number,
            last_name,
            first_name,
            address,
            phone,
            place_of_birth,
            date_of_birth,
            nationality,
            marital_status,
            gender,
            hire_date,
            termination_date,
            clasification,
            job_id,
            bank_id,
            bank_account,
            status,
            hours,
            division_id,
            payroll_id,
        );
        self.store
            .write()
            .await
            .insert(employee.id, employee.clone());
        Ok(employee)
    }

    async fn fetch(&self, id: Uuid) -> AppResult<Option<Employee>> {
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn fetch_by_division(&self, division_id: Uuid) -> AppResult<Vec<Employee>> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|employee| employee.division_id == division_id)
            .cloned()
            .collect())
    }

    async fn update(&self, id: Uuid, updates: UpdateEmployeeParams) -> AppResult<Option<Employee>> {
        let mut guard = self.store.write().await;
        if let Some(existing) = guard.get_mut(&id) {
            if let Some(id_number) = updates.id_number {
                existing.id_number = id_number;
            }
            if let Some(last_name) = updates.last_name {
                existing.last_name = last_name;
            }
            if let Some(first_name) = updates.first_name {
                existing.first_name = first_name;
            }
            if let Some(address) = updates.address {
                existing.address = address;
            }
            if let Some(phone) = updates.phone {
                existing.phone = phone;
            }
            if let Some(place_of_birth) = updates.place_of_birth {
                existing.place_of_birth = place_of_birth;
            }
            if let Some(date_of_birth) = updates.date_of_birth {
                existing.date_of_birth = date_of_birth;
            }
            if let Some(nationality) = updates.nationality {
                existing.nationality = nationality;
            }
            if let Some(marital_status) = updates.marital_status {
                existing.marital_status = marital_status;
            }
            if let Some(gender) = updates.gender {
                existing.gender = gender;
            }
            if let Some(hire_date) = updates.hire_date {
                existing.hire_date = hire_date;
            }
            if let Some(termination_date) = updates.termination_date {
                existing.termination_date = termination_date;
            }
            if let Some(clasification) = updates.clasification {
                existing.clasification = clasification;
            }
            if let Some(job_id) = updates.job_id {
                existing.job_id = job_id;
            }
            if let Some(bank_id) = updates.bank_id {
                existing.bank_id = bank_id;
            }
            if let Some(bank_account) = updates.bank_account {
                existing.bank_account = bank_account;
            }
            if let Some(status) = updates.status {
                existing.status = status;
            }
            if let Some(hours) = updates.hours {
                existing.hours = hours;
            }

            return Ok(Some(existing.clone()));
        }

        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.store.write().await.remove(&id).is_some())
    }
}
