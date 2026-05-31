#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# build.sh — Cross-platform build script for to-clipboard
#
# Usage:
#   ./build.sh              → build for current platform (release)
#   ./build.sh linux        → build for Linux (x86_64 + ARM64)
#   ./build.sh windows      → build for Windows (x86_64)
#   ./build.sh all          → build all supported targets
#   ./build.sh clean        → remove build artifacts
#
# Requirements:
#   - Rust toolchain (rustup, cargo)
#   - Cross-compilation targets added via rustup (run setup first)
# ---------------------------------------------------------------------------

set -euo pipefail
IFS=$'\n\t'

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_DIR="$PROJECT_DIR/releases"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info()  { echo -e "${GREEN}[INFO]${NC}  $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step()  { echo -e "${CYAN}[STEP]${NC}  $*"; }

# ── Setup cross-compilation targets ────────────────────────────────────────
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

# ── Build for a single target ──────────────────────────────────────────────
build_target() {
    local target="$1"
    local suffix="$2"

    log_step "Building for $target..."

    cargo build --release --target "$target" --quiet 2>&1 | tail -5

    local src="$PROJECT_DIR/target/$target/release/to-clipboard"
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
        log_info "→ $(basename "$dest")   $(du -h "$dest" | cut -f1)"
    else
        log_warn "Binary not found at $src — skipping."
    fi
}

# ── Build for current platform only ────────────────────────────────────────
build_current() {
    log_step "Building for current platform..."
    cargo build --release
    strip target/release/to-clipboard 2>/dev/null || true
    log_info "→ target/release/to-clipboard   $(du -h target/release/to-clipboard | cut -f1)"
}

# ── Main ───────────────────────────────────────────────────────────────────
main() {
    cd "$PROJECT_DIR"

    case "${1:-current}" in
        setup)
            setup
            ;;
        linux)
            build_target "x86_64-unknown-linux-gnu"    "linux-x86_64"
            build_target "aarch64-unknown-linux-gnu"   "linux-arm64"
            ;;
        windows)
            build_target "x86_64-pc-windows-gnu"       "windows-x86_64"
            ;;
        macos|osx)
            build_target "x86_64-apple-darwin"         "macos-x86_64"
            build_target "aarch64-apple-darwin"        "macos-arm64"
            ;;
        all)
            log_info "Building all targets into $RELEASE_DIR/ …"
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
            rm -rf "$PROJECT_DIR/target" "$RELEASE_DIR"
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
