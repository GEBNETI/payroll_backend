use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
pub struct Employee {
    pub id: Uuid,
    pub id_number: String,
    pub last_name: String,
    pub first_name: String,
    pub address: String,
    pub phone: String,
    pub place_of_birth: String,
    #[schema(value_type = String, format = Date)]
    pub date_of_birth: NaiveDate,
    pub nationality: String,
    pub marital_status: String,
    pub gender: String,
    #[schema(value_type = String, format = Date)]
    pub hire_date: NaiveDate,
    #[schema(value_type = Option<String>, format = Date)]
    pub termination_date: Option<NaiveDate>,
    pub clasification: String,
    pub job_id: Uuid,
    pub bank_id: Uuid,
    pub bank_account: String,
    pub status: String,
    pub hours: i32,
    pub division_id: Uuid,
    pub payroll_id: Uuid,
}

impl Employee {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(
        id: Uuid,
        id_number: impl Into<String>,
        last_name: impl Into<String>,
        first_name: impl Into<String>,
        address: impl Into<String>,
        phone: impl Into<String>,
        place_of_birth: impl Into<String>,
        date_of_birth: NaiveDate,
        nationality: impl Into<String>,
        marital_status: impl Into<String>,
        gender: impl Into<String>,
        hire_date: NaiveDate,
        termination_date: Option<NaiveDate>,
        clasification: impl Into<String>,
        job_id: Uuid,
        bank_id: Uuid,
        bank_account: impl Into<String>,
        status: impl Into<String>,
        hours: i32,
        division_id: Uuid,
        payroll_id: Uuid,
    ) -> Self {
        Self {
            id,
            id_number: id_number.into(),
            last_name: last_name.into(),
            first_name: first_name.into(),
            address: address.into(),
            phone: phone.into(),
            place_of_birth: place_of_birth.into(),
            date_of_birth,
            nationality: nationality.into(),
            marital_status: marital_status.into(),
            gender: gender.into(),
            hire_date,
            termination_date,
            clasification: clasification.into(),
            job_id,
            bank_id,
            bank_account: bank_account.into(),
            status: status.into(),
            hours,
            division_id,
            payroll_id,
        }
    }
}
