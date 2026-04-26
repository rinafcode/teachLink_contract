#!/usr/bin/env bash
set -euo pipefail

# TeachLink Contract Migration Progress Tracking
# Monitor and report migration progress

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
MIGRATION_DIR="$ROOT_DIR/migration"
LOGS_DIR="$MIGRATION_DIR/logs"
REPORTS_DIR="$MIGRATION_DIR/reports"

COMMAND="status"  # status|monitor|report|cleanup
NETWORK=""
CONTRACT_ID=""
FOLLOW=0
OUTPUT_FORMAT="text"  # text|json|html

usage() {
  cat <<USAGE
Usage: $0 [command] [options]

TeachLink Contract Migration Progress Tracking

Commands:
  status     Show current migration status
  monitor    Monitor ongoing migration (follow mode)
  report     Generate migration report
  cleanup    Clean up old migration artifacts

Options:
  --network <name>       Network name
  --contract-id <id>     Contract ID
  --follow               Follow mode for monitoring
  --format <format>      Output format: text|json|html (default: text)
  -h, --help             Show this help

Examples:
  $0 status --network testnet
  $0 monitor --contract-id CB4HK... --follow
  $0 report --format html > migration_report.html
  $0 cleanup --network testnet

USAGE
}

log() {
  local level="$1"
  local message="$2"
  echo "[$level] $message"
}

error() {
  local message="$1"
  echo "[ERROR] $message" >&2
  exit 1
}

warning() {
  local message="$1"
  echo "[WARN] $message" >&2
}

info() {
  local message="$1"
  echo "[INFO] $message"
}

create_directories() {
  mkdir -p "$LOGS_DIR"
  mkdir -p "$REPORTS_DIR"
  mkdir -p "$MIGRATION_DIR/backups"
}

get_progress_file() {
  local network="${1:-}"
  local contract_id="${2:-}"

  if [[ -n "$contract_id" ]]; then
    echo "$MIGRATION_DIR/.migration_progress_${contract_id}"
  elif [[ -n "$network" ]]; then
    echo "$MIGRATION_DIR/.migration_progress_${network}"
  else
    echo "$MIGRATION_DIR/.migration_progress"
  fi
}

parse_progress_line() {
  local line="$1"
  # Format: step:status:timestamp
  IFS=':' read -r step status timestamp <<< "$line"
  echo "$step|$status|$timestamp"
}

get_migration_status() {
  local progress_file
  progress_file=$(get_progress_file "$NETWORK" "$CONTRACT_ID")

  if [[ ! -f "$progress_file" ]]; then
    echo "No migration in progress"
    return 1
  fi

  echo "Migration Progress ($progress_file):"
  echo "=================================="

  local total_steps=0
  local completed_steps=0
  local failed_steps=0

  while IFS= read -r line; do
    [[ -z "$line" ]] && continue

    IFS='|' read -r step status timestamp <<< "$(parse_progress_line "$line")"

    ((total_steps++))

    case "$status" in
      completed) ((completed_steps++)) ;;
      failed) ((failed_steps++)) ;;
    esac

    local timestamp_readable
    timestamp_readable=$(date -d "@$timestamp" '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo "$timestamp")

    printf "%-25s %-12s %s\n" "$step" "$status" "$timestamp_readable"
  done < "$progress_file"

  echo ""
  echo "Summary:"
  echo "  Total steps: $total_steps"
  echo "  Completed: $completed_steps"
  echo "  Failed: $failed_steps"
  echo "  Success rate: $((total_steps > 0 ? (completed_steps * 100) / total_steps : 0))%"
}

