---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-04
updated: 2026-03-04
version: 1.0
---

# Greeting Proof Slice Plan

## Summary

Add one backend `moc` and one frontend `moc` that work together as the smallest real full-stack proof slice under the current `moc` model.

## Problem

The repository already proves separate backend and frontend `moc` paths, but it still needs a concrete slice where a frontend fetches live backend data and renders it.

## Users

- contributors validating the current `moc` model
- future agent workflows that need a minimal, real frontend/backend example

## Requirements

### Must Have

- `greeting-api-service` as `backend_app(service)` exposing `GET /api/v1/greeting`
- `greeting-panel-web` as `frontend_app` with `language: tauri_ts`
- shared `greeting-http` protocol declarations on both `moc.yaml` files
- frontend `depends_on_mocs` referencing the backend `moc`
- honest verification boundaries separating automated checks from manual full render validation

### Should Have

- bounded repository checks for backend contract coverage and frontend host readiness
- a default manual demo port with an override path for tests and local runs

## User Stories

- As a contributor, I want a tiny backend service and frontend panel that talk over HTTP so that the `moc` model proves a real multi-`moc` product slice.
- As a reviewer, I want automated checks that cover the backend contract and frontend state logic so that the proof does not depend on hand-waving.

## Acceptance Criteria

- [x] The backend returns `title` and `message` JSON fields from `GET /api/v1/greeting`
- [x] The frontend renders loading, success, and error states
- [x] The frontend declares and uses the backend protocol contract
- [x] Documentation states exactly which checks are automated and which verification remains manual

---

### Change Log
- 2026-03-04: Added the approved plan for the greeting proof slice and aligned it to the two-`moc` implementation.
