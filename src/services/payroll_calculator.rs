use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use evalexpr::{ContextWithMutableVariables, HashMapContext};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{
        employee::Employee,
        job::Job,
        payroll_concept::{
            PayrollConcept, PayrollConceptPeriod, PayrollConceptScope, PayrollConceptType,
        },
        payroll_concept_definition::PayrollConceptDefinition,
        payroll_history::PayrollHistoryStatus,
    },
    error::{AppError, AppResult},
    services::{
        division::DivisionService,
        employee::EmployeeService,
        employee_payroll_concept::EmployeePayrollConceptService,
        job::JobService,
        payroll_concept::PayrollConceptService,
        payroll_concept_definition::PayrollConceptDefinitionService,
        payroll_history::PayrollHistoryService,
        payroll_history_detail::{CreatePayrollHistoryDetailParams, PayrollHistoryDetailService},
    },
};

/// Maximum allowed length for formulas and conditions (DoS protection)
const MAX_FORMULA_LENGTH: usize = 1000;
/// Maximum allowed length for condition expressions
const MAX_CONDITION_LENGTH: usize = 500;
/// Maximum number of warnings to return (prevents huge responses on large payrolls)
const MAX_WARNINGS: usize = 100;

/// Base context variable names that are always available in formulas
const BASE_CONTEXT_VARS: &[&str] = &["salary", "hours", "classification", "job_salary", "job_title"];

#[derive(Debug, Serialize, ToSchema)]
pub struct CalculationResult {
    pub total_employees: i32,
    pub total_details_created: i32,
    pub total_earnings: f64,
    pub total_deductions: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CleanResult {
    pub total_details_deleted: usize,
}

/// Holds a global concept with its prefetched definition
struct GlobalConceptWithDefinition {
    concept_id: Uuid,
    concept_code: String,
    concept_type: PayrollConceptType,
    definition: PayrollConceptDefinition,
    sanitized_code: String,
}

#[derive(Clone)]
pub struct PayrollCalculatorService {
    payroll_history_service: Arc<PayrollHistoryService>,
    payroll_history_detail_service: Arc<PayrollHistoryDetailService>,
    division_service: Arc<DivisionService>,
    employee_service: Arc<EmployeeService>,
    employee_payroll_concept_service: Arc<EmployeePayrollConceptService>,
    payroll_concept_service: Arc<PayrollConceptService>,
    payroll_concept_definition_service: Arc<PayrollConceptDefinitionService>,
    job_service: Arc<JobService>,
}

impl PayrollCalculatorService {
    pub fn new(
        payroll_history_service: Arc<PayrollHistoryService>,
        payroll_history_detail_service: Arc<PayrollHistoryDetailService>,
        division_service: Arc<DivisionService>,
        employee_service: Arc<EmployeeService>,
        employee_payroll_concept_service: Arc<EmployeePayrollConceptService>,
        payroll_concept_service: Arc<PayrollConceptService>,
        payroll_concept_definition_service: Arc<PayrollConceptDefinitionService>,
        job_service: Arc<JobService>,
    ) -> Self {
        Self {
            payroll_history_service,
            payroll_history_detail_service,
            division_service,
            employee_service,
            employee_payroll_concept_service,
            payroll_concept_service,
            payroll_concept_definition_service,
            job_service,
        }
    }

    pub async fn calculate(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<CalculationResult> {
        // 1. Validate PayrollHistory exists and is in draft status
        let history = self
            .payroll_history_service
            .get(organization_id, payroll_id, history_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "payroll history `{}` not found for payroll `{}`",
                    history_id, payroll_id
                ))
            })?;

        // 2. Validate status is draft
        if history.status != PayrollHistoryStatus::Draft {
            return Err(AppError::validation(format!(
                "Cannot calculate payroll history `{}`: status must be 'draft', but is '{:?}'",
                history_id, history.status
            )));
        }

        // 3. Validate period
        Self::validate_period(&history.period)?;

        // 4. Clean existing details first (idempotency) - fail fast on errors
        self.clean_internal(organization_id, payroll_id, history_id)
            .await?;

        // 5. Fetch and filter concepts by period
        let all_concepts = self
            .payroll_concept_service
            .list(organization_id, payroll_id)
            .await?;

        let filtered_concepts: Vec<PayrollConcept> = all_concepts
            .into_iter()
            .filter(|c| Self::should_include_concept(&history.period, &c.period))
            .collect();

        // 6. Separate concepts by scope
        let individual_concepts: Vec<&PayrollConcept> = filtered_concepts
            .iter()
            .filter(|c| c.scope == PayrollConceptScope::Individual)
            .collect();

