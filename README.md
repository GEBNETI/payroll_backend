# Nómina API

Axum-based REST API for managing organizations, payrolls, and hierarchical divisions, backed by SurrealDB. This repo demonstrates a layered architecture (domain ➝ services ➝ handlers ➝ routes) with strong validation and comprehensive integration tests.

## Features

- Health check endpoint for service metadata.
- CRUD over organizations.
- Payroll management tied to organizations.
- Division management tied to payrolls with optional parent–child relationships.
- Job management tied to payrolls with salary tracking.
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
| POST   | `/organizations/:organization_id/payrolls/:payroll_id/divisions` | Create division (optional `parent_division_id`) |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/divisions` | List divisions for a payroll |
| GET    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id` | Fetch division |
| PUT    | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id` | Update division fields / parent |
| DELETE | `/organizations/:organization_id/payrolls/:payroll_id/divisions/:division_id` | Delete division |

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
