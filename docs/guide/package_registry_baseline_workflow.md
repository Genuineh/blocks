---
status: active
last_verified_commit: N/A
owner: Developer
created: 2026-03-16
updated: 2026-03-17
version: 1.0
---

# Package Registry Baseline Workflow

## Goal

Use the Phase 2 `blocks pkg` baseline to scaffold, resolve, publish, and fetch package metadata.

## Commands

Create a package baseline:

```bash
cargo run -p blocks-cli -- pkg init packages --kind block --id demo.phase2 --json
```

Resolve and write a deterministic lockfile:

```bash
cargo run -p blocks-cli -- pkg resolve packages/demo-phase2 --lock --json
```

Resolve in compatibility mode when a manifest still carries unknown top-level keys:

```bash
cargo run -p blocks-cli -- pkg resolve packages/demo-phase2 --compat --json
```

Publish to a local file registry:

```bash
cargo run -p blocks-cli -- pkg publish packages/demo-phase2 --to .tmp/file-registry --json
```

Fetch from a local file registry:

```bash
cargo run -p blocks-cli -- pkg fetch demo.phase2 --provider file:.tmp/file-registry --json
```

Run the package-resolution conformance suite:

```bash
cargo run -p blocks-cli -- conformance run package \
  packages/demo-phase2 \
  --provider file:.tmp/file-registry \
  --json
```

## Third-Party Adopter Sample

The package baseline is not tied to this repository's `blocks/` layout. A third-party adopter can validate an external dependency graph with only a package root and a file registry:

```text
third-party-adopter/
  packages/
    consumer-portal/
      package.yaml
      block.yaml
  file-registry/
    dep__shared/
      0.2.4/
        package.yaml
        block.yaml
```

Example `consumer-portal/package.yaml`:

```yaml
api_version: blocks.pkg/v1
kind: block
id: consumer.portal
version: 0.1.0
descriptor:
  path: block.yaml
dependencies:
  - id: dep.shared
    kind: block
    req: ^0.2.0
```

Validate that adopter workspace with the public conformance surface:

```bash
cargo run -p blocks-cli -- conformance run package \
  third-party-adopter/packages/consumer-portal \
  --provider file:third-party-adopter/file-registry \
  --json
```

This verifies three Phase 2 invariants in one command:

- dependency resolution succeeds against a provider outside the current workspace layout
- `blocks.lock` is written for the consumer package
- repeated resolution is byte-stable for both JSON output and lockfile contents

## Notes

- `package.yaml` owns package identity and dependencies
- descriptor files still own descriptor semantics
- compatibility mode downgrades unknown top-level manifest keys to warnings during resolve
- file-registry fetch selects the highest available compatible release
- `conformance run package` is the public package-resolution verification surface for CI and third-party adopters
- checksum and remote protocol behavior are not part of Phase 2
