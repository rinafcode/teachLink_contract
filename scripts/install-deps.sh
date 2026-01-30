#!/usr/bin/env bash
# Automated dependency installation script for TeachLink development environment
set -euo pipefail

color() {
  if [[ -t 1 ]]; then
    case "$1" in
      green) printf "\033[32m" ;;
      yellow) printf "\033[33m" ;;
      red) printf "\033[31m" ;;
      blue) printf "\033[34m" ;;
      reset) printf "\033[0m" ;;
    esac
  fi
}

info() { color green; printf "[✓]"; color reset; printf " %s\n" "$*"; }
warn() { color yellow; printf "[⚠]"; color reset; printf " %s\n" "$*"; }
err() { color red; printf "[✗]"; color reset; printf " %s\n" "$*"; }
section() { color blue; printf "\n▸ %s\n" "$*"; color reset; }
prompt() { color yellow; printf "[?]"; color reset; printf " %s " "$*"; }

# Detect OS
detect_os() {
  if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "macos"
  elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "linux"
  else
    echo "unknown"
  fi
}

OS=$(detect_os)

printf "╔══════════════════════════════════════════════════════════╗\n"
printf "║   TeachLink Automated Dependency Installation           ║\n"
printf "╚══════════════════════════════════════════════════════════╝\n"
printf "\nDetected OS: %s\n" "$OS"

# Install Rust
install_rust() {
  section "Installing Rust"

  if command -v rustc >/dev/null 2>&1; then
    info "Rust is already installed: $(rustc --version)"
    return 0
  fi

  prompt "Rust is not installed. Install now? (y/n)"
  read -r response
  if [[ "$response" != "y" ]]; then
    warn "Skipping Rust installation"
    return 1
  fi

  info "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

  # Source cargo env
  if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
    info "Rust installed successfully: $(rustc --version)"
  else
    err "Rust installation may have failed. Please check manually."
    return 1
  fi
}

# Install wasm32 target
install_wasm_target() {
  section "Installing WASM Target"

  if ! command -v rustup >/dev/null 2>&1; then
    err "rustup not found. Install Rust first."
    return 1
  fi

  if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    info "wasm32-unknown-unknown target is already installed"
    return 0
  fi

  info "Installing wasm32-unknown-unknown target..."
  rustup target add wasm32-unknown-unknown

  if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    info "wasm32-unknown-unknown target installed successfully"
  else
    err "Failed to install wasm32-unknown-unknown target"
    return 1
  fi
}

# Install Stellar CLI
install_stellar_cli() {
  section "Installing Stellar CLI"

  if command -v stellar >/dev/null 2>&1; then
    info "Stellar CLI is already installed: $(stellar --version)"
    return 0
  fi

  if command -v soroban >/dev/null 2>&1; then
    info "Soroban CLI is already installed: $(soroban --version)"
    return 0
  fi

  prompt "Stellar/Soroban CLI is not installed. Install now? (y/n)"
  read -r response
  if [[ "$response" != "y" ]]; then
    warn "Skipping Stellar CLI installation"
    return 1
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    err "cargo not found. Install Rust first."
    return 1
  fi

  info "Installing Stellar CLI (this may take several minutes)..."
  cargo install --locked stellar-cli --features opt

  if command -v stellar >/dev/null 2>&1; then
    info "Stellar CLI installed successfully: $(stellar --version)"
  else
    err "Stellar CLI installation may have failed. Please check manually."
    return 1
  fi
}

# Install Docker (optional)
install_docker() {
  section "Installing Docker (Optional)"

  if command -v docker >/dev/null 2>&1; then
    info "Docker is already installed: $(docker --version)"
    return 0
  fi

  prompt "Docker is not installed. Would you like installation instructions? (y/n)"
  read -r response
  if [[ "$response" != "y" ]]; then
    warn "Skipping Docker installation"
    return 0
  fi

  if [[ "$OS" == "macos" ]]; then
    printf "\nTo install Docker on macOS:\n"
    printf "1. Download Docker Desktop from: https://docs.docker.com/desktop/install/mac-install/\n"
    printf "2. Install Docker Desktop and launch it\n"
    printf "3. Verify installation with: docker --version\n"
  elif [[ "$OS" == "linux" ]]; then
    printf "\nTo install Docker on Linux:\n"
    printf "1. Follow instructions at: https://docs.docker.com/engine/install/\n"
    printf "2. Add your user to docker group: sudo usermod -aG docker \$USER\n"
    printf "3. Verify installation with: docker --version\n"
  else
    printf "\nPlease visit https://docs.docker.com/get-docker/ for installation instructions.\n"
  fi
}

# Install additional tools
install_additional_tools() {
  section "Additional Tools"

  # Git
  if ! command -v git >/dev/null 2>&1; then
    warn "Git is not installed (recommended for version control)"
    printf "  → Install: https://git-scm.com/downloads\n"
  else
    info "Git is installed: $(git --version)"
  fi

  # curl
  if ! command -v curl >/dev/null 2>&1; then
    warn "curl is not installed (required for some scripts)"
    if [[ "$OS" == "macos" ]]; then
      printf "  → Should be pre-installed on macOS\n"
    elif [[ "$OS" == "linux" ]]; then
      printf "  → Install: sudo apt-get install curl (Debian/Ubuntu)\n"
      printf "  → Install: sudo yum install curl (RHEL/CentOS)\n"
    fi
  else
    info "curl is installed: $(curl --version | head -n1)"
  fi
}

# Update Rust toolchain
update_rust() {
  section "Updating Rust Toolchain"

  if command -v rustup >/dev/null 2>&1; then
    info "Updating Rust toolchain..."
    rustup update stable
    rustup component add rustfmt clippy
    info "Rust toolchain updated"
  else
    warn "rustup not found. Skipping update."
  fi
}

# Main installation flow
main() {
  install_rust || true
  install_wasm_target || true
  install_stellar_cli || true
  update_rust || true
  install_additional_tools || true
  install_docker || true

  # Summary
  printf "\n╔══════════════════════════════════════════════════════════╗\n"
  printf "║       Installation Complete                              ║\n"
  printf "╚══════════════════════════════════════════════════════════╝\n\n"

  printf "Next steps:\n"
  printf "1. Run validation: ./scripts/validate-env.sh\n"
  printf "2. Configure environment: cp .env.example .env\n"
  printf "3. Generate deployer key: stellar keys generate --global teachlink-deployer\n"
  printf "4. Start developing: ./scripts/dev.sh\n\n"
}

main
