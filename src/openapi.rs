use utoipa::OpenApi;

/// Aggregated OpenAPI document for the service.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::check,
        crate::handlers::organization::create,
        crate::handlers::organization::list,
        crate::handlers::organization::get,
        crate::handlers::organization::update,
        crate::handlers::organization::delete,
        crate::handlers::payroll::create,
        crate::handlers::payroll::list,
        crate::handlers::payroll::get,
        crate::handlers::payroll::update,
        crate::handlers::payroll::delete,
        crate::handlers::division::create,
        crate::handlers::division::list,
        crate::handlers::division::get,
        crate::handlers::division::update,
        crate::handlers::division::delete,
    ),
    components(
        schemas(
            crate::domain::health::HealthSnapshot,
            crate::domain::organization::Organization,
            crate::domain::payroll::Payroll,
            crate::domain::division::Division,
            crate::handlers::organization::CreateOrganizationRequest,
            crate::handlers::organization::UpdateOrganizationRequest,
            crate::handlers::organization::OrganizationResponse,
            crate::handlers::payroll::CreatePayrollRequest,
            crate::handlers::payroll::UpdatePayrollRequest,
            crate::handlers::payroll::PayrollResponse,
            crate::handlers::division::CreateDivisionRequest,
            crate::handlers::division::UpdateDivisionRequest,
            crate::handlers::division::DivisionResponse,
        )
    ),
    tags(
        (name = "Health", description = "Service health endpoints"),
        (name = "Organizations", description = "Organization management"),
        (name = "Payrolls", description = "Payroll management"),
        (name = "Divisions", description = "Division management"),
    )
)]
pub struct ApiDoc;
