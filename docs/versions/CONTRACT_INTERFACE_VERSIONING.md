# Contract Interface Versioning Strategy

This document defines the TeachLink contract interface versioning system for compatibility management.

## Scope

This policy covers public contract entry points exposed by `TeachLinkBridge` and consumed by off-chain clients, SDKs, and indexers.

## Semantic Versioning Model

TeachLink uses semantic versioning (`MAJOR.MINOR.PATCH`) for interface compatibility:

- `MAJOR`: breaking interface changes.
- `MINOR`: backward-compatible interface additions.
- `PATCH`: backward-compatible fixes and clarifications.

Contract version values are represented on-chain as:

- `current`: the latest contract interface version.
- `minimum_compatible`: the oldest client interface version supported by the deployed contract.

## Compatibility Rules

A client version is compatible only when all conditions hold:

1. `client.major == current.major`
2. `client >= minimum_compatible`
3. `client <= current`

If any condition fails, the contract reports incompatibility.

## On-Chain Interface Methods

The contract exposes explicit interface version controls:

- `get_interface_version_status()`
- `get_interface_version()`
- `get_min_compat_interface_version()`
- `is_interface_compatible(client_version)`
- `assert_interface_compatible(client_version)`
- `set_interface_version(current, minimum_compatible)` (admin only)

## Upgrade Policy

When deploying interface changes:

1. Decide version bump type (`MAJOR`, `MINOR`, or `PATCH`).
2. Set `current` to the new semantic version.
3. Set `minimum_compatible` to the oldest client version guaranteed to work.
4. Keep `minimum_compatible.major == current.major`.
5. Ensure `minimum_compatible <= current`.
6. Run compatibility tests before release.

## Backward Compatibility Guarantees

- Minor and patch releases must not break clients between `minimum_compatible` and `current`.
- Breaking changes require a major bump and corresponding client migration guidance.
- Contracts must reject impossible compatibility windows (for example, mismatched major versions).

## Operational Guidance

- SDKs should query `get_interface_version_status()` at startup.
- Clients should fail fast when outside the compatible window.
- Indexers should track interface versions per deployment and chain.

## Testing Requirements

Versioning tests must cover:

- Default version state after initialization.
- Positive compatibility at lower/upper boundary versions.
- Rejection of below-minimum versions.
- Rejection of higher-than-current versions.
- Rejection of different major versions.
- Rejection of invalid admin updates (`minimum_compatible > current` or major mismatch).
