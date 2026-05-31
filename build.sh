#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# build.sh - release build helper for to-clipboard
#
# Usage:
#   ./build.sh              -> build for current platform (release)
#   ./build.sh linux        -> build for Linux (x86_64 + ARM64)
#   ./build.sh windows      -> build for Windows (x86_64 GNU)
#   ./build.sh macos        -> build for macOS (x86_64 + ARM64)
#   ./build.sh all          -> build all configured targets
#   ./build.sh clean        -> remove build artifacts
#
# Requirements:
#   - Rust toolchain with cargo
#   - rustup for target installation checks
#   - platform linkers/SDKs for non-native targets
# ---------------------------------------------------------------------------

set -euo pipefail
IFS=$'\n\t'

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_DIR="$PROJECT_DIR/releases"
SAFE_TARGET_DIR="${TO_CLIPBOARD_TARGET_DIR:-${TMPDIR:-/tmp}/to-clipboard-target}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC}  $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step() { echo -e "${CYAN}[STEP]${NC}  $*"; }

require_command() {
    local command_name="$1"

    if ! command -v "$command_name" >/dev/null 2>&1; then
        log_error "Required command not found: $command_name"
        exit 1
    fi
}

target_installed() {
    local target="$1"

    rustup target list --installed | grep -qx "$target"
}

ensure_target_installed() {
    local target="$1"

    if ! target_installed "$target"; then
        log_warn "Rust target '$target' is not installed."
        log_warn "Install it with: rustup target add $target"
        return 1
    fi
}

target_dir_for() {
    local target="$1"

    case "$target" in
        *-pc-windows-gnu)
            printf '%s\n' "$SAFE_TARGET_DIR"
            ;;
        *)
            printf '%s\n' "$PROJECT_DIR/target"
            ;;
    esac
}

# Setup cross-compilation targets
setup() {
    log_step "Installing cross-compilation targets..."
    rustup target add \
        x86_64-unknown-linux-gnu \
        aarch64-unknown-linux-gnu \
        x86_64-pc-windows-gnu \
        x86_64-apple-darwin \
        aarch64-apple-darwin 2>/dev/null || true

    log_info "Available targets:"
    rustup target list --installed
    echo ""
    log_info "Setup complete. Run './build.sh all' to build."
}

# Build for a single target
build_target() {
    local target="$1"
    local suffix="$2"
    local target_dir

    ensure_target_installed "$target" || return 0
    target_dir="$(target_dir_for "$target")"

    log_step "Building for $target..."

    if ! cargo build --release --target "$target" --target-dir "$target_dir"; then
        log_warn "Build failed for $target; skipping this target."
        log_warn "Non-native targets may require an additional cross linker or platform SDK."
        return 0
    fi

    local src="$target_dir/$target/release/to-clipboard"
    local dest="$RELEASE_DIR/to-clipboard-$suffix"

    # Windows binaries get .exe extension
    if [[ "$suffix" == *windows* ]]; then
        src="${src}.exe"
        dest="${dest}.exe"
    fi

    if [[ -f "$src" ]]; then
        mkdir -p "$RELEASE_DIR"
        cp "$src" "$dest"
        strip "$dest" 2>/dev/null || true
        log_info "Built $(basename "$dest") ($(du -h "$dest" | cut -f1))"
    else
        log_warn "Binary not found at $src; skipping."
    fi
}

# Build for current platform only
build_current() {
    log_step "Building for current platform..."
    cargo build --release
    strip target/release/to-clipboard 2>/dev/null || true

    local binary="target/release/to-clipboard"
    if [[ -f "${binary}.exe" ]]; then
        binary="${binary}.exe"
    fi

    log_info "Built $binary ($(du -h "$binary" | cut -f1))"
}

# Main
main() {
    cd "$PROJECT_DIR"
    require_command cargo

    case "${1:-current}" in
        setup)
            require_command rustup
            setup
            ;;
        linux)
            require_command rustup
            build_target "x86_64-unknown-linux-gnu"    "linux-x86_64"
            build_target "aarch64-unknown-linux-gnu"   "linux-arm64"
            ;;
        windows)
            require_command rustup
            build_target "x86_64-pc-windows-gnu"       "windows-x86_64"
            ;;
        macos|osx)
            require_command rustup
            build_target "x86_64-apple-darwin"         "macos-x86_64"
            build_target "aarch64-apple-darwin"        "macos-arm64"
            ;;
        all)
            require_command rustup
            log_info "Building all targets into $RELEASE_DIR/..."
            build_target "x86_64-unknown-linux-gnu"    "linux-x86_64"
            build_target "aarch64-unknown-linux-gnu"   "linux-arm64"
            build_target "x86_64-pc-windows-gnu"       "windows-x86_64"
            build_target "x86_64-apple-darwin"         "macos-x86_64"
            build_target "aarch64-apple-darwin"        "macos-arm64"
            log_info ""
            log_info "All done! Binaries in $RELEASE_DIR/:"
            ls -lh "$RELEASE_DIR/"
            ;;
        clean)
            log_step "Cleaning build artifacts..."
            rm -rf "$PROJECT_DIR/target" "$SAFE_TARGET_DIR" "$RELEASE_DIR"
            log_info "Clean."
            ;;
        current)
            build_current
            ;;
        *)
            log_error "Unknown target: $1"
            echo "Usage: $0 {setup|linux|windows|macos|all|clean|current}"
            exit 1
            ;;
    esac
}

main "$@"
