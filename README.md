# Nómina API

Axum-based REST API for managing organizations, payrolls, and hierarchical divisions, backed by SurrealDB. This repo demonstrates a layered architecture (domain ➝ services ➝ handlers ➝ routes) with strong validation and comprehensive integration tests.

## Features

- Health check endpoint for service metadata.
- CRUD over organizations.
- Payroll management tied to organizations.
- Division management tied to payrolls with optional parent–child relationships.
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
| POST   | `/payrolls`        | Create payroll (`organization_id` required) |
| GET    | `/payrolls`        | List payrolls |
| GET    | `/payrolls/:id`    | Fetch payroll |
| PUT    | `/payrolls/:id`    | Update payroll fields |
| DELETE | `/payrolls/:id`    | Delete payroll |
| POST   | `/divisions`       | Create division (`payroll_id`, optional `parent_division_id`) |
| GET    | `/divisions`       | List divisions |
| GET    | `/divisions/:id`   | Fetch division |
| PUT    | `/divisions/:id`   | Update division fields / parent |
| DELETE | `/divisions/:id`   | Delete division |

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
