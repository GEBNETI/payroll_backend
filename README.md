# Nómina API

Axum-based REST API for managing organizations, payrolls, and hierarchical divisions, backed by SurrealDB. This repo demonstrates a layered architecture (domain ➝ services ➝ handlers ➝ routes) with strong validation and comprehensive integration tests.

## Features

- Health check endpoint for service metadata.
- CRUD over organizations.
- Payroll management tied to organizations.
- Division management tied to payrolls with optional parent–child relationships.
- Job management tied to payrolls with salary tracking.
- Bank management tied to organizations, used by employees for deposit info.
- Employee management scoped to a division and payroll with job/bank validation.
- Payroll concept catalog for earnings/deductions, including scope tracking.
- Global concept definitions that capture the formula and optional condition for each global concept.
- Employee-specific concept assignments with per-assignment amounts for individual concepts.
- Payroll history tracking for payroll run status and detailed calculations per employee.
- SurrealDB repository implementations plus in-memory doubles for integration tests.

## HTTP Endpoints

| Method | Path                | Description |
|--------|--------------------|-------------|
| GET    | `/health`          | Service metadata probe |
| POST   | `/organizations`   | Create organization |
| GET    | `/organizations`   | List organizations |
| GET    | `/organizations/:id` | Fetch organization |
| PUT    | `/organizations/:id` | Update organization name |
| DELETE | `/organizations/:id` | Delete organization |
| POST   | `/organizations/:organization_id/payrolls` | Create payroll within an organization |
| GET    | `/organizations/:organization_id/payrolls` | List payrolls for an organization |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id` | Fetch payroll |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id` | Update payroll fields |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id` | Delete payroll |
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/jobs` | Create job |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/jobs` | List jobs for a payroll |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/jobs/:job_id` | Fetch job |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id/jobs/:job_id` | Update job title or salary |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/jobs/:job_id` | Delete job |
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/concepts` | Create payroll concept (`code`, `type`, `scope`) |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/concepts` | List payroll concepts for a payroll |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/concepts/:concept_id` | Fetch payroll concept |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id/concepts/:concept_id` | Update payroll concept fields |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/concepts/:concept_id` | Delete payroll concept |
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/concepts/:concept_id/definition` | Create global concept definition (formula + condition) |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/concepts/:concept_id/definition` | Fetch concept definition |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id/concepts/:concept_id/definition` | Update concept definition |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/concepts/:concept_id/definition` | Delete concept definition |
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/divisions` | Create division (optional `parent_division_id`) |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/divisions` | List divisions for a payroll |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id` | Fetch division |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id` | Update division fields / parent |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id` | Delete division |
| POST   | `/organizations/:organization_id/banks` | Create bank within an organization |
| GET    | `/organizations/:organization_id/banks` | List banks for an organization |
| GET    | `/organizations/:organization_id/banks/:bank_id` | Fetch bank |
| PUT    | `/organizations/:organization_id/banks/:bank_id` | Update bank name |
| DELETE | `/organizations/:organization_id/banks/:bank_id` | Delete bank |
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees` | Create employee inside a division |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees` | List employees for a division |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees/:employee_id` | Fetch employee |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees/:employee_id` | Update employee |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees/:employee_id` | Delete employee |
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees/:employee_id/concepts` | Assign an individual payroll concept to an employee with an amount |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees/:employee_id/concepts` | List concept assignments for an employee |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees/:employee_id/concepts/:assignment_id` | Fetch employee concept assignment |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees/:employee_id/concepts/:assignment_id` | Update assignment amount |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id/employees/:employee_id/concepts/:assignment_id` | Remove assignment |
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/history` | Create payroll history entry |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/history` | List payroll history for a payroll |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/history/:history_id` | Fetch payroll history entry |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id/history/:history_id` | Update payroll history entry |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/history/:history_id` | Delete payroll history entry |
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/history/:history_id/details` | Create payroll history detail |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/history/:history_id/details` | List details for a payroll history entry |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/history/:history_id/details/:detail_id` | Fetch payroll history detail |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/history/:history_id/details/:detail_id` | Delete payroll history detail |

## API Documentation

- OpenAPI document: `GET /api-docs/openapi.json`
- Interactive Swagger UI: visit `http://localhost:3000/swagger-ui` after running `cargo run` with the required SurrealDB environment variables configured.

The documentation stays in sync with the handlers using `utoipa`, so request/response schemas and parameters are always up to date.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SURREALDB_URL` | SurrealDB endpoint (e.g. `https://...` or `ws://...`) |
| `SURREALDB_NAMESPACE` | Namespace to use |
| `SURREALDB_DATABASE` | Database name |
| `SURREALDB_USERNAME` | Auth user |
| `SURREALDB_PASSWORD` | Auth password |
| `PORT` (optional) | HTTP port (defaults to `3000`) |

The server fails fast if any of these are missing or invalid.

## Development

```bash
# Lint & fmt
cargo fmt
cargo clippy --all-targets --all-features

# Tests (integration + unit)
cargo test

# Run locally
SURREALDB_URL=... \
SURREALDB_NAMESPACE=... \
SURREALDB_DATABASE=... \
SURREALDB_USERNAME=... \
SURREALDB_PASSWORD=... \
cargo run
```

## Testing Strategy

- Unit tests live next to code (see `domain`, `services`).
- Integration tests under `tests/` use the in-memory repositories defined in `tests/support/` so they run without external dependencies.
- Each new feature should ship with at least one happy path and one edge case test.

## Project Structure

```
src/
  domain/         # Pure business models
  services/       # Validation & orchestration
  infrastructure/ # SurrealDB adapters and configuration
  handlers/       # Axum handlers (HTTP layer)
  routes/         # Routers per feature
  server.rs       # App state + router bootstrap
```

Feel free to extend modules in the same pattern (domain ➝ service ➝ handler ➝ route) for new resources.
