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
    error::AppResult,
    services::{
        division::DivisionService, payroll_history::PayrollHistoryService,
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

#[derive(Clone)]
pub struct PayrollHistoryReportService {
    payroll_history_service: Arc<PayrollHistoryService>,
    payroll_history_detail_service: Arc<PayrollHistoryDetailService>,
    division_service: Arc<DivisionService>,
}

impl PayrollHistoryReportService {
    pub fn new(
        payroll_history_service: Arc<PayrollHistoryService>,
        payroll_history_detail_service: Arc<PayrollHistoryDetailService>,
        division_service: Arc<DivisionService>,
    ) -> Self {
        Self {
            payroll_history_service,
            payroll_history_detail_service,
            division_service,
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
}
