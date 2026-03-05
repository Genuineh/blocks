---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-04
updated: 2026-03-04
version: 1.0
related_prds:
  - docs/prds/GREETING_PROOF_SLICE_PLAN.md
---

# Greeting Proof Slice Specification

## Overview

The greeting proof slice adds one HTTP backend service and one Tauri-plus-web frontend panel. The backend owns the API contract, and the frontend consumes that contract through an explicit `moc` dependency and matching protocol declaration.

## Goals

- Prove the current `moc` model can express a real frontend/backend slice
- Keep the runtime minimal and inspectable
- Keep verification honest by separating automated contract checks from manual UI confirmation

## Non-Goals

- Full browser automation across the live backend and frontend
- Production-grade routing, persistence, or deployment concerns

## Architecture

### Components

- `mocs/greeting-api-service`: `backend_app(service)` with a minimal `std::net::TcpListener` server
- `mocs/greeting-panel-web`: `frontend_app` using shared JS state logic, static preview assets, and a Tauri host shell
- `scripts/repo_check.sh`: bounded checks for workspace tests, frontend state logic, and host readiness

### Data Flow

1. The backend listens on `127.0.0.1`, defaulting to port `4318`
2. The frontend issues a simple `GET` request to `/api/v1/greeting`
3. The backend returns JSON with required `title` and `message`
4. The frontend renders loading, then success or error

## API Specification

### GET /api/v1/greeting

- **Input**: no request body, simple `GET`
- **Output**:
  - `title: string`
  - `message: string`
- **Errors**:
  - `404` for unknown paths
  - `405` for unsupported methods

## Data Models

### GreetingResponse

| Field | Type | Description |
|-------|------|-------------|
| title | string | Heading rendered by the frontend |
| message | string | Supporting body copy rendered by the frontend |

## Technical Decisions

| Decision | Choice | Rationale |
|---------|--------|-----------|
| Backend HTTP stack | `std::net::TcpListener` | Keeps the proof slice small and dependency-light |
| Browser compatibility | `Access-Control-Allow-Origin: *` | Allows a static preview origin to call the backend without custom headers |
| Frontend tests | `node --test` over shared JS module | Covers state logic without introducing a browser harness |
| End-to-end proof | Manual render step | Current repository should not claim full automated fetch-plus-render coverage |

## Testing Strategy

- Workspace `cargo test` covers the backend API contract on ephemeral ports
- `node --test mocs/greeting-panel-web/tests/greeting_panel.test.mjs` covers loading, success, and error state logic
- `cargo --offline run --manifest-path mocs/greeting-panel-web/src-tauri/Cargo.toml -- --headless-probe` covers frontend host readiness
- Manual browser or Tauri rendering remains required for the live fetch-plus-render join

---

### Change Log
- 2026-03-04: Added the implemented technical spec for the greeting proof slice and documented the verification boundary.