monitor_migration() {
  local progress_file
  progress_file=$(get_progress_file "$NETWORK" "$CONTRACT_ID")

  if [[ ! -f "$progress_file" ]]; then
    error "No migration progress file found"
  fi

  info "Monitoring migration progress... (Ctrl+C to stop)"

  if [[ $FOLLOW -eq 1 ]]; then
    # Follow mode - watch for changes
    local last_size=0
    while true; do
      if [[ -f "$progress_file" ]]; then
        local current_size
        current_size=$(stat -f%z "$progress_file" 2>/dev/null || stat -c%s "$progress_file" 2>/dev/null || echo "0")

        if [[ "$current_size" != "$last_size" ]]; then
          echo ""
          echo "$(date '+%Y-%m-%d %H:%M:%S') - Progress update:"
          get_migration_status
          last_size=$current_size
        fi
      fi
      sleep 2
    done
  else
    get_migration_status
  fi
}

generate_report() {
  local report_file="$REPORTS_DIR/migration_report_$(date +%Y%m%d_%H%M%S)"

  case "$OUTPUT_FORMAT" in
    json) report_file="${report_file}.json" ;;
    html) report_file="${report_file}.html" ;;
    *) report_file="${report_file}.txt" ;;
  esac

  info "Generating migration report: $report_file"

  case "$OUTPUT_FORMAT" in
    json)
      generate_json_report > "$report_file"
      ;;
    html)
      generate_html_report > "$report_file"
      ;;
    text|*)
      generate_text_report > "$report_file"
      ;;
  esac

  info "Report generated: $report_file"
}

generate_text_report() {
  cat <<EOF
TeachLink Contract Migration Report
===================================

Generated: $(date)
Network: ${NETWORK:-All}
Contract ID: ${CONTRACT_ID:-All}

MIGRATION STATUS
-----------------
EOF

  get_migration_status

  cat <<EOF

LOGS AND BACKUPS
-----------------
EOF

  # List recent logs
  echo "Recent migration logs:"
  find "$MIGRATION_DIR" -name "migration_*.log" -type f -mtime -7 2>/dev/null | sort -r | head -10 | while read -r log_file; do
    echo "  $log_file"
  done

  echo ""
  echo "Recent backups:"
  find "$MIGRATION_DIR/backups" -type d -name "20*" -mtime -7 2>/dev/null | sort -r | head -10 | while read -r backup_dir; do
    echo "  $backup_dir"
  done

  cat <<EOF

SYSTEM INFORMATION
------------------
EOF

  echo "Migration directory: $MIGRATION_DIR"
  echo "Available disk space: $(df -h "$MIGRATION_DIR" 2>/dev/null | tail -1 | awk '{print $4}' || echo "Unknown")"
  echo "Migration scripts version: $(git log -1 --oneline -- "$0" 2>/dev/null || echo "Unknown")"
}

generate_json_report() {
  local progress_file
  progress_file=$(get_progress_file "$NETWORK" "$CONTRACT_ID")

  cat <<EOF
{
  "report_type": "migration_status",
  "generated_at": "$(date -Iseconds)",
  "network": "${NETWORK:-null}",
  "contract_id": "${CONTRACT_ID:-null}",
  "migration_status": {
EOF

  if [[ -f "$progress_file" ]]; then
    echo '    "steps": ['
    local first=1
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue

      if [[ $first -eq 0 ]]; then
        echo ','
      fi
      first=0

      IFS='|' read -r step status timestamp <<< "$(parse_progress_line "$line")"
      cat <<EOF
      {
        "step": "$step",
        "status": "$status",
        "timestamp": $timestamp,
        "timestamp_readable": "$(date -d "@$timestamp" '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo "$timestamp")"
      }
EOF
    done < "$progress_file"
    echo '    ]'
  else
    echo '    "steps": []'
  fi

  cat <<EOF
  },
  "system_info": {
    "migration_directory": "$MIGRATION_DIR",
    "scripts_version": "$(git log -1 --oneline -- "$0" 2>/dev/null || echo "unknown")"
  }
}
EOF
}

