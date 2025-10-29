#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored messages
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_success() {
    echo -e "${BLUE}[SUCCESS]${NC} $1"
}

# Function to display help
show_help() {
    cat << EOF
Usage: $0 [OPTION]

Generate code coverage reports using cargo-llvm-cov.

Options:
    --html               Generate HTML report and open in browser (default)
    --lcov               Generate lcov.info file for CI/codecov
    --text               Show coverage summary in terminal
    --all                Generate all formats
    --clean              Clean coverage artifacts before running
    --check-thresholds   Enforce 80% minimum coverage for lines, regions, and functions
    --help               Display this help message

Examples:
    $0                        # Generate HTML report
    $0 --html                 # Generate HTML report
    $0 --lcov                 # Generate lcov.info
    $0 --text                 # Show text summary
    $0 --all                  # Generate all formats
    $0 --check-thresholds     # Check coverage meets 80% thresholds

Requirements:
  - cargo-llvm-cov must be installed (cargo install cargo-llvm-cov)

Coverage Thresholds:
  When --check-thresholds is used, the script will fail if:
  - Line coverage < 80%
  - Region coverage < 80%
  - Function coverage < 80%

EOF
}

# Check if cargo-llvm-cov is installed
check_dependencies() {
    if ! command -v cargo-llvm-cov &> /dev/null; then
        print_error "cargo-llvm-cov is not installed."
        echo ""
        echo "Please install it with:"
        echo "  cargo install cargo-llvm-cov"
        echo ""
        exit 1
    fi
}

# Clean coverage artifacts
clean_coverage() {
    print_info "Cleaning coverage artifacts..."
    cargo llvm-cov clean --workspace
    rm -f lcov.info
    print_info "Coverage artifacts cleaned"
}

# Generate HTML coverage report
generate_html() {
    local threshold_flags="$1"
    print_info "Generating HTML coverage report..."
    cargo llvm-cov --workspace --html $threshold_flags
    print_success "HTML coverage report generated at target/llvm-cov/html/index.html"

    # Open in browser
    if command -v open &> /dev/null; then
        open target/llvm-cov/html/index.html
    elif command -v xdg-open &> /dev/null; then
        xdg-open target/llvm-cov/html/index.html
    else
        print_warning "Could not open browser automatically"
        print_info "Open target/llvm-cov/html/index.html in your browser"
    fi
}

# Generate lcov report
generate_lcov() {
    local threshold_flags="$1"
    print_info "Generating lcov report..."
    cargo llvm-cov --workspace --lcov --output-path lcov.info $threshold_flags
    print_success "lcov report generated at lcov.info"
}

# Generate text summary
generate_text() {
    local threshold_flags="$1"
    print_info "Generating coverage summary..."
    cargo llvm-cov --workspace $threshold_flags
}

# Main script
main() {
    local clean=false
    local format="html"
    local check_thresholds=false
    local threshold_flags=""

    # Parse arguments
    if [ $# -eq 0 ]; then
        format="html"
    else
        while [ $# -gt 0 ]; do
            case "$1" in
                --html)
                    format="html"
                    shift
                    ;;
                --lcov)
                    format="lcov"
                    shift
                    ;;
                --text)
                    format="text"
                    shift
                    ;;
                --all)
                    format="all"
                    shift
                    ;;
                --clean)
                    clean=true
                    shift
                    ;;
                --check-thresholds)
                    check_thresholds=true
                    shift
                    ;;
                --help|-h)
                    show_help
                    exit 0
                    ;;
                *)
                    print_error "Unknown option: $1"
                    echo ""
                    show_help
                    exit 1
                    ;;
            esac
        done
    fi

    # Set threshold flags if requested
    if [ "$check_thresholds" = true ]; then
        threshold_flags="--fail-under-lines 80 --fail-under-regions 80 --fail-under-functions 80"
        print_info "Coverage thresholds enabled: Lines/Regions/Functions must be ≥ 80%"
    fi

    # Check dependencies
    check_dependencies

    # Clean if requested
    if [ "$clean" = true ]; then
        clean_coverage
    fi

    # Generate coverage based on format
    case "$format" in
        html)
            generate_html "$threshold_flags"
            ;;
        lcov)
            generate_lcov "$threshold_flags"
            ;;
        text)
            generate_text "$threshold_flags"
            ;;
        all)
            generate_text "$threshold_flags"
            echo ""
            generate_html "$threshold_flags"
            echo ""
            generate_lcov "$threshold_flags"
            ;;
    esac

    echo ""
    print_success "Coverage generation complete!"
}

main "$@"
