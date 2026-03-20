#!/usr/bin/env bash
#
# Lumina Toolchain Installer
#
# This script installs the Lumina compiler and runtime environment locally.
# It requires the Rust toolchain (cargo) to be present on the system.

set -e

# --- Configuration ---
REPO_URL="https://github.com/IshimweIsaac/Lumina.git"
INSTALL_DIR="${LUMINA_HOME:-$HOME/.lumina}"
BIN_DIR="$INSTALL_DIR/bin"
EXE_NAME="lumina"

# --- Output Formatting ---
BOLD="$(tput bold 2>/dev/null || echo '')"
RESET="$(tput sgr0 2>/dev/null || echo '')"
GREEN="$(tput setaf 2 2>/dev/null || echo '')"
RED="$(tput setaf 1 2>/dev/null || echo '')"
BLUE="$(tput setaf 4 2>/dev/null || echo '')"

log_info() { echo -e "${BLUE}[INFO]${RESET} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${RESET} ${BOLD}$1${RESET}"; }
log_error() { echo -e "${RED}[ERROR]${RESET} $1"; exit 1; }

# --- Architecture & Prerequisite Checks ---
log_info "Initializing Lumina Toolchain installation..."

if ! command -v cargo >/dev/null 2>&1; then
    log_error "Rust toolchain (cargo) is not installed.\nPlease install Rust via https://rustup.rs/ and try again."
fi

if ! command -v git >/dev/null 2>&1; then
    log_error "Git is required to fetch the Lumina source tree."
fi

# --- Workspace Setup ---
mkdir -p "$BIN_DIR"
TMP_WORKSPACE=$(mktemp -d -t lumina-install-XXXXXX)

log_info "Cloning Lumina repository into temporary workspace..."
git clone --depth 1 "$REPO_URL" "$TMP_WORKSPACE" >/dev/null 2>&1 || log_error "Failed to clone repository. Check network connectivity."

# --- Compilation ---
log_info "Compiling the Lumina runtime (this may take a few minutes)..."
cd "$TMP_WORKSPACE"

# Build the CLI target in release mode
cargo build --release --bin lumina-cli || log_error "Compilation failed. Ensure your Rust toolchain is up-to-date."

# --- Installation ---
log_info "Translocating compiled binary to $BIN_DIR..."
cp "target/release/lumina-cli" "$BIN_DIR/$EXE_NAME"
chmod +x "$BIN_DIR/$EXE_NAME"

# --- Cleanup ---
cd /
rm -rf "$TMP_WORKSPACE"

log_success "Lumina successfully installed at $BIN_DIR/$EXE_NAME"

# --- Path Injection Heuristics ---
case $SHELL in
*/zsh)
    PROFILE="$HOME/.zshrc"
    ;;
*/bash)
    PROFILE="$HOME/.bashrc"
    ;;
*)
    PROFILE="$HOME/.profile"
    ;;
esac

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$PROFILE"
    log_info "Added $BIN_DIR to your PATH in $PROFILE."
    log_info "Please run: source $PROFILE"
else
    log_info "The directory $BIN_DIR is already in your PATH."
fi

log_success "Installation complete! Try running: lumina repl"
