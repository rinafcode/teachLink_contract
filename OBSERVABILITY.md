# Observability (Monitoring, Alerting, Dashboards)

This repo includes an in-repo observability stack for the **TeachLink indexer runtime** (the long-running process) plus **contract-level telemetry** via Soroban events.

## Runtime Metrics + Dashboards (Indexer)

The indexer exports Prometheus metrics and comes with a ready-to-run local monitoring stack (Prometheus, Alertmanager, Grafana).

Start here:

- `indexer/MONITORING.md` (how to run the stack, what metrics exist, alert rules, dashboards)
- `indexer/observability/prometheus/alerts.yml` (alert thresholds)
- `indexer/observability/grafana/dashboards/` (dashboards JSON)

## Contract Telemetry (Soroban Events)

Soroban contracts are not long-running processes, so they are monitored via **events** (and indexed off-chain).

This contract emits:

- `BridgeMetricsUpdatedEvent`
- `ChainMetricsUpdatedEvent`
- `AlertTriggeredEvent` (when `evaluate_alerts` triggers rules)

Bootstrap baseline alert rules (owner-auth):

- `TeachLinkBridge::bootstrap_default_alert_rules(...)`

## Incident Response

For on-call / operational handling, see `INCIDENT_RESPONSE.md`.

