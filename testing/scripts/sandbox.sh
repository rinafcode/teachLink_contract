
#!/usr/bin/env bash
# =============================================================
# TeachLink Sandbox Runner
# Issue #381 — One-command sandbox environment
#
# Usage:
#   ./scripts/sandbox.sh              # Full sandbox run
#   ./scripts/sandbox.sh --no-docker  # Run tests locally (no Docker)
#   ./scripts/sandbox.sh --keep-up    # Don't tear down after tests
#   ./scripts/sandbox.sh --help       # Show this message
# =============================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SANDBOX_ENV="$ROOT_DIR/config/networks/sandbox.env"

# ── Colors ──────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

log()    { echo -e "${BLUE}[sandbox]${NC} $*"; }
ok()     { echo -e "${GREEN}[sandbox]${NC} ✅ $*"; }
warn()   { echo -e "${YELLOW}[sandbox]${NC} ⚠️  $*"; }
err()    { echo -e "${RED}[sandbox]${NC} ❌ $*"; exit 1; }

# ── Flags ────────────────────────────────────────────────────
USE_DOCKER=true
KEEP_UP=false

for arg in "$@"; do
  case $arg in
    --no-docker) USE_DOCKER=false ;;
    --keep-up)   KEEP_UP=true ;;
    --help)
      grep '^#' "$0" | grep -v '/usr/bin' | sed 's/^# \?//'
      exit 0
      ;;
  esac
done

# ── Header ───────────────────────────────────────────────────
echo ""
echo -e "${BOLD}╔════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║   TeachLink Sandbox — Issue #381       ║${NC}"
echo -e "${BOLD}╚════════════════════════════════════════╝${NC}"
echo ""

cd "$ROOT_DIR"

# ── Load sandbox environment ─────────────────────────────────
if [[ -f "$SANDBOX_ENV" ]]; then
  log "Loading sandbox config from $SANDBOX_ENV"
  set -a
  # shellcheck source=/dev/null
  source "$SANDBOX_ENV"
  set +a
  ok "Sandbox config loaded (network=$STELLAR_NETWORK)"
else
  err "Sandbox config not found at $SANDBOX_ENV — run setup first"
fi

# ── Docker path ──────────────────────────────────────────────
if [[ "$USE_DOCKER" == true ]]; then
  command -v docker >/dev/null 2>&1 || err "Docker not found. Install from https://docs.docker.com/get-docker/"

  log "Starting local Stellar node..."
  docker-compose up -d stellar-local

  log "Waiting for Stellar node to be healthy..."
  TRIES=0
  until curl -sf http://localhost:8000 >/dev/null 2>&1; do
    TRIES=$((TRIES + 1))
    if [[ $TRIES -gt 30 ]]; then
      err "Stellar node didn't start after 30 tries. Check: docker-compose logs stellar-local"
    fi
    echo -n "."
    sleep 2
  done
  echo ""
  ok "Stellar node is up at http://localhost:8000"

  log "Running sandbox test suite via Docker..."
  docker-compose run --rm sandbox

  if [[ "$KEEP_UP" == false ]]; then
    log "Tearing down sandbox..."
    docker-compose stop stellar-local sandbox
    docker-compose rm -f stellar-local sandbox
    ok "Sandbox stopped and cleaned up"
  else
    warn "Sandbox kept running (--keep-up). Stop with: docker-compose down"
  fi

# ── Local (no Docker) path ───────────────────────────────────
else
  log "Running sandbox tests locally (no Docker)..."
  warn "Local mode uses Soroban's in-process mock environment only."
  warn "For full network simulation, run without --no-docker."

  export STELLAR_NETWORK=sandbox
  export RUST_BACKTRACE=1

  cargo test --all-features -- --test-threads=1 2>&1

  ok "Local sandbox tests complete"
fi

# ── Summary ──────────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}══════════════════════════════════════════${NC}"
echo -e "${BOLD}${GREEN}  Sandbox run complete! ✅${NC}"
echo -e "${BOLD}${GREEN}══════════════════════════════════════════${NC}"
echo ""
echo "  Next steps:"
echo "  • Check test output above for any failures"
echo "  • Add new tests in testing/sandbox/"
echo "  • Run again anytime: ./scripts/sandbox.sh"
echo ""

chmod +x scripts/sandbox.sh