        let global_concepts: Vec<&PayrollConcept> = filtered_concepts
            .iter()
            .filter(|c| c.scope == PayrollConceptScope::Global)
            .collect();

        let mut warnings = Vec::new();

        // 7. Build sanitized code mapping and detect collisions (across all scopes + base vars)
        let all_concepts: Vec<&PayrollConcept> = individual_concepts
            .iter()
            .chain(global_concepts.iter())
            .copied()
            .collect();
        let (unified_code_map, collisions, base_var_collisions) =
            Self::build_sanitized_code_map_with_base_vars(&all_concepts);

        // Extract individual and global code maps from unified map
        let individual_code_map: HashMap<String, String> = individual_concepts
            .iter()
            .filter_map(|c| unified_code_map.get(&c.code).map(|s| (c.code.clone(), s.clone())))
            .collect();
        let global_code_map: HashMap<String, String> = global_concepts
            .iter()
            .filter_map(|c| unified_code_map.get(&c.code).map(|s| (c.code.clone(), s.clone())))
            .collect();

        // Add collision warnings (within concepts)
        for (sanitized, originals) in collisions {
            warnings.push(format!(
                "Variable name collision: codes {:?} all map to `{}`. Values may be overwritten.",
                originals, sanitized
            ));
        }

        // Add base variable collision warnings (concept codes that shadow base vars)
        for (concept_code, base_var) in base_var_collisions {
            warnings.push(format!(
                "Concept code `{}` shadows base context variable `{}`. The concept value will override the employee/job field.",
                concept_code, base_var
            ));
        }

        // 8. Prefetch all global concept definitions and validate formulas
        let mut globals_with_defs: Vec<GlobalConceptWithDefinition> = Vec::new();
        for concept in &global_concepts {
            let definition = match self
                .payroll_concept_definition_service
                .get(organization_id, payroll_id, concept.id)
                .await
            {
                Ok(Some(def)) => def,
                Ok(None) => {
                    warnings.push(format!(
                        "Global concept `{}` skipped: no definition found",
                        concept.code
                    ));
                    continue;
                }
                Err(e) => {
                    warnings.push(format!(
                        "Global concept `{}` skipped: failed to fetch definition: {}",
                        concept.code, e
                    ));
                    continue;
                }
            };

            // Validate formula and condition length (DoS protection)
            if definition.formula.len() > MAX_FORMULA_LENGTH {
                warnings.push(format!(
                    "Global concept `{}` skipped: formula exceeds maximum length of {} characters",
                    concept.code, MAX_FORMULA_LENGTH
                ));
                continue;
            }
            if definition.condition.len() > MAX_CONDITION_LENGTH {
                warnings.push(format!(
                    "Global concept `{}` skipped: condition exceeds maximum length of {} characters",
                    concept.code, MAX_CONDITION_LENGTH
                ));
                continue;
            }

            // Pre-parse validation: try to build the expression tree to catch syntax errors early
            if let Err(e) = Self::validate_formula_syntax(&definition.formula) {
                warnings.push(format!(
                    "Global concept `{}` skipped: invalid formula syntax: {}",
                    concept.code, e
                ));
                continue;
            }

            // Normalize empty/whitespace conditions to "true"
            let normalized_condition = Self::normalize_condition(&definition.condition);

            if let Err(e) = Self::validate_condition_syntax(&normalized_condition) {
                warnings.push(format!(
                    "Global concept `{}` skipped: invalid condition syntax: {}",
                    concept.code, e
                ));
                continue;
            }

            // Check for disallowed functions (regex, random) in formula and condition
            let disallowed_in_formula = Self::check_disallowed_functions(&definition.formula);
            let disallowed_in_condition = Self::check_disallowed_functions(&normalized_condition);

            if !disallowed_in_formula.is_empty() {
                warnings.push(format!(
                    "Global concept `{}` skipped: formula contains disallowed functions: {}. These functions are not permitted in payroll formulas.",
                    concept.code,
                    disallowed_in_formula.join(", ")
                ));
                continue;
            }

            if !disallowed_in_condition.is_empty() {
                warnings.push(format!(
                    "Global concept `{}` skipped: condition contains disallowed functions: {}. These functions are not permitted in payroll conditions.",
                    concept.code,
                    disallowed_in_condition.join(", ")
                ));
                continue;
            }

            // Store the normalized condition back into the definition for use during evaluation
            let mut definition = definition;
            definition.condition = normalized_condition;

            let sanitized_code = global_code_map
                .get(&concept.code)
                .cloned()
                .unwrap_or_else(|| Self::sanitize_variable_name(&concept.code));

            globals_with_defs.push(GlobalConceptWithDefinition {
                concept_id: concept.id,
                concept_code: concept.code.clone(),
                concept_type: concept.concept_type.clone(),
                definition,
                sanitized_code,
            });
        }

