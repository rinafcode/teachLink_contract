# TeachLink Indexer Monitoring

This repository now includes a baseline observability stack for the production runtime that exists in-repo: the NestJS indexer service and its PostgreSQL dependency.

## Scope and architecture

In-scope services:

- `indexer-prod`: the long-running TeachLink indexer API and event processor
- `postgres`: the indexer datastore
- `prometheus`: metrics collection and rule evaluation
- `alertmanager`: alert routing and alert state inspection
- `grafana`: operational dashboards
- `postgres-exporter`: PostgreSQL metrics exporter
- `blackbox-exporter`: HTTP health probing for the indexer
- `alert-webhook`: local webhook sink used to verify end-to-end alert delivery

Out of scope:

- Soroban contracts themselves are not long-running processes in this repository and therefore are not scraped directly.
- Managed Stellar infrastructure outside this repository is only checked indirectly through the indexer's Horizon dependency health check.

Architecture:

1. `indexer-prod` exposes `GET /health`, `GET /metrics`, and `GET /metrics/json`.
2. Prometheus scrapes the indexer metrics endpoint, PostgreSQL exporter, and the monitoring stack itself.
3. Blackbox Exporter probes `http://indexer-prod:3000/health` for user-facing availability.
4. Alertmanager receives Prometheus alerts and forwards them to the local webhook sink for verifiable testing.
5. Grafana provisions dashboards from JSON files under `observability/grafana/dashboards/`.

## Metrics sources

### Indexer metrics

The indexer now exports Prometheus metrics from `/metrics`, including:

- HTTP request volume by route, method, and status code
- HTTP request latency
- Dashboard cache hits and misses
- Dashboard generation latency
- Persisted indexer totals for processed events and errors
- Last processed ledger
- Last processed event timestamp
- Event-processing lag in seconds
- Dependency health for PostgreSQL and Horizon
- Default process and Node.js runtime metrics from `prom-client`

The `/health` endpoint now returns a structured readiness payload with:

- overall service status
- PostgreSQL connectivity status
- Horizon reachability status
- indexer state freshness based on `INDEXER_STALE_AFTER_SECONDS`

### PostgreSQL metrics

`postgres-exporter` supplies database availability and core PostgreSQL metrics such as `pg_up`.

### Probe-based metrics

`blackbox-exporter` records `probe_success` against the indexer health endpoint, which catches failures that a plain scrape may miss.

## Alert rules

Alert rules are defined in `observability/prometheus/alerts.yml`.

Critical alerts:

- `TeachLinkIndexerDown`: Prometheus cannot scrape the indexer metrics endpoint
- `TeachLinkIndexerHealthcheckFailed`: the indexer health endpoint fails blackbox probing
- `TeachLinkIndexerDatabaseUnavailable`: PostgreSQL is unreachable from the app or exporter
- `TeachLinkIndexerHorizonUnavailable`: the indexer cannot reach Stellar Horizon
- `TeachLinkIndexerStaleProcessing`: no event has been processed within the configured freshness window

Warning alerts:

- `TeachLinkIndexerHighHttpErrorRate`: 5xx rate above 5% over 10 minutes
- `TeachLinkIndexerHighLatency`: average HTTP latency above 1 second over 10 minutes
- `TeachLinkGrafanaDown`: Grafana scrape target unavailable
- `TeachLinkAlertmanagerDown`: Alertmanager scrape target unavailable

Thresholds are intentionally conservative to keep noise low while still surfacing real failures.

## Dashboards

Provisioned dashboards:

- `TeachLink Service Overview`
- `TeachLink Platform Dependencies`

They show:

- service uptime and probe health
- indexer ledger lag and processing totals
- throughput, latency, and active alerts
- database reachability and dependency health
- runtime memory and scrape health

## Running the stack

From `indexer/`:

```bash
cp .env.example .env
docker compose --profile production --profile observability up -d
```

Endpoints:

- Prometheus: `http://localhost:${PROMETHEUS_PORT:-9090}`
- Alertmanager: `http://localhost:${ALERTMANAGER_PORT:-9093}`
- Grafana: `http://localhost:${GRAFANA_PORT:-3001}`
- Indexer health: `http://localhost:3000/health`
- Indexer metrics: `http://localhost:3000/metrics`
- Alert webhook sink: `http://localhost:5001/alerts`

Grafana credentials come from `GRAFANA_ADMIN_USER` and `GRAFANA_ADMIN_PASSWORD`. Set them in `.env` rather than editing compose files.

## Alert test procedure

### Automated test

Run:

```bash
./observability/test-alerting.sh
```

What it does:

1. Starts the production and observability profiles.
2. Stops `indexer-prod`.
3. Polls Alertmanager until `TeachLinkIndexerDown` is firing.
4. Fetches the webhook payload from the local receiver.
5. Starts `indexer-prod` again before exiting.

Expected result:

- the script prints that `TeachLinkIndexerDown` is firing
- `http://localhost:9093` shows the alert as active
- `http://localhost:5001/alerts` shows the delivered payload

### Manual verification

1. Start the stack with the production and observability profiles.
2. Open Grafana and confirm both dashboards are loaded.
3. Open Prometheus and verify targets for `teachlink-indexer`, `postgres-exporter`, `grafana`, `alertmanager`, and `teachlink-indexer-health` are `UP`.
4. Stop `indexer-prod` with `docker compose stop indexer-prod`.
5. Wait at least 2 minutes for the `TeachLinkIndexerDown` and `TeachLinkIndexerHealthcheckFailed` alerts to fire.
6. Confirm Alertmanager lists the alerts and the webhook sink received a payload.
7. Start the service again with `docker compose start indexer-prod` and verify the alerts resolve.

## Validation

Recommended checks:

```bash
docker compose config
docker run --rm -v "$PWD/observability/prometheus:/workspace" prom/prometheus:v2.54.1 promtool check config /workspace/prometheus.yml
docker run --rm -v "$PWD/observability/prometheus:/workspace" prom/prometheus:v2.54.1 promtool check rules /workspace/alerts.yml
npm install
npm run build
```

## Security notes

- No alerting secrets or Grafana credentials are hardcoded in the configs.
- Metrics remain on the private compose network unless you intentionally publish the relevant ports.
- The local `alert-webhook` service is for validation and can be replaced with an external notification integration in real deployments.
- Monitoring does not log wallet secrets or contract private material.

## Limitations

- The repository does not manage host-level infrastructure, so host CPU or disk alerts are not included.
- Horizon is checked as an external dependency via HTTP reachability only.
- The local webhook sink is meant for baseline verification; production teams should swap it for Slack, PagerDuty, Opsgenie, or email routing appropriate to their environment.
