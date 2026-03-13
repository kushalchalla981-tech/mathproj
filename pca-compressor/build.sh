#!/bin/bash

# Build script for PCA Compressor

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print informational message
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

# Print warning message
warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Print error message
error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Main build process
main() {
    info "Starting build process for PCA Compressor..."

    # Check for required tools
    if ! command_exists cargo; then
        error "cargo not found. Please install Rust: https://rustup.rs/"
    fi

    # Cargo version
    info "Cargo version: $(cargo --version)"

    # Parse command line arguments
    BUILD_TYPE="debug"
    BUILD_CLI=true
    BUILD_PYTHON=false
    BUILD_GUI=false
    RUN_TESTS=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --release)
                BUILD_TYPE="release"
                shift
                ;;
            --cli)
                BUILD_CLI=true
                shift
                ;;
            --python)
                BUILD_PYTHON=true
                shift
                ;;
            --gui)
                BUILD_GUI=true
                shift
                ;;
            --all)
                BUILD_CLI=true
                BUILD_PYTHON=true
                BUILD_GUI=true
                shift
                ;;
            --test)
                RUN_TESTS=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --release    Build in release mode"
                echo "  --cli        Build CLI only (default)"
                echo "  --python     Build Python bindings"
                echo "  --gui        Build GUI"
                echo "  --all        Build all components"
                echo "  --test       Run tests after build"
                echo "  --help       Show this help message"
                exit 0
                ;;
            *)
                error "Unknown option: $1. Use --help for usage."
                ;;
        esac
    done

    # Build flag
    BUILD_FLAG=""
    if [[ "$BUILD_TYPE" == "release" ]]; then
        BUILD_FLAG="--release"
    fi

    info "Build type: $BUILD_TYPE"

    # Build CLI
    if [[ "$BUILD_CLI" == true ]]; then
        info "Building CLI..."
        cd crates/pca-cli
        cargo build $BUILD_FLAG
        cd ../..
        info "CLI build complete"
    fi

    # Build Python bindings
    if [[ "$BUILD_PYTHON" == true ]]; then
        if ! command_exists maturin; then
            warn "maturin not found. Install with: pip install maturin"
            warn "Skipping Python bindings build"
        else
            info "Building Python bindings..."
            cd crates/pca-py
            maturin build $BUILD_FLAG
            cd ../..
            info "Python bindings build complete"
        fi
    fi

    # Build GUI
    if [[ "$BUILD_GUI" == true ]]; then
        if ! command_exists tauri; then
            warn "tauri-cli not found. Install with: cargo install tauri-cli"
            warn "Skipping GUI build"
        else
            info "Building GUI..."
            cd crates/pca-gui
            # Note: GUI build requires tauri-cli and may need additional setup
            cargo build $BUILD_FLAG
            cd ../..
            info "GUI build complete"
        fi
    fi

    # Run tests
    if [[ "$RUN_TESTS" == true ]]; then
        info "Running tests..."
        cargo test $BUILD_FLAG
        info "Tests complete"
    fi

    # Summary
    info "Build process complete!"
    info ""
    info "Built components:"
    [[ "$BUILD_CLI" == true ]] && info "  - CLI"
    [[ "$BUILD_PYTHON" == true ]] && info "  - Python bindings"
    [[ "$BUILD_GUI" == true ]] && info "  - GUI"

    # Show artifact locations
    if [[ "$BUILD_TYPE" == "release" ]]; then
        info ""
        info "Artifact locations:"
        [[ "$BUILD_CLI" == true ]] && info "  CLI: target/release/pca-compress"
    else
        info ""
        info "Artifact locations:"
        [[ "$BUILD_CLI" == true ]] && info "  CLI: target/debug/pca-compress"
    fi
}

# Run main function
main "$@"