# Naming Conventions

This document defines naming standards for all TeachLink modules.

## Goals

- Keep code readable across Rust and TypeScript modules.
- Reduce cognitive overhead when switching between contract and indexer code.
- Enforce conventions automatically in CI and local development.

## Rust (contracts, testing, benches)

- Functions and methods: `snake_case` (example: `validate_bridge_out`).
- Variables and parameters: `snake_case` (example: `destination_chain`).
- Structs, enums, traits, type aliases: `PascalCase` (example: `EscrowValidator`).
- Constants and statics: `UPPER_SNAKE_CASE` (example: `MAX_ESCROW_DESCRIPTION_LENGTH`).
- Modules and file names: `snake_case` (example: `property_based_tests.rs`).

Enforcement:

- `cargo fmt`
- `cargo clippy --all-targets --all-features`
- Workspace lint: `nonstandard_style = "deny"` in `Cargo.toml`

## TypeScript (indexer and recommendation modules)

- Classes, interfaces, types, enums: `PascalCase` (example: `IndexerService`).
- Functions, methods, local variables, parameters: `camelCase` (example: `startIndexing`).
- Runtime constants (`const` values intended as constants): `UPPER_SNAKE_CASE` or `camelCase`.
- File names: `kebab-case` (example: `event-processor.service.ts`).
- Keep external payload fields in source format when required by on-chain/API schemas
  (example: `event.data.backup_id`).

Enforcement:

- `npm run lint` (ESLint with `@typescript-eslint/naming-convention`)
- `npm run format`

## External Data and Storage Naming

- Database table and column names can remain `snake_case` when mapped through entities.
- Contract event payload fields should preserve emitted schema names.
- Internal mapped DTO/entity fields should use project conventions (`camelCase` in TypeScript).

## Change Management for Renames

- Prefer small, behavior-preserving renames.
- Rename references atomically in the same change.
- Update related tests, docs, and examples in the same PR.
