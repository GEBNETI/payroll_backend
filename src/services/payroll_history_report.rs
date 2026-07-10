use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{
        division::Division, payroll_concept::PayrollConceptType, payroll_history::PayrollHistory,
        payroll_history_detail::PayrollHistoryDetail,
    },
    error::{AppError, AppResult},
    services::{
        division::DivisionService, organization::OrganizationService,
        payroll_history::PayrollHistoryService,
        payroll_history_detail::PayrollHistoryDetailService,
    },
};

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct EarningsDeductionsReport {
    pub payroll_history_id: Uuid,
    pub organization_name: String,
    pub payroll_name: String,
    pub title: String,
    pub period: String,
    #[schema(value_type = String, format = Date)]
    pub start_date: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub end_date: NaiveDate,
    pub earnings: Vec<PayrollConceptReport>,
    pub total_earnings: f64,
    pub deductions: Vec<PayrollConceptReport>,
    pub total_deductions: f64,
    pub net_total: f64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PayrollConceptReport {
    pub payroll_concept_id: Uuid,
    pub code: String,
    pub name: String,
    pub employees: Vec<PayrollConceptEmployeeReport>,
    pub total: f64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PayrollConceptEmployeeReport {
    pub employee_id: Uuid,
    pub employee_id_number: String,
    pub employee_full_name: String,
    pub amount: f64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PayrollReport {
    pub payroll_history_id: Uuid,
    pub organization_name: String,
    pub payroll_name: String,
    pub title: String,
    pub period: String,
    #[schema(value_type = String, format = Date)]
    pub start_date: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub end_date: NaiveDate,
    pub divisions: Vec<PayrollReportDivision>,
    pub summary: PayrollReportSummary,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PayrollReportDivision {
    pub division_id: Uuid,
    pub division_name: String,
    pub division_budget_code: String,
    pub employees: Vec<PayrollReportEmployee>,
    pub subdivisions: Vec<PayrollReportSubdivision>,
    pub summary: PayrollReportSummary,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PayrollReportSubdivision {
    pub division_id: Uuid,
    pub division_name: String,
    pub division_budget_code: String,
    pub employees: Vec<PayrollReportEmployee>,
    pub summary: PayrollReportSummary,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PayrollReportEmployee {
    pub employee_id: Uuid,
    pub employee_id_number: String,
    pub employee_full_name: String,
    pub summary: PayrollReportSummary,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PayrollReportSummary {
    pub earnings: Vec<PayrollReportConceptSummary>,
    pub total_earnings: f64,
    pub deductions: Vec<PayrollReportConceptSummary>,
    pub total_deductions: f64,
    pub net_total: f64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct PayrollReportConceptSummary {
    pub payroll_concept_id: Uuid,
    pub code: String,
    pub name: String,
    pub total: f64,
}

pub struct PatriaTextFile {
    pub filename: String,
    pub content: String,
}

#[derive(Clone)]
pub struct PayrollHistoryReportService {
    payroll_history_service: Arc<PayrollHistoryService>,
    payroll_history_detail_service: Arc<PayrollHistoryDetailService>,
    division_service: Arc<DivisionService>,
    organization_service: Arc<OrganizationService>,
}

impl PayrollHistoryReportService {
    pub fn new(
        payroll_history_service: Arc<PayrollHistoryService>,
        payroll_history_detail_service: Arc<PayrollHistoryDetailService>,
        division_service: Arc<DivisionService>,
        organization_service: Arc<OrganizationService>,
    ) -> Self {
        Self {
            payroll_history_service,
            payroll_history_detail_service,
            division_service,
            organization_service,
        }
    }

    pub async fn earnings_deductions(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<EarningsDeductionsReport> {
        let history = self
            .payroll_history_service
            .ensure_exists(organization_id, payroll_id, history_id)
            .await?;
        let details = self
            .payroll_history_detail_service
            .list(organization_id, payroll_id, history_id)
            .await?;

        Ok(Self::build_earnings_deductions_report(history, details))
    }

    pub async fn payroll(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<PayrollReport> {
        let history = self
            .payroll_history_service
            .ensure_exists(organization_id, payroll_id, history_id)
            .await?;
        let details = self
            .payroll_history_detail_service
            .list(organization_id, payroll_id, history_id)
            .await?;
        let divisions = self
            .division_service
            .list(organization_id, payroll_id)
            .await?;

        Ok(Self::build_payroll_report(history, details, divisions))
    }

    pub async fn patria_text_file(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
        bank_id: Uuid,
        payment_date: NaiveDate,
    ) -> AppResult<PatriaTextFile> {
        let history = self
            .payroll_history_service
            .ensure_exists(organization_id, payroll_id, history_id)
            .await?;
        let organization = self
            .organization_service
            .get(organization_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!("organization `{organization_id}` not found"))
            })?;
        let details: Vec<PayrollHistoryDetail> = self
            .payroll_history_detail_service
            .list(organization_id, payroll_id, history_id)
            .await?
            .into_iter()
            .filter(|detail| detail.bank_id == bank_id)
            .collect();

        if details.is_empty() {
            return Err(AppError::not_found(format!(
                "no payroll history details found for bank `{bank_id}`"
            )));
        }

        Self::build_patria_text_file(history, organization.rif, details, payment_date)
    }

    fn build_earnings_deductions_report(
        history: PayrollHistory,
        details: Vec<PayrollHistoryDetail>,
    ) -> EarningsDeductionsReport {
        let mut earnings = BTreeMap::new();
        let mut deductions = BTreeMap::new();

        for detail in details {
            let reports = match detail.payroll_concept_type {
                PayrollConceptType::Earning => &mut earnings,
                PayrollConceptType::Deduction => &mut deductions,
            };
            let key = (
                detail.payroll_concept_code.clone(),
                detail.payroll_concept_name.clone(),
                detail.payroll_concept_id,
            );
            let report = reports.entry(key).or_insert_with(|| PayrollConceptReport {
                payroll_concept_id: detail.payroll_concept_id,
                code: detail.payroll_concept_code.clone(),
                name: detail.payroll_concept_name.clone(),
                employees: Vec::new(),
                total: 0.0,
            });

            report.total += detail.amount;
            report.employees.push(PayrollConceptEmployeeReport {
                employee_id: detail.employee_id,
                employee_id_number: detail.employee_id_number,
                employee_full_name: format!(
                    "{} {}",
                    detail.employee_first_name, detail.employee_last_name
                ),
                amount: detail.amount,
            });
        }

        let earnings = Self::into_sorted_reports(earnings);
        let deductions = Self::into_sorted_reports(deductions);
        let total_earnings = earnings.iter().map(|report| report.total).sum();
        let total_deductions = deductions.iter().map(|report| report.total).sum();

        EarningsDeductionsReport {
            payroll_history_id: history.id,
            organization_name: history.organization_name,
            payroll_name: history.payroll_name,
            title: history.title,
            period: history.period,
            start_date: history.start_date,
            end_date: history.end_date,
            earnings,
            total_earnings,
            deductions,
            total_deductions,
            net_total: total_earnings - total_deductions,
        }
    }

    fn build_patria_text_file(
        history: PayrollHistory,
        rif: String,
        details: Vec<PayrollHistoryDetail>,
        payment_date: NaiveDate,
    ) -> AppResult<PatriaTextFile> {
        let mut details_by_employee: HashMap<Uuid, Vec<PayrollHistoryDetail>> = HashMap::new();
        for detail in details {
            details_by_employee
                .entry(detail.employee_id)
                .or_default()
                .push(detail);
        }

        let mut records = Vec::new();
        let mut total_cents = 0_u64;
        let mut bank_name = None;
        for employee_details in details_by_employee.into_values() {
            let sample = employee_details
                .first()
                .expect("employee details must not be empty");
            let earnings: f64 = employee_details
                .iter()
                .filter(|detail| detail.payroll_concept_type == PayrollConceptType::Earning)
                .map(|detail| detail.amount)
                .sum();
            let deductions: f64 = employee_details
                .iter()
                .filter(|detail| detail.payroll_concept_type == PayrollConceptType::Deduction)
                .map(|detail| detail.amount)
                .sum();
            let amount_cents = Self::amount_to_cents(earnings - deductions, "employee net amount")?;
            total_cents = total_cents.checked_add(amount_cents).ok_or_else(|| {
                AppError::validation("total amount exceeds the Patria file limit")
            })?;
            bank_name.get_or_insert_with(|| sample.bank_name.clone());

            records.push((
                sample.employee_id_number.clone(),
                format!(
                    "{} {}",
                    sample.employee_first_name, sample.employee_last_name
                ),
                sample.employee_bank_account.clone(),
                amount_cents,
            ));
        }
        records.sort_by(|a, b| a.0.cmp(&b.0));

        let record_count = u64::try_from(records.len())
            .map_err(|_| AppError::validation("record count exceeds the Patria file limit"))?;
        let date = payment_date.format("%Y%m%d").to_string();
        let mut content = format!(
            "ONTNOM{}{}{}VES{}\n",
            Self::normalize_rif(&rif)?,
            Self::fixed_number(record_count, 7, "record count")?,
            Self::fixed_number(total_cents, 15, "total amount")?,
            date,
        );

        for (id_number, full_name, bank_account, amount_cents) in records {
            let (prefix, digits) = Self::split_id_number(&id_number)?;
            let bank_account = Self::normalize_bank_account(&bank_account)?;
            content.push_str(&format!(
                "{}{}{}{}{}\n",
                prefix,
                Self::fixed_number(digits, 8, "id number")?,
                bank_account,
                Self::fixed_number(amount_cents, 11, "employee net amount")?,
                Self::fixed_text(&full_name, 40),
            ));
        }

        let filename = format!(
            "{}-{}-{}-{}-patria.txt",
            Self::slug(&history.organization_name),
            Self::slug(&history.payroll_name),
            date,
            Self::slug(bank_name.as_deref().unwrap_or("bank")),
        );
        Ok(PatriaTextFile { filename, content })
    }

    fn normalize_rif(value: &str) -> AppResult<String> {
        let rif = value.trim().to_ascii_uppercase();
        let mut characters = rif.chars();
        let Some(prefix) = characters.next() else {
            return Err(AppError::validation("organization rif is invalid"));
        };
        if !matches!(prefix, 'G' | 'J')
            || rif.len() != 10
            || !characters.all(|value| value.is_ascii_digit())
        {
            return Err(AppError::validation("organization rif is invalid"));
        }
        Ok(rif)
    }

    fn split_id_number(value: &str) -> AppResult<(char, u64)> {
        let id_number = value.trim().to_ascii_uppercase();
        let mut characters = id_number.chars();
        let Some(prefix) = characters.next() else {
            return Err(AppError::validation("employee id number is invalid"));
        };
        let digits: String = characters.collect();
        if !matches!(prefix, 'V' | 'E')
            || digits.len() != 8
            || !digits.chars().all(|value| value.is_ascii_digit())
        {
            return Err(AppError::validation("employee id number is invalid"));
        }
        let digits = digits
            .parse()
            .map_err(|_| AppError::validation("employee id number is invalid"))?;
        Ok((prefix, digits))
    }

    fn normalize_bank_account(value: &str) -> AppResult<String> {
        let bank_account = value.trim();
        if bank_account.len() != 20 || !bank_account.chars().all(|value| value.is_ascii_digit()) {
            return Err(AppError::validation("employee bank account is invalid"));
        }
        Ok(bank_account.to_string())
    }

    fn amount_to_cents(value: f64, field: &str) -> AppResult<u64> {
        if !value.is_finite() || value < 0.0 {
            return Err(AppError::validation(format!("{field} cannot be negative")));
        }
        let cents = (value * 100.0).round();
        if cents > u64::MAX as f64 {
            return Err(AppError::validation(format!(
                "{field} exceeds the Patria file limit"
            )));
        }
        Ok(cents as u64)
    }

    fn fixed_number(value: u64, width: usize, field: &str) -> AppResult<String> {
        let value = value.to_string();
        if value.len() > width {
            return Err(AppError::validation(format!(
                "{field} exceeds the Patria file limit"
            )));
        }
        Ok(format!("{value:0>width$}"))
    }

    fn fixed_text(value: &str, width: usize) -> String {
        let value: String = value.chars().take(width).collect();
        format!("{value:width$}")
    }

    fn slug(value: &str) -> String {
        let slug = value
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>();
        let slug = slug.trim_matches('-');
        if slug.is_empty() {
            "unknown".to_string()
        } else {
            slug.to_string()
        }
    }

    fn into_sorted_reports(
        reports: BTreeMap<(String, String, Uuid), PayrollConceptReport>,
    ) -> Vec<PayrollConceptReport> {
        reports
            .into_values()
            .map(|mut report| {
                report.employees.sort_by(|a, b| {
                    a.employee_id_number
                        .cmp(&b.employee_id_number)
                        .then_with(|| a.employee_full_name.cmp(&b.employee_full_name))
                        .then_with(|| a.employee_id.cmp(&b.employee_id))
                });
                report
            })
            .collect()
    }

    fn build_payroll_report(
        history: PayrollHistory,
        details: Vec<PayrollHistoryDetail>,
        divisions: Vec<Division>,
    ) -> PayrollReport {
        let division_by_id: HashMap<Uuid, Division> = divisions
            .into_iter()
            .map(|division| (division.id, division))
            .collect();
        let mut details_by_root: HashMap<Uuid, Vec<PayrollHistoryDetail>> = HashMap::new();

        for detail in &details {
            let root_id = Self::root_division_id(detail.division_id, &division_by_id);
            details_by_root
                .entry(root_id)
                .or_default()
                .push(detail.clone());
        }

        let mut divisions: Vec<PayrollReportDivision> = details_by_root
            .into_iter()
            .map(|(root_id, root_details)| {
                let mut direct_details = Vec::new();
                let mut details_by_subdivision: HashMap<Uuid, Vec<PayrollHistoryDetail>> =
                    HashMap::new();

                for detail in &root_details {
                    if detail.division_id == root_id {
                        direct_details.push(detail.clone());
                    } else {
                        details_by_subdivision
                            .entry(detail.division_id)
                            .or_default()
                            .push(detail.clone());
                    }
                }

                let mut subdivisions: Vec<PayrollReportSubdivision> = details_by_subdivision
                    .into_iter()
                    .map(|(division_id, subdivision_details)| {
                        let division = division_by_id.get(&division_id);
                        let sample = subdivision_details
                            .first()
                            .expect("subdivision details must not be empty");
                        PayrollReportSubdivision {
                            division_id,
                            division_name: division
                                .map(|value| value.name.clone())
                                .unwrap_or_else(|| sample.division_name.clone()),
                            division_budget_code: division
                                .map(|value| value.budget_code.clone())
                                .unwrap_or_else(|| sample.division_budget_code.clone()),
                            employees: Self::build_employee_reports(subdivision_details.clone()),
                            summary: Self::build_summary(&subdivision_details),
                        }
                    })
                    .collect();
                subdivisions.sort_by(|a, b| a.division_name.cmp(&b.division_name));

                let sample = root_details
                    .first()
                    .expect("root division details must not be empty");
                let division = division_by_id.get(&root_id);
                PayrollReportDivision {
                    division_id: root_id,
                    division_name: division
                        .map(|value| value.name.clone())
                        .unwrap_or_else(|| sample.division_name.clone()),
                    division_budget_code: division
                        .map(|value| value.budget_code.clone())
                        .unwrap_or_else(|| sample.division_budget_code.clone()),
                    employees: Self::build_employee_reports(direct_details),
                    subdivisions,
                    summary: Self::build_summary(&root_details),
                }
            })
            .collect();
        divisions.sort_by(|a, b| a.division_name.cmp(&b.division_name));

        PayrollReport {
            payroll_history_id: history.id,
            organization_name: history.organization_name,
            payroll_name: history.payroll_name,
            title: history.title,
            period: history.period,
            start_date: history.start_date,
            end_date: history.end_date,
            divisions,
            summary: Self::build_summary(&details),
        }
    }

    fn root_division_id(division_id: Uuid, divisions: &HashMap<Uuid, Division>) -> Uuid {
        let mut current_id = division_id;
        let mut visited = HashSet::new();

        while visited.insert(current_id) {
            match divisions
                .get(&current_id)
                .and_then(|division| division.parent_division_id)
            {
                Some(parent_id) => current_id = parent_id,
                None => break,
            }
        }

        current_id
    }

    fn build_employee_reports(details: Vec<PayrollHistoryDetail>) -> Vec<PayrollReportEmployee> {
        let mut details_by_employee: HashMap<Uuid, Vec<PayrollHistoryDetail>> = HashMap::new();
        for detail in details {
            details_by_employee
                .entry(detail.employee_id)
                .or_default()
                .push(detail);
        }

        let mut employees: Vec<PayrollReportEmployee> = details_by_employee
            .into_values()
            .map(|employee_details| {
                let sample = employee_details
                    .first()
                    .expect("employee details must not be empty");
                PayrollReportEmployee {
                    employee_id: sample.employee_id,
                    employee_id_number: sample.employee_id_number.clone(),
                    employee_full_name: format!(
                        "{} {}",
                        sample.employee_first_name, sample.employee_last_name
                    ),
                    summary: Self::build_summary(&employee_details),
                }
            })
            .collect();
        employees.sort_by(|a, b| {
            a.employee_id_number
                .cmp(&b.employee_id_number)
                .then_with(|| a.employee_full_name.cmp(&b.employee_full_name))
        });
        employees
    }

    fn build_summary(details: &[PayrollHistoryDetail]) -> PayrollReportSummary {
        let mut earnings = BTreeMap::new();
        let mut deductions = BTreeMap::new();

        for detail in details {
            let summaries = match detail.payroll_concept_type {
                PayrollConceptType::Earning => &mut earnings,
                PayrollConceptType::Deduction => &mut deductions,
            };
            let key = (
                detail.payroll_concept_code.clone(),
                detail.payroll_concept_name.clone(),
                detail.payroll_concept_id,
            );
            let summary = summaries
                .entry(key)
                .or_insert_with(|| PayrollReportConceptSummary {
                    payroll_concept_id: detail.payroll_concept_id,
                    code: detail.payroll_concept_code.clone(),
                    name: detail.payroll_concept_name.clone(),
                    total: 0.0,
                });
            summary.total += detail.amount;
        }

        let earnings: Vec<PayrollReportConceptSummary> = earnings.into_values().collect();
        let deductions: Vec<PayrollReportConceptSummary> = deductions.into_values().collect();
        let total_earnings = earnings.iter().map(|summary| summary.total).sum();
        let total_deductions = deductions.iter().map(|summary| summary.total).sum();

        PayrollReportSummary {
            earnings,
            total_earnings,
            deductions,
            total_deductions,
            net_total: total_earnings - total_deductions,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use uuid::Uuid;

    use super::*;
    use crate::domain::{
        division::Division,
        payroll_concept::{PayrollConceptPeriod, PayrollConceptScope},
        payroll_history::{NewPayrollHistoryData, PayrollHistoryStatus},
        payroll_history_detail::NewPayrollHistoryDetailData,
    };

    fn history() -> PayrollHistory {
        PayrollHistory::new(NewPayrollHistoryData {
            id: Uuid::nil(),
            title: "July payroll".to_string(),
            period: "1".to_string(),
            start_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
            created_at: chrono::Utc::now(),
            organization_id: Uuid::new_v4(),
            organization_name: "Acme".to_string(),
            payroll_id: Uuid::new_v4(),
            payroll_name: "Main".to_string(),
            status: PayrollHistoryStatus::Draft,
            total_employees: 0,
            total_earnings: 0.0,
            total_deductions: 0.0,
        })
    }

    fn detail(
        concept_id: Uuid,
        code: &str,
        name: &str,
        concept_type: PayrollConceptType,
        id_number: &str,
        first_name: &str,
        last_name: &str,
        amount: f64,
    ) -> PayrollHistoryDetail {
        PayrollHistoryDetail::new(NewPayrollHistoryDetailData {
            id: Uuid::new_v4(),
            payroll_history_id: Uuid::nil(),
            division_id: Uuid::new_v4(),
            division_name: "Operations".to_string(),
            division_budget_code: "OPS".to_string(),
            job_id: Uuid::new_v4(),
            job_title: "Engineer".to_string(),
            job_salary: 1.0,
            employee_id: Uuid::new_v4(),
            employee_id_number: id_number.to_string(),
            employee_last_name: last_name.to_string(),
            employee_first_name: first_name.to_string(),
            employee_hire_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            employee_salary: 1.0,
            employee_clasification: "regular".to_string(),
            employee_hours: 40,
            employee_bank_account: "123".to_string(),
            bank_id: Uuid::new_v4(),
            bank_name: "Bank".to_string(),
            payroll_concept_id: concept_id,
            payroll_concept_code: code.to_string(),
            payroll_concept_name: name.to_string(),
            payroll_concept_type: concept_type,
            payroll_concept_scope: PayrollConceptScope::Global,
            payroll_concept_period: PayrollConceptPeriod::One,
            amount,
        })
    }

    #[test]
    fn groups_details_by_type_and_concept_and_calculates_totals() {
        let salary_id = Uuid::new_v4();
        let bonus_id = Uuid::new_v4();
        let tax_id = Uuid::new_v4();
        let report = PayrollHistoryReportService::build_earnings_deductions_report(
            history(),
            vec![
                detail(
                    tax_id,
                    "TAX",
                    "Income tax",
                    PayrollConceptType::Deduction,
                    "V-2",
                    "Zoë",
                    "Adams",
                    25.0,
                ),
                detail(
                    salary_id,
                    "SALARY",
                    "Salary",
                    PayrollConceptType::Earning,
                    "V-2",
                    "Zoë",
                    "Adams",
                    100.0,
                ),
                detail(
                    salary_id,
                    "SALARY",
                    "Salary",
                    PayrollConceptType::Earning,
                    "V-1",
                    "Ana",
                    "Baker",
                    150.0,
                ),
                detail(
                    bonus_id,
                    "BONUS",
                    "Bonus",
                    PayrollConceptType::Earning,
                    "V-1",
                    "Ana",
                    "Baker",
                    50.0,
                ),
            ],
        );

        assert_eq!(report.earnings.len(), 2);
        assert_eq!(report.organization_name, "Acme");
        assert_eq!(report.payroll_name, "Main");
        assert_eq!(report.title, "July payroll");
        assert_eq!(
            report.start_date,
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
        );
        assert_eq!(
            report.end_date,
            NaiveDate::from_ymd_opt(2026, 7, 15).unwrap()
        );
        assert_eq!(report.earnings[0].code, "BONUS");
        assert_eq!(report.earnings[0].total, 50.0);
        assert_eq!(report.earnings[1].code, "SALARY");
        assert_eq!(report.earnings[1].total, 250.0);
        assert_eq!(report.earnings[1].employees[0].employee_id_number, "V-1");
        assert_eq!(report.total_earnings, 300.0);
        assert_eq!(report.deductions[0].total, 25.0);
        assert_eq!(report.total_deductions, 25.0);
        assert_eq!(report.net_total, 275.0);
    }

    #[test]
    fn groups_payroll_report_by_division_and_subdivision() {
        let root_id = Uuid::new_v4();
        let subdivision_id = Uuid::new_v4();
        let payroll_id = Uuid::new_v4();
        let mut root_detail = detail(
            Uuid::new_v4(),
            "SALARY",
            "Salary",
            PayrollConceptType::Earning,
            "V-1",
            "Ana",
            "Baker",
            150.0,
        );
        root_detail.division_id = root_id;
        root_detail.division_name = "Administration".to_string();
        root_detail.division_budget_code = "ADM".to_string();

        let mut subdivision_detail = detail(
            Uuid::new_v4(),
            "TAX",
            "Income tax",
            PayrollConceptType::Deduction,
            "V-2",
            "Zoë",
            "Adams",
            25.0,
        );
        subdivision_detail.division_id = subdivision_id;
        subdivision_detail.division_name = "Human Resources".to_string();
        subdivision_detail.division_budget_code = "HR".to_string();

        let report = PayrollHistoryReportService::build_payroll_report(
            history(),
            vec![root_detail, subdivision_detail],
            vec![
                Division::new(root_id, "Administration", "", "ADM", payroll_id, None),
                Division::new(
                    subdivision_id,
                    "Human Resources",
                    "",
                    "HR",
                    payroll_id,
                    Some(root_id),
                ),
            ],
        );

        assert_eq!(report.divisions.len(), 1);
        assert_eq!(report.divisions[0].division_name, "Administration");
        assert_eq!(report.divisions[0].employees.len(), 1);
        assert_eq!(report.divisions[0].subdivisions.len(), 1);
        assert_eq!(
            report.divisions[0].subdivisions[0].division_name,
            "Human Resources"
        );
        assert_eq!(report.divisions[0].summary.total_earnings, 150.0);
        assert_eq!(report.divisions[0].summary.total_deductions, 25.0);
        assert_eq!(report.summary.net_total, 125.0);
    }

    #[test]
    fn builds_a_fixed_width_patria_text_file_for_one_bank() {
        let bank_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let mut earning = detail(
            Uuid::new_v4(),
            "SALARY",
            "Salary",
            PayrollConceptType::Earning,
            "V00000001",
            "Ana",
            "Baker",
            100.50,
        );
        earning.employee_id = employee_id;
        earning.bank_id = bank_id;
        earning.bank_name = "Bank".to_string();
        earning.employee_bank_account = "01020000000000000001".to_string();

        let mut deduction = detail(
            Uuid::new_v4(),
            "TAX",
            "Income tax",
            PayrollConceptType::Deduction,
            "V00000001",
            "Ana",
            "Baker",
            0.50,
        );
        deduction.employee_id = employee_id;
        deduction.bank_id = bank_id;
        deduction.bank_name = "Bank".to_string();
        deduction.employee_bank_account = "01020000000000000001".to_string();

        let file = PayrollHistoryReportService::build_patria_text_file(
            history(),
            "G000000000".to_string(),
            vec![earning, deduction],
            NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
        )
        .unwrap();

        let lines: Vec<&str> = file.content.lines().collect();
        assert_eq!(lines[0], "ONTNOMG0000000000000001000000000010000VES20260710");
        assert_eq!(lines[1].len(), 80);
        assert_eq!(&lines[1][0..9], "V00000001");
        assert_eq!(&lines[1][9..29], "01020000000000000001");
        assert_eq!(&lines[1][29..40], "00000010000");
        assert_eq!(file.filename, "acme-main-20260710-bank-patria.txt");
    }
}
