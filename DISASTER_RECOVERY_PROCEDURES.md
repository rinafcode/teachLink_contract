# Disaster Recovery Procedures

This runbook defines backup verification, restoration testing, monitoring, and disaster recovery drills for TeachLink.

## Scope

- Smart contract backup manifests and verification events
- Indexer backup/recovery records
- Off-chain backup artifacts referenced by integrity hashes

## Backup Integrity Verification

1. Create a backup manifest on-chain with `create_backup`.
2. Compute and store the off-chain backup hash (same bytes passed as `integrity_hash`).
3. Run `verify_backup` with the expected hash.
4. Confirm `BackupVerifiedEvent` is indexed in `backup_verifications`.

Verification checks:
- Hash match result (`valid=true/false`)
- Verifier identity (`verified_by`)
- Verification timestamp (`verified_at`)
- Ledger/transaction traceability

## Backup Restoration Testing

Run restoration drills at least monthly and after major releases.

Drill workflow:
1. Select a recent backup manifest (`/backup/manifests`).
2. Restore data into an isolated environment.
3. Execute application smoke checks.
4. Record drill outcome on-chain with `record_recovery`.
5. Confirm `RecoveryExecutedEvent` is indexed in `recovery_records`.

Track:
- Recovery duration (`recovery_duration_secs`)
- Success/failure flag (`success`)
- Recovery operator (`executed_by`)

## Monitoring Backup Success Rates

Use indexer backup endpoints:

- `GET /backup/verifications`
- `GET /backup/integrity-metrics?windowHours=24`
- `GET /backup/rto-metrics`
- `GET /backup/recoveries`
- `GET /backup/audit-trail?since=<unix-seconds>`

Primary SLOs:
- Backup verification success rate >= 99%
- Backup coverage rate (backups verified in window) >= 95%
- Recovery drill success rate >= 95%

Alert thresholds:
- Any invalid verification in last 24 hours
- Coverage rate below 95%
- Failed recovery drill

## Disaster Recovery Scenarios To Test

Test each scenario quarterly:

1. Data corruption
2. Partial data loss
3. Full environment loss
4. Indexer database restore
5. Delayed backup verification pipeline

For each scenario, capture:
- Detection timestamp
- Recovery start/end timestamps
- RTO achieved vs target
- Data integrity validation result
- Corrective actions

## Operational Checklist

- Daily: review integrity metrics and invalid verifications.
- Weekly: review backup coverage and missed schedules.
- Monthly: execute at least one restoration drill.
- Quarterly: execute full disaster recovery scenario tests.

## Evidence and Audit Artifacts

Retain for compliance:
- Backup manifests (`backup_manifests`)
- Verification records (`backup_verifications`)
- Recovery records (`recovery_records`)
- Incident reports and drill reports
