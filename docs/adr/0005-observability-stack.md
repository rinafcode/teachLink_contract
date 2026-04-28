# ADR-0005: Observability Stack (Prometheus, Alertmanager, Grafana)

**Status:** Accepted  
**Date:** 2024-04-08  
**Authors:** [@rinafcode]  
**Reviewers:** [@rinafcode]  
**Tags:** infrastructure, indexer

---

## Context

The TeachLink indexer is a long-running service with several subsystems (event ingestion, API, background jobs). Production incidents on an event-driven system are hard to diagnose without visibility into throughput, lag, error rates, and resource usage.

Additionally, the Soroban contract itself emits telemetry-relevant events (reward distributions, escrow state changes, bridge operations) that should be surfaced to operators for health monitoring.

An observability stack must be chosen that:

- Collects and stores time-series metrics from the indexer runtime
- Provides alerting for anomalies (ingestion lag, high error rates, validator slashing events)
- Offers dashboards accessible to non-developer operators
- Can be self-hosted alongside the indexer without external SaaS dependency

## Decision

We will use Prometheus for metrics collection, Alertmanager for alert routing, and Grafana for dashboards. The NestJS indexer will expose a `/metrics` endpoint compatible with Prometheus scraping. Contract-level telemetry will be surfaced by mapping Soroban events to Prometheus counters and gauges within the indexer.

## Alternatives Considered

| Alternative | Reason Rejected |
|-------------|-----------------|
| Datadog / New Relic (SaaS APM) | Per-seat and per-metric pricing unsuitable for an open-source project; introduces external data dependency |
| OpenTelemetry collector → Jaeger | Better suited to distributed tracing than time-series metrics; the indexer's primary observability need is metrics and alerting, not trace correlation |
| InfluxDB + Chronograf | Viable time-series stack, but Prometheus + Grafana has broader community adoption, more pre-built dashboards, and stronger NestJS integration libraries |

## Consequences

### Positive

- Prometheus + Grafana is the de-facto standard for self-hosted service monitoring; large library of pre-built dashboards and exporters
- Alertmanager integrates with common notification channels (PagerDuty, Slack, email) without custom webhook code
- NestJS has first-class Prometheus integration via `@willsoto/nestjs-prometheus`
- Entirely self-hosted; no external data egress required

### Negative / Trade-offs

- Requires operators to run three additional services (Prometheus, Alertmanager, Grafana) alongside the indexer
- Long-term metric storage requires configuring Prometheus remote write or a separate time-series store (e.g., Thanos, Mimir) for retention beyond the default 15-day window
- Grafana dashboard-as-code tooling (Grafonnet) adds a learning curve for contributors who want to modify dashboards

### Neutral

- The observability stack is documented separately in `OBSERVABILITY.md` and `indexer/MONITORING.md`; this ADR records only the technology selection rationale

## Implementation Notes

**Affected modules:**

- `indexer/` (metrics endpoint and instrumentation)
- `docker-compose.yml` (Prometheus, Alertmanager, Grafana service definitions)

**Reference docs:**

- `OBSERVABILITY.md`
- `indexer/MONITORING.md`

**Related issues / PRs:** #

---

## Review Checklist

- [x] Context accurately describes the problem without solution bias
- [x] Decision is stated clearly and unambiguously
- [x] At least two alternatives are documented with rejection rationale
- [x] Consequences cover both positive and negative outcomes
- [x] Status field is set correctly
- [x] Tags accurately reflect the domain
- [x] Linked to the relevant GitHub issue or PR