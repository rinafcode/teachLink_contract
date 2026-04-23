# Disaster Recovery Procedures

This document defines complete recovery scenarios, data restoration testing, service restoration runbooks, and repeatable testing procedures for TeachLink.

## Purpose and Scope

- Purpose: Ensure timely, verifiable recovery from incidents affecting data, indexers, off-chain artifacts, or full environment failures.
- Scope: smart contract state snapshots and manifests, indexer databases, off-chain artifacts referenced by integrity hashes, deployment infrastructure (indexers, API services, observability), critical third-party integrations.

## Roles & Responsibilities

- On-call Recovery Lead: coordinates recovery and communications.
- Infrastructure Engineer: restores infrastructure and storage.
- Indexer Operator: restores indexer DBs, replays events.
- Application Owner: runs smoke tests and validates functionality.
- Compliance/Audit: collects evidence artifacts and signs off.

## Recovery Objectives

- Recovery Time Objective (RTO): target by component (e.g., indexer DB 2 hours, API service 4 hours, full environment 8 hours).
- Recovery Point Objective (RPO): target snapshot age (e.g., off-chain artifacts hourly, indexer WAL-based replay to last confirmed block).

## Recovery Scenarios (detailed)

1) Data Corruption (single-table / artifact)
	- Detection: alert from integrity-check or failed verification.
	- Immediate action: isolate affected service, promote read-only fallback if available.
	- Restore: identify latest good manifest, restore artifact from `backups/artifacts/<manifest>`, verify integrity hash.
	- Validation: run `data_integrity_verify` and application smoke tests.
	- Post-recovery: replay missing events if required; record incident and corrective actions.

2) Partial Data Loss (indexer shards or partial contract state)
	- Detection: missing indexer metrics, inconsistent query results.
	- Restore: restore indexer DB from latest full backup; replay WAL or event stream from last backup point to current.
	- Validation: run indexer reconciliation job and compare counts with golden manifest.

3) Full Environment Loss (region or cluster outage)
	- Actions:
	  - Failover to secondary region (if configured) or provision new cluster following the `infrastructure/runbooks/provision_cluster.md` steps.
	  - Restore storage volumes from backups and attach to instances.
	  - Redeploy indexers, APIs, and workers using the tagged release used at backup time.
	- Validation: run end-to-end smoke tests and synthetic transactions.

4) Key/Secrets Compromise
	- Actions: rotate compromised secrets, revoke affected credentials, update manifests referencing secrets, redeploy services with new secrets.
	- Validation: verify unauthorized access stops and rotate verification keys where applicable.

5) Third-Party Service Outage (e.g., cloud storage)
	- Actions: switch to configured secondary provider or restore artifacts from alternative replication target.
	- Validation: confirm read/write operations against the failover provider.

## Test Data Restoration Procedures

- Pre-reqs: isolated test environment, service account with restore privileges, sample backup manifest id, and a verification key.

Step-by-step restore (example):

1. Provision an isolated environment (use VM/container image `teachlink/dr-test`).
2. Fetch backup manifest: `aws s3 cp s3://teachlink-backups/manifests/<manifest>.json ./manifest.json` (or equivalent provider command).
3. Validate manifest integrity: compare stored `integrity_hash` with `sha256sum` of artifacts.
4. Restore artifacts to test storage: `restore_tool --manifest ./manifest.json --target ./restore`.
5. Restore indexer DB (if included): stop indexer service, load DB snapshot, start indexer, run `indexer_replay --from <manifest_block>`.
6. Run automated validation suite: `scripts/recovery_test.sh` (Linux/macOS) or `scripts/recovery_test.ps1` (Windows).
7. Record outcome: capture `RecoveryExecutedEvent` if run on-chain or save `dr_report.json` in `backups/recovery_reports/`.

Verification checks:
- Hash match for each restored artifact.
- Application smoke tests pass: health endpoints, a sample read, and sample write (if safe).
- Indexer reconciliation: counts within tolerance vs golden manifest.

Roll-back plan: if validation fails, revert test environment, record failure with logs, and iterate on restore steps.

## Service Restoration Plan (runbook)

1. Triage & Communication
	- Notify stakeholders and escalate via on-call rota.
	- Create incident ticket with severity, target RTO/RPO, and assigned roles.

2. Stabilize & Isolate
	- Disable incoming traffic to affected services via load balancer/DNS.
	- Ensure monitoring continues to capture metrics and logs.

3. Restore Persistence Layer
	- Restore object store from backups.
	- Restore databases (indexer DBs) from snapshots and replay event streams.

4. Restore Core Services in Order
	- Indexer services (bring online first so downstream APIs can serve data).
	- API/backend services.
	- Worker/background jobs.
	- Frontend and public endpoints.

5. Validate
	- Execute smoke test suite and synthetic transactions.
	- Run integrity verification and reconcile indexer counts.

6. Scale & Harden
	- Scale services to target capacity.
	- Apply any hotfixes and mitigations identified during recovery.

7. Close Incident
	- Document timeline, RTO/RPO achieved, root cause analysis, and follow-ups.

## Testing Procedures and Drill Schedule

- Drill types and cadence:
  - Backup verification: weekly automated checks.
  - Restoration drill (isolated): monthly.
  - Full DR scenario (cross-team): quarterly.
  - Tabletop exercises (process review): semi-annually.

- Drill execution checklist:
  1. Announce drill window and non-production environment targets.
  2. Run `scripts/recovery_test.sh` or `scripts/recovery_test.ps1`.
  3. Validate results and collect `dr_report.json` and logs.
  4. Post-drill review and action items.

## Automation and Scripts

See `scripts/recovery_test.sh` and `scripts/recovery_test.ps1` for a small, repeatable validation harness that:
- verifies artifact integrity,
- checks indexer reconciliation endpoints,
- runs smoke tests against restored environment,
- emits a `dr_report.json` with pass/fail and timing metrics.

## Evidence & Audit

- Store drill reports in `backups/recovery_reports/<YYYY-MM-DD>-<drill-id>.json`.
- Attach relevant logs, verification traces, and artifact manifests.

## Metrics to Capture

- Recovery duration per component (seconds)
- Success/failure boolean
- Data integrity pass rate
- Number of manual interventions required

## Post-Incident Review

- Perform RCA within 72 hours, publish action items, and track remediation in the incident ticket.

## File locations

- Test scripts: [scripts/recovery_test.sh](scripts/recovery_test.sh)
- Windows test script: [scripts/recovery_test.ps1](scripts/recovery_test.ps1)

---
*Created/Updated by DR automation on branch `dr/comprehensive-procedures`.*
