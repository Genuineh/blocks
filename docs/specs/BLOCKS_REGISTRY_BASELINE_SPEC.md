---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-16
updated: 2026-03-16
version: 1.0
---

# Blocks Registry Baseline Specification

## Overview

This specification defines the Phase 2 registry baseline for package resolution.

The registry layer is provider-driven and currently supports:

- workspace provider
- file registry provider
- remote-ready interface seam

## Provider Precedence

Resolver input includes an ordered provider list.

Resolution rules:

1. evaluate providers in declared order
2. the first provider with compatible candidates becomes the selected provider
3. inside that provider, select the highest compatible version
4. if a tie remains, select lexicographically smallest `source.location`

## Conflict Handling

If the same `(id, kind, version)` exists in multiple providers with different normalized manifest content, resolution fails with `pkg.resolve.conflicting_release`.

## File Registry Baseline

Phase 2 file registry layout is local-filesystem only.

Canonical release root:

- `<registry-root>/<package-id with dots replaced by "__">/<version>/`

Required release artifacts:

- `package.yaml`
- descriptor file referenced by `descriptor.path`

Resolver notes:

- workspace providers are evaluated before later providers when they return compatible candidates
- if a provider returns no compatible candidate, resolution falls through to the next provider
- providers that do not discover a real release return no candidate; resolver does not fabricate releases from version requirements or path naming conventions
- the legacy `dep.sample` compatibility shim is default-off and only re-enabled through explicit `blocks pkg resolve --compat`
- `pkg fetch` selects the highest compatible file-registry release, not just `0.1.0`

## Fetch Error Taxonomy

Phase 2 fetch errors are limited to:

- `pkg.fetch.not_found`
- `pkg.fetch.source_unavailable`
- `pkg.fetch.unsupported_source`
- `pkg.fetch.cache_write_failed`

Checksum mismatch semantics are explicitly out of scope for Phase 2.
