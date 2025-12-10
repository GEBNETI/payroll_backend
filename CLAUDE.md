# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

```bash
cargo check              # Fast validation before committing
cargo fmt                # Format code (must pass before push)
cargo clippy --all-targets --all-features  # Lint (treat warnings as errors)
cargo test               # Run all unit and integration tests
cargo test -- --nocapture  # Run tests with stdout visible
cargo test <test_name>   # Run a single test by name
cargo run                # Run locally (requires SurrealDB env vars)
cargo build --release    # Optimized build for deployment
```

## Environment Variables

Required for running the server:
- `SURREALDB_URL` - SurrealDB endpoint
- `SURREALDB_NAMESPACE` - Namespace to use
- `SURREALDB_DATABASE` - Database name
- `SURREALDB_USERNAME` - Auth user
- `SURREALDB_PASSWORD` - Auth password
- `PORT` (optional) - HTTP port, defaults to 3000

## Architecture

This is an Axum-based REST API backed by SurrealDB following a layered architecture:

```
domain → services → handlers → routes
```

### Layer Responsibilities

- **domain/** - Pure business types with no framework dependencies. Contains: Organization, Payroll, Division, Job, Bank, Employee, PayrollConcept, PayrollConceptDefinition, EmployeePayrollConcept
- **services/** - Business logic, validation, and orchestration. Each service owns a repository trait and validates cross-entity invariants
- **handlers/** - HTTP request/response logic, converts domain data to transport payloads
- **routes/** - Composable routers per feature that wire HTTP paths
- **infrastructure/** - SurrealDB repository implementations
- **server.rs** - `AppState` wires all services together; `router()` builds the Axum router

### Key Patterns

- Repository traits defined in services, implemented in infrastructure (dependency inversion)
- Services receive dependent services via Arc for cross-entity validation (e.g., EmployeeService validates job/bank existence)
- AppState holds Arc-wrapped services for shared access across handlers

## Testing

- Unit tests use `#[cfg(test)]` modules alongside production code
- Integration tests in `tests/` use in-memory repositories from `tests/support/` (no external DB needed)
- `tests/support/in_memory_repository.rs` provides test doubles for all repository traits

## Commit Style

Use Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`