        // 9. Sort global concepts by dependency order (topological sort)
        let all_sanitized_codes: HashSet<String> = globals_with_defs
            .iter()
            .map(|g| g.sanitized_code.clone())
            .chain(individual_code_map.values().cloned())
            .collect();

        let sorted_globals = match Self::topological_sort_globals(globals_with_defs, &all_sanitized_codes) {
            Ok(sorted) => sorted,
            Err(cycle_warning) => {
                // Circular dependency detected - fail with clear error
                return Err(AppError::validation(format!(
                    "Cannot calculate payroll: {}",
                    cycle_warning
                )));
            }
        };

        // 9.5. Check for undefined variable references in formulas/conditions
        let base_vars: HashSet<String> = BASE_CONTEXT_VARS.iter().map(|s| s.to_string()).collect();
        let all_available_vars: HashSet<String> = all_sanitized_codes
            .iter()
            .cloned()
            .chain(base_vars)
            .collect();

        for global in &sorted_globals {
            let formula_vars =
                Self::extract_variable_references(&global.definition.formula, &all_sanitized_codes);
            let condition_vars =
                Self::extract_variable_references(&global.definition.condition, &all_sanitized_codes);

            for var in formula_vars {
                if !all_available_vars.contains(&var) {
                    warnings.push(format!(
                        "Global concept `{}`: formula references undefined variable `{}`. This may cause evaluation errors.",
                        global.concept_code, var
                    ));
                }
            }

            for var in condition_vars {
                if !all_available_vars.contains(&var) {
                    warnings.push(format!(
                        "Global concept `{}`: condition references undefined variable `{}`. This may cause evaluation errors.",
                        global.concept_code, var
                    ));
                }
            }
        }

        // 10. Fetch all employees for the payroll (across all divisions)
        let divisions = self
            .division_service
            .list(organization_id, payroll_id)
            .await?;

        let mut employees = Vec::new();
        for division in divisions {
            let division_employees = self
                .employee_service
                .list(organization_id, payroll_id, division.id)
                .await?;
            employees.extend(division_employees);
        }

        let mut total_employees = 0;
        let mut total_details_created = 0;
        let mut total_earnings = 0.0;
        let mut total_deductions = 0.0;

        // 11. Process each employee
        for employee in employees {
            let employee_had_details = self
                .process_employee(
                    &employee,
                    &individual_concepts,
                    &individual_code_map,
                    &sorted_globals,
                    organization_id,
                    payroll_id,
                    history_id,
                    &mut warnings,
                    &mut total_details_created,
                    &mut total_earnings,
                    &mut total_deductions,
                )
                .await;

            if employee_had_details {
                total_employees += 1;
            }
        }

        // 12. Update PayrollHistory totals
        use crate::services::payroll_history::UpdatePayrollHistoryParams;
        self.payroll_history_service
            .update(
                organization_id,
                payroll_id,
                history_id,
                UpdatePayrollHistoryParams {
                    title: None,
                    period: None,
                    start_date: None,
                    end_date: None,
                    status: None,
                    total_employees: Some(total_employees),
                    total_earnings: Some(total_earnings),
                    total_deductions: Some(total_deductions),
                },
            )
            .await?;

        // Cap warnings to prevent huge responses on large payrolls
        let warnings = Self::cap_warnings(warnings);

