# Incident Response

This document is the operational runbook for responding to production incidents in TeachLink.

## Signals (How You Find Out)

- Prometheus alerts (see `indexer/observability/prometheus/alerts.yml`)
- Grafana dashboards (see `indexer/observability/grafana/dashboards/`)
- Indexer health endpoint: `GET /health`
- Contract telemetry via Soroban events (indexed off-chain)

## Severity

- SEV1: user-impacting outage or data integrity risk (payments/escrow/bridge stuck, indexer down)
- SEV2: degraded performance, partial feature outage, or elevated error rate
- SEV3: minor degradation, noisy alerts, or non-urgent bugs

## First 10 Minutes Checklist

1. Identify which alert fired and the affected component (indexer, DB, Horizon, etc.).
2. Confirm blast radius in Grafana: error rate, latency, event-processing lag.
3. Check indexer `GET /health` for readiness and dependency status.
4. If indexer is down: restart the service and verify alerts resolve.
5. If DB is down: validate DB connectivity and failover/restart per infra playbook.
6. If Horizon is unreachable: validate network/DNS and consider switching endpoints.

## Mitigation Patterns

- Reduce load: disable non-critical cron/jobs, increase backoff, shed traffic at edge.
- Restore service: restart the failing service, roll back last deploy, or deploy a hotfix.
- Protect integrity: pause/disable risky flows if supported (circuit breakers / admin gates).

## Communications

- Announce incident start (SEV + short symptom).
- Provide updates every 15-30 minutes for SEV1/SEV2 until resolved.
- Announce resolution + customer impact summary.

## Post-Incident

- Write a postmortem within 48 hours.
- Add/adjust alerts to catch recurrence earlier.
- Add regression tests for the root cause where applicable.