generate_html_report() {
  cat <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>TeachLink Migration Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f0f0f0; padding: 10px; border-radius: 5px; }
        .status { margin: 20px 0; }
        .step { margin: 5px 0; padding: 5px; border-left: 3px solid #ccc; }
        .completed { border-left-color: #4CAF50; background: #f9fff9; }
        .failed { border-left-color: #f44336; background: #fff9f9; }
        .in-progress { border-left-color: #2196F3; background: #f9f9ff; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <div class="header">
        <h1>TeachLink Contract Migration Report</h1>
        <p>Generated: $(date)</p>
        <p>Network: ${NETWORK:-All}</p>
        <p>Contract ID: ${CONTRACT_ID:-All}</p>
    </div>

    <div class="status">
        <h2>Migration Status</h2>
EOF

  local progress_file
  progress_file=$(get_progress_file "$NETWORK" "$CONTRACT_ID")

  if [[ -f "$progress_file" ]]; then
    echo "        <table>"
    echo "            <tr><th>Step</th><th>Status</th><th>Timestamp</th></tr>"

    while IFS= read -r line; do
      [[ -z "$line" ]] && continue

      IFS='|' read -r step status timestamp <<< "$(parse_progress_line "$line")"
      local timestamp_readable
      timestamp_readable=$(date -d "@$timestamp" '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo "$timestamp")

      local css_class
      case "$status" in
        completed) css_class="completed" ;;
        failed) css_class="failed" ;;
        *) css_class="in-progress" ;;
      esac

      echo "            <tr class=\"$css_class\"><td>$step</td><td>$status</td><td>$timestamp_readable</td></tr>"
    done < "$progress_file"

    echo "        </table>"
  else
    echo "        <p>No migration in progress</p>"
  fi

  cat <<EOF
    </div>

    <div class="status">
        <h2>System Information</h2>
        <ul>
            <li>Migration Directory: $MIGRATION_DIR</li>
            <li>Scripts Version: $(git log -1 --oneline -- "$0" 2>/dev/null || echo "Unknown")</li>
        </ul>
    </div>
</body>
</html>
EOF
}

cleanup_artifacts() {
  info "Cleaning up old migration artifacts..."

  local days=30  # Keep artifacts for 30 days

  # Remove old progress files
  find "$MIGRATION_DIR" -name ".migration_progress*" -type f -mtime +$days -delete 2>/dev/null || true

  # Remove old logs
  find "$MIGRATION_DIR" -name "migration_*.log" -type f -mtime +$days -delete 2>/dev/null || true

  # Remove old backups (be careful with this)
  if confirm "Remove backup directories older than $days days?"; then
    find "$MIGRATION_DIR/backups" -type d -name "20*" -mtime +$days -exec rm -rf {} + 2>/dev/null || true
  fi

  # Remove old reports
  find "$REPORTS_DIR" -name "migration_report_*" -type f -mtime +$days -delete 2>/dev/null || true

  info "Cleanup completed"
}

confirm() {
  local prompt="$1"
  read -r -p "$prompt [Y/n] " reply
  [[ -z "$reply" || "$reply" =~ ^[Yy]$ ]]
}

main() {
  # Parse command
  if [[ $# -gt 0 && ! "$1" =~ ^-- ]]; then
    COMMAND="$1"
    shift
  fi

  # Parse options
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --network) NETWORK="$2"; shift 2 ;;
      --contract-id) CONTRACT_ID="$2"; shift 2 ;;
      --follow) FOLLOW=1; shift ;;
      --format) OUTPUT_FORMAT="$2"; shift 2 ;;
      -h|--help) usage; exit 0 ;;
      *) error "Unknown option: $1" ;;
    esac
  done

  create_directories

  case "$COMMAND" in
    status) get_migration_status ;;
    monitor) monitor_migration ;;
    report) generate_report ;;
    cleanup) cleanup_artifacts ;;
    *) error "Unknown command: $COMMAND" ;;
  esac
}

# Run main function
main "$@"