        Ok(CalculationResult {
            total_employees,
            total_details_created,
            total_earnings,
            total_deductions,
            warnings,
        })
    }

    /// Caps the number of warnings to MAX_WARNINGS and adds a truncation message if needed.
    fn cap_warnings(mut warnings: Vec<String>) -> Vec<String> {
        if warnings.len() > MAX_WARNINGS {
            let total = warnings.len();
            warnings.truncate(MAX_WARNINGS - 1);
            warnings.push(format!(
                "... and {} more warnings (truncated to {} total)",
                total - MAX_WARNINGS + 1,
                MAX_WARNINGS
            ));
        }
        warnings
    }

    async fn process_employee(
        &self,
        employee: &Employee,
        individual_concepts: &[&PayrollConcept],
        individual_code_map: &HashMap<String, String>,
        sorted_globals: &[GlobalConceptWithDefinition],
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
        warnings: &mut Vec<String>,
        total_details_created: &mut i32,
        total_earnings: &mut f64,
        total_deductions: &mut f64,
    ) -> bool {
        let mut calculated_concepts: HashMap<String, f64> = HashMap::new();
        let mut employee_had_details = false;
        let mut employee_earnings = 0.0;
        let mut employee_deductions = 0.0;

        // Fetch job for context building
        let job = match self
            .job_service
            .get(organization_id, payroll_id, employee.job_id)
            .await
        {
            Ok(Some(j)) => j,
            Ok(None) => {
                warnings.push(format!(
                    "Employee `{}` skipped: job `{}` not found",
                    employee.id, employee.job_id
                ));
                return false;
            }
            Err(e) => {
                warnings.push(format!(
                    "Employee `{}` skipped: failed to fetch job: {}",
                    employee.id, e
                ));
                return false;
            }
        };

        // Process individual concepts
        let employee_concepts = match self
            .employee_payroll_concept_service
            .list(organization_id, payroll_id, employee.division_id, employee.id)
            .await
        {
            Ok(concepts) => concepts,
            Err(e) => {
                warnings.push(format!(
                    "Employee `{}`: failed to fetch payroll concepts: {}",
                    employee.id, e
                ));
                return false;
            }
        };

        for employee_concept in employee_concepts {
            // Find the matching individual concept
            if let Some(concept) = individual_concepts
                .iter()
                .find(|c| c.id == employee_concept.payroll_concept_id)
            {
                // Create detail
                match self
                    .payroll_history_detail_service
                    .create(
                        organization_id,
                        payroll_id,
                        history_id,
                        CreatePayrollHistoryDetailParams {
                            employee_id: employee.id,
                            payroll_concept_id: concept.id,
                            amount: employee_concept.amount,
                        },
                    )
                    .await
                {
                    Ok(_) => {
                        employee_had_details = true;
                        *total_details_created += 1;

                        // Update totals (both global and per-employee)
                        match concept.concept_type {
                            PayrollConceptType::Earning => {
                                *total_earnings += employee_concept.amount;
                                employee_earnings += employee_concept.amount;
                            }
                            PayrollConceptType::Deduction => {
                                *total_deductions += employee_concept.amount;
                                employee_deductions += employee_concept.amount;
                            }
                        }

                        // Store in calculated concepts map with sanitized code
                        let sanitized_code = individual_code_map
                            .get(&concept.code)
                            .cloned()
                            .unwrap_or_else(|| Self::sanitize_variable_name(&concept.code));
                        calculated_concepts.insert(sanitized_code, employee_concept.amount);
                    }
                    Err(e) => {
                        warnings.push(format!(
                            "Failed to create detail for employee `{}`, concept `{}`: {}",
                            employee.id, concept.id, e
                        ));
                    }
                }
            }
        }

        // Process global concepts (already sorted by dependency order)
        for global in sorted_globals {
            // Build evaluation context
            let context = match Self::build_eval_context(employee, &job, &calculated_concepts) {
                Ok(ctx) => ctx,
                Err(e) => {
                    warnings.push(format!(
                        "Employee `{}`, concept `{}`: failed to build context: {}",
                        employee.id, global.concept_code, e
                    ));
                    continue;
                }
            };

            // Evaluate condition
            let condition_result =
                match Self::evaluate_condition(&global.definition.condition, &context) {
                    Ok(result) => result,
                    Err(e) => {
                        warnings.push(format!(
                            "Employee `{}`, concept `{}`: condition evaluation error: {}",
                            employee.id, global.concept_code, e
                        ));
                        continue;
                    }
                };

            // Skip if condition is false (add warning for visibility)
            if !condition_result {
                warnings.push(format!(
                    "Employee `{}` skipped for concept `{}`: condition evaluated to false",
                    employee.id, global.concept_code
                ));
                continue;
            }

            // Evaluate formula
            let amount = match Self::evaluate_formula(&global.definition.formula, &context) {
                Ok(amt) => amt,
                Err(e) => {
                    warnings.push(format!(
                        "Employee `{}`, concept `{}`: formula evaluation error: {}",
                        employee.id, global.concept_code, e
                    ));
                    continue;
                }
            };

            // Create detail
            match self
                .payroll_history_detail_service
                .create(
                    organization_id,
                    payroll_id,
                    history_id,
                    CreatePayrollHistoryDetailParams {
                        employee_id: employee.id,
                        payroll_concept_id: global.concept_id,
                        amount,
                    },
                )
                .await
            {
                Ok(_) => {
                    employee_had_details = true;
                    *total_details_created += 1;

                    // Update totals (both global and per-employee)
                    match global.concept_type {
                        PayrollConceptType::Earning => {
                            *total_earnings += amount;
                            employee_earnings += amount;
                        }
                        PayrollConceptType::Deduction => {
                            *total_deductions += amount;
                            employee_deductions += amount;
                        }
                    }

                    // Store in calculated concepts map with sanitized code
                    calculated_concepts.insert(global.sanitized_code.clone(), amount);
                }
                Err(e) => {
                    warnings.push(format!(
                        "Failed to create detail for employee `{}`, concept `{}`: {}",
                        employee.id, global.concept_id, e
                    ));
                }
            }
        }

        // Check for negative balance (deductions greater than earnings)
        if employee_had_details && employee_deductions > employee_earnings {
            warnings.push(format!(
                "Employee `{}` has negative balance: deductions ({:.2}) exceed earnings ({:.2})",
                employee.id, employee_deductions, employee_earnings
            ));
        }

        employee_had_details
    }

    pub async fn clean(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<CleanResult> {
        // Validate status is draft
        let history = self
            .payroll_history_service
            .get(organization_id, payroll_id, history_id)
            .await?
            .ok_or_else(|| {
                AppError::not_found(format!(
                    "payroll history `{}` not found for payroll `{}`",
                    history_id, payroll_id
                ))
            })?;

        if history.status != PayrollHistoryStatus::Draft {
            return Err(AppError::validation(format!(
                "Cannot clean payroll history `{}`: status must be 'draft', but is '{:?}'",
                history_id, history.status
            )));
        }

        let deleted_count = self
            .clean_internal(organization_id, payroll_id, history_id)
            .await?;

        // Reset totals to 0
        use crate::services::payroll_history::UpdatePayrollHistoryParams;
        self.payroll_history_service
            .update(
                organization_id,
                payroll_id,
                history_id,
                UpdatePayrollHistoryParams {
                    title: None,
                    period: None,
                    start_date: None,
                    end_date: None,
                    status: None,
                    total_employees: Some(0),
                    total_earnings: Some(0.0),
                    total_deductions: Some(0.0),
                },
            )
            .await?;

        Ok(CleanResult {
            total_details_deleted: deleted_count,
        })
    }

    async fn clean_internal(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<usize> {
        // Validate history exists
        self.payroll_history_service
            .ensure_exists(organization_id, payroll_id, history_id)
            .await?;

        // Fetch all details for this history
        let details = self
            .payroll_history_detail_service
            .list(organization_id, payroll_id, history_id)
            .await?;

        let mut deleted_count = 0;
        let mut delete_errors = Vec::new();

        // Delete each detail - fail fast on errors
        for detail in details {
            match self
                .payroll_history_detail_service
                .delete(organization_id, payroll_id, history_id, detail.id)
                .await
            {
                Ok(true) => deleted_count += 1,
                Ok(false) => {} // Already deleted, continue
                Err(e) => {
                    delete_errors.push(format!("Failed to delete detail `{}`: {}", detail.id, e));
                }
            }
        }

        // If there were any delete errors, fail the operation
        if !delete_errors.is_empty() {
            return Err(AppError::internal(format!(
                "Failed to clean payroll history details: {}",
                delete_errors.join("; ")
            )));
        }

        Ok(deleted_count)
    }

    pub async fn recalculate(
        &self,
        organization_id: Uuid,
        payroll_id: Uuid,
        history_id: Uuid,
    ) -> AppResult<CalculationResult> {
        // Note: calculate() already validates draft status, but we want a clear error
        // if clean fails due to non-draft status
        self.clean(organization_id, payroll_id, history_id).await?;
        self.calculate(organization_id, payroll_id, history_id)
            .await
    }

    // Helper functions

    fn validate_period(period: &str) -> AppResult<()> {
        match period {
            "1" | "2" | "special" => Ok(()),
            _ => Err(AppError::validation(format!(
                "Invalid period `{}`. Must be one of: \"1\", \"2\", \"special\"",
                period
            ))),
        }
    }

    fn should_include_concept(history_period: &str, concept_period: &PayrollConceptPeriod) -> bool {
        matches!(
            (history_period, concept_period),
            ("1", PayrollConceptPeriod::One | PayrollConceptPeriod::Both)
                | ("2", PayrollConceptPeriod::Two | PayrollConceptPeriod::Both)
                | ("special", PayrollConceptPeriod::Special)
        )
    }

    /// Builds a mapping from original code to sanitized code, and detects collisions.
    /// Checks for collisions across ALL concepts (individual + global) and against base context variables.
    /// Returns (code_map, collisions, base_var_collisions) where:
    /// - code_map: maps original code to sanitized code
    /// - collisions: maps sanitized names to lists of original codes that collide with each other
    /// - base_var_collisions: list of (concept_code, base_var) pairs where a concept shadows a base var
    fn build_sanitized_code_map_with_base_vars(
        concepts: &[&PayrollConcept],
    ) -> (
        HashMap<String, String>,
        HashMap<String, Vec<String>>,
        Vec<(String, String)>,
    ) {
        let mut code_map: HashMap<String, String> = HashMap::new();
        let mut reverse_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut base_var_collisions: Vec<(String, String)> = Vec::new();

        // Build set of base variable names for quick lookup
        let base_vars: HashSet<&str> = BASE_CONTEXT_VARS.iter().copied().collect();

        for concept in concepts {
            let sanitized = Self::sanitize_variable_name(&concept.code);
            code_map.insert(concept.code.clone(), sanitized.clone());
            reverse_map
                .entry(sanitized.clone())
                .or_default()
                .push(concept.code.clone());

            // Check if sanitized code collides with a base context variable
            if base_vars.contains(sanitized.as_str()) {
                base_var_collisions.push((concept.code.clone(), sanitized));
            }
        }

        // Find collisions (sanitized names with more than one original concept)
        let collisions: HashMap<String, Vec<String>> = reverse_map
            .into_iter()
            .filter(|(_, originals)| originals.len() > 1)
            .collect();

        (code_map, collisions, base_var_collisions)
    }

    fn sanitize_variable_name(name: &str) -> String {
        // Replace invalid characters for evalexpr identifiers
        // Valid: alphanumeric and underscore, cannot start with digit
        let mut sanitized = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>();

        // If starts with digit, prefix with underscore
        if sanitized.chars().next().is_some_and(|c| c.is_numeric()) {
            sanitized = format!("_{}", sanitized);
        }

        sanitized
    }

    /// Extracts variable references from a formula/condition string.
    /// Returns identifiers that appear to be variable references.
    /// The `known_concept_codes` parameter ensures concept codes are recognized even if they
    /// match evalexpr keywords or context variable names.
    fn extract_variable_references(
        expression: &str,
        known_concept_codes: &HashSet<String>,
    ) -> HashSet<String> {
        let mut vars = HashSet::new();
        let mut current_var = String::new();
        let mut in_string = false;
        let mut escape_next = false;

        for ch in expression.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
                if !current_var.is_empty() && !current_var.chars().all(|c| c.is_numeric()) {
                    // Include if it's a known concept code OR not a keyword
                    if known_concept_codes.contains(&current_var)
                        || !Self::is_evalexpr_keyword(&current_var)
                    {
                        vars.insert(current_var.clone());
                    }
                    current_var.clear();
                }
                continue;
            }

            if in_string {
                continue;
            }

            if ch.is_alphanumeric() || ch == '_' {
                current_var.push(ch);
            } else if !current_var.is_empty() && !current_var.chars().all(|c| c.is_numeric()) {
                // Include if it's a known concept code OR not a keyword
                if known_concept_codes.contains(&current_var)
                    || !Self::is_evalexpr_keyword(&current_var)
                {
                    vars.insert(current_var.clone());
                }
                current_var.clear();
            }
        }

        // Don't forget the last variable
        if !current_var.is_empty()
            && !current_var.chars().all(|c| c.is_numeric())
            && (known_concept_codes.contains(&current_var)
                || !Self::is_evalexpr_keyword(&current_var))
        {
            vars.insert(current_var);
        }

        vars
    }

    /// Returns true if the string is an evalexpr language keyword or built-in function.
    /// Note: Context variables (salary, hours, etc.) are NOT keywords - they're just
    /// variables we set, so they're not filtered here.
    fn is_evalexpr_keyword(s: &str) -> bool {
        matches!(
            s,
            // Language keywords
            "true" | "false" | "if" | "else" | "and" | "or" | "not"
            // Built-in math functions
            | "min" | "max" | "len" | "floor" | "ceil" | "round"
            | "sqrt" | "abs" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan"
            | "sinh" | "cosh" | "tanh" | "asinh" | "acosh" | "atanh"
            | "ln" | "log" | "log2" | "log10" | "exp" | "exp2" | "pow"
            // String functions
            | "str" | "contains" | "starts_with" | "ends_with"
            | "regex_matches" | "regex_replace" | "to_lowercase" | "to_uppercase" | "trim"
            // Type functions
            | "typeof" | "is_string" | "is_int" | "is_float" | "is_number" | "is_boolean" | "is_tuple" | "is_empty"
            // Tuple/array functions
            | "first" | "last" | "head" | "tail" | "nth" | "take" | "skip"
            // Random
            | "random"
        )
    }

    /// Performs topological sort on global concepts based on their formula dependencies.
    /// Returns sorted list or an error message if a cycle (including self-reference) is detected.
    fn topological_sort_globals(
        globals: Vec<GlobalConceptWithDefinition>,
        _all_available_vars: &HashSet<String>,
    ) -> Result<Vec<GlobalConceptWithDefinition>, String> {
        // Build dependency graph
        let mut dependencies: HashMap<String, HashSet<String>> = HashMap::new();
        let global_codes: HashSet<String> =
            globals.iter().map(|g| g.sanitized_code.clone()).collect();

        // Track self-referencing concepts for better error messages
        let mut self_referencing: Vec<String> = Vec::new();

        for global in &globals {
            let formula_vars =
                Self::extract_variable_references(&global.definition.formula, &global_codes);
            let condition_vars =
                Self::extract_variable_references(&global.definition.condition, &global_codes);

            let all_vars: HashSet<String> = formula_vars.union(&condition_vars).cloned().collect();

            // Check for self-reference
            if all_vars.contains(&global.sanitized_code) {
                self_referencing.push(global.concept_code.clone());
            }

            // Only keep dependencies that are other global concepts (excluding self)
            let global_deps: HashSet<String> = all_vars
                .into_iter()
                .filter(|v| global_codes.contains(v) && v != &global.sanitized_code)
                .collect();

            dependencies.insert(global.sanitized_code.clone(), global_deps);
        }

        // Fail fast if any concept references itself
        if !self_referencing.is_empty() {
            return Err(format!(
                "Self-referencing global concept(s) detected: {}. A concept cannot reference itself in its formula or condition.",
                self_referencing.join(", ")
            ));
        }

        // Kahn's algorithm for topological sort
        // in_degree tracks how many DEPENDENCIES a concept HAS (not how many depend on it)
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for (code, deps) in &dependencies {
            in_degree.insert(code.clone(), deps.len());
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(code, _)| code.clone())
            .collect();

        // Sort queue for deterministic behavior
        queue.sort();

        let mut sorted_codes: Vec<String> = Vec::new();

        while let Some(code) = queue.pop() {
            sorted_codes.push(code.clone());

            // Find concepts that depend on this one and reduce their in-degree
            for (other_code, deps) in &dependencies {
                if deps.contains(&code) {
                    if let Some(deg) = in_degree.get_mut(other_code) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(other_code.clone());
                            queue.sort(); // Keep deterministic
                        }
                    }
                }
            }
        }

        if sorted_codes.len() != global_codes.len() {
            // Find the concepts involved in the cycle for better error message
            let in_cycle: Vec<String> = global_codes
                .iter()
                .filter(|code| !sorted_codes.contains(code))
                .cloned()
                .collect();
            return Err(format!(
                "Circular dependency detected among global concepts: {}. These concepts form a dependency cycle and cannot be evaluated.",
                in_cycle.join(", ")
            ));
        }

        // Rebuild the sorted list by consuming globals
        let mut code_to_global: HashMap<String, GlobalConceptWithDefinition> = globals
            .into_iter()
            .map(|g| (g.sanitized_code.clone(), g))
            .collect();

        let sorted: Vec<GlobalConceptWithDefinition> = sorted_codes
            .iter()
            .filter_map(|code| code_to_global.remove(code))
            .collect();

        Ok(sorted)
    }

    fn build_eval_context(
        employee: &Employee,
        job: &Job,
        calculated_concepts: &HashMap<String, f64>,
    ) -> Result<HashMapContext, String> {
        let mut context = HashMapContext::new();

        // Employee fields - handle errors instead of unwrap
        context
            .set_value("salary".to_string(), employee.salary.into())
            .map_err(|e| format!("Failed to set salary: {}", e))?;
        context
            .set_value("hours".to_string(), (employee.hours as f64).into())
            .map_err(|e| format!("Failed to set hours: {}", e))?;
        context
            .set_value(
                "classification".to_string(),
                employee.clasification.clone().into(),
            )
            .map_err(|e| format!("Failed to set classification: {}", e))?;

        // Job fields
        context
            .set_value("job_salary".to_string(), job.salary.into())
            .map_err(|e| format!("Failed to set job_salary: {}", e))?;
        context
            .set_value("job_title".to_string(), job.job_title.clone().into())
            .map_err(|e| format!("Failed to set job_title: {}", e))?;

        // Previously calculated concept amounts (use sanitized concept code as variable name)
        for (concept_code, amount) in calculated_concepts {
            context
                .set_value(concept_code.clone(), (*amount).into())
                .map_err(|e| format!("Failed to set {}: {}", concept_code, e))?;
        }

        Ok(context)
    }

    fn evaluate_formula(formula: &str, context: &HashMapContext) -> Result<f64, String> {
        let result = evalexpr::eval_number_with_context(formula, context)
            .map_err(|e| format!("Formula evaluation error: {}", e))?;

        // Validate the result is a finite number (not NaN or infinity)
        if !result.is_finite() {
            return Err(format!(
                "Formula produced non-finite result: {}. Check for division by zero or overflow.",
                if result.is_nan() { "NaN" } else { "infinity" }
            ));
        }

        Ok(result)
    }

    fn evaluate_condition(condition: &str, context: &HashMapContext) -> Result<bool, String> {
        evalexpr::eval_boolean_with_context(condition, context)
            .map_err(|e| format!("Condition evaluation error: {}", e))
    }

    /// Pre-parse validation for formula syntax.
    /// Uses evalexpr's build_operator_tree to check syntax without evaluating.
    fn validate_formula_syntax(formula: &str) -> Result<(), String> {
        evalexpr::build_operator_tree(formula)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Pre-parse validation for condition syntax.
    /// Uses evalexpr's build_operator_tree to check syntax without evaluating.
    fn validate_condition_syntax(condition: &str) -> Result<(), String> {
        evalexpr::build_operator_tree(condition)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Normalizes a condition string. Empty or whitespace-only conditions are
    /// treated as "always true" and normalized to "true".
    fn normalize_condition(condition: &str) -> String {
        let trimmed = condition.trim();
        if trimmed.is_empty() {
            "true".to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Checks if a formula or condition contains potentially dangerous functions
    /// like regex (ReDoS risk) or random (nondeterministic results).
    /// Returns a list of disallowed function names found.
    ///
    /// This uses proper token boundary detection to avoid false positives:
    /// - `random_bonus` does NOT match `random`
    /// - `my_regex_matches` does NOT match `regex_matches`
    /// - Only matches when the function name appears as a standalone identifier followed by `(`
    fn check_disallowed_functions(expression: &str) -> Vec<&'static str> {
        const DISALLOWED_FUNCTIONS: &[&str] = &[
            "regex_matches",
            "regex_replace",
            "random",
        ];

        let mut found = Vec::new();
        for func in DISALLOWED_FUNCTIONS {
            if Self::contains_function_call(expression, func) {
                found.push(*func);
            }
        }
        found
    }

    /// Checks if an expression contains a function call with the given name.
    /// Ensures proper token boundaries (not part of a larger identifier).
    fn contains_function_call(expression: &str, func_name: &str) -> bool {
        let chars: Vec<char> = expression.chars().collect();
        let func_chars: Vec<char> = func_name.chars().collect();
        let func_len = func_chars.len();

        if chars.len() < func_len {
            return false;
        }

        let mut i = 0;
        while i <= chars.len() - func_len {
            // Check if we're at a potential match
            let slice_matches = chars[i..i + func_len]
                .iter()
                .zip(func_chars.iter())
                .all(|(a, b)| a == b);

            if slice_matches {
                // Check token boundary before (must not be alphanumeric or underscore)
                let valid_start = i == 0 || {
                    let prev = chars[i - 1];
                    !prev.is_alphanumeric() && prev != '_'
                };

                // Check token boundary after (must be followed by optional whitespace then '(')
                let valid_end = if i + func_len < chars.len() {
                    let after = chars[i + func_len];
                    // Must not be part of a larger identifier
                    if after.is_alphanumeric() || after == '_' {
                        false
                    } else {
                        // Check for '(' possibly after whitespace
                        let rest: String = chars[i + func_len..].iter().collect();
                        let trimmed = rest.trim_start();
                        trimmed.starts_with('(')
                    }
                } else {
                    false // Function name at end without '(' is not a call
                };

                if valid_start && valid_end {
                    return true;
                }
            }

            i += 1;
        }

        false
    }
}
