#!/usr/bin/env sh

set -eu

COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.yml}"
ALERT_NAME="${ALERT_NAME:-TeachLinkIndexerDown}"
WAIT_SECONDS="${WAIT_SECONDS:-150}"
POLL_SECONDS="${POLL_SECONDS:-10}"

echo "Starting TeachLink production and observability profiles..."
docker compose -f "$COMPOSE_FILE" --profile production --profile observability up -d \
  postgres indexer-prod prometheus alertmanager grafana postgres-exporter blackbox-exporter alert-webhook

echo "Stopping indexer-prod to trigger ${ALERT_NAME}..."
docker compose -f "$COMPOSE_FILE" stop indexer-prod

elapsed=0
while [ "$elapsed" -lt "$WAIT_SECONDS" ]; do
  if curl -fsS "http://localhost:9093/api/v2/alerts" | grep -q "\"alertname\":\"${ALERT_NAME}\""; then
    echo "Alert ${ALERT_NAME} is firing in Alertmanager."
    echo "Latest webhook payload:"
    curl -fsS "http://localhost:5001/alerts"
    docker compose -f "$COMPOSE_FILE" start indexer-prod >/dev/null
    exit 0
  fi

  sleep "$POLL_SECONDS"
  elapsed=$((elapsed + POLL_SECONDS))
done

echo "Alert ${ALERT_NAME} did not appear within ${WAIT_SECONDS}s."
docker compose -f "$COMPOSE_FILE" start indexer-prod >/dev/null
exit 1
