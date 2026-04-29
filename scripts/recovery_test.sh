#!/usr/bin/env bash
set -euo pipefail

START_TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
HEALTH_URL=${HEALTH_URL:-http://localhost:8080/health}
INDEXER_URL=${INDEXER_URL:-http://8081/health}
REPORT=${REPORT:-backups/recovery_reports/$(date -u +"%Y-%m-%dT%H%M%SZ").dr_report.json}

mkdir -p "$(dirname "$REPORT")"

echo "{\"start\": \"$START_TS\"}" > "$REPORT"

echo "Checking service health: $HEALTH_URL"
health_code=$(curl -s -o /dev/null -w "%{http_code}" "$HEALTH_URL" || echo "000")
echo "Checking indexer health: $INDEXER_URL"
indexer_code=$(curl -s -o /dev/null -w "%{http_code}" "$INDEXER_URL" || echo "000")

pass=true
if [ "$health_code" != "200" ] || [ "$indexer_code" != "200" ]; then
  pass=false
fi

END_TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > "$REPORT" <<EOF
{
  "start": "$START_TS",
  "end": "$END_TS",
  "health_status_code": "$health_code",
  "indexer_status_code": "$indexer_code",
  "pass": $pass
}
EOF

echo "DR test complete. Report: $REPORT"

if [ "$pass" != true ]; then
  echo "One or more checks failed" >&2
  exit 2
fi

exit 0
