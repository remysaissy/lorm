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
    --package <name>            Generate coverage only for specified package (also: --package=<name>)
    --all-packages              Generate coverage for all packages (default)
    --html                      Generate HTML report and open in browser (default)
    --lcov                      Generate lcov.info file for CI/codecov
    --text                      Show coverage summary in terminal
    --all                       Generate all formats
    --clean                     Clean coverage artifacts before running
    --check-thresholds          Enforce minimum coverage for lines, regions, and functions
    --threshold <value>         Set all coverage thresholds to the same value (default: 80)
    --threshold-lines <value>   Set line coverage threshold (default: 80)
    --threshold-regions <value> Set region coverage threshold (default: 80)
    --threshold-functions <value> Set function coverage threshold (default: 80)
    --no-default-features       Disable default features (like cargo --no-default-features)
    --features <name>           Enable specific feature(s) - can be comma-separated (also: --features=<name>)
    --all-features              Enable all features during coverage
    --help                      Display this help message

Examples:
    $0                                          # Generate HTML report for all packages
    $0 --all-packages                           # Generate HTML report for all packages
    $0 --package outreach-web                   # Generate coverage for outreach-web only
    $0 --package=outreach-web                   # Same as above (POSIX style)
    $0 --package outreach-api --html            # Generate HTML report for outreach-api
    $0 --html                                   # Generate HTML report
    $0 --lcov                                   # Generate lcov.info
    $0 --text                                   # Show text summary
    $0 --all                                    # Generate all formats
    $0 --check-thresholds                       # Check coverage meets 80% thresholds (default)
    $0 --check-thresholds --threshold=90        # Check coverage meets 90% thresholds
    $0 --check-thresholds --threshold-lines=85  # Check line coverage meets 85%
    $0 --check-thresholds --threshold-lines=85 --threshold-regions=80 --threshold-functions=90
                                                # Check different thresholds for each metric
    $0 --all-features                           # Generate report with all features enabled
    $0 --no-default-features                    # Generate report with no features
    $0 --no-default-features --features=feat1   # Generate report with only feat1
    $0 --features=feat1,feat2                   # Generate report with multiple features
    $0 --no-default-features --features=feat1 --check-thresholds
                                                # Check coverage for feat1 only

Requirements:
  - cargo-llvm-cov must be installed (cargo install cargo-llvm-cov)

Coverage Thresholds:
  When --check-thresholds is used, the script will fail if coverage is below the thresholds.
  Default threshold for all metrics is 80%.

  You can customize thresholds using:
  - --threshold <value>: Set all thresholds to the same value
  - --threshold-lines <value>: Set line coverage threshold
  - --threshold-regions <value>: Set region coverage threshold
  - --threshold-functions <value>: Set function coverage threshold

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
    local cargo_flags="$2"
    local package_flags="$3"
    print_info "Generating HTML coverage report..."
    cargo llvm-cov $package_flags --html $threshold_flags $cargo_flags
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
    local cargo_flags="$2"
    local package_flags="$3"
    print_info "Generating lcov report..."
    cargo llvm-cov $package_flags --lcov --output-path lcov.info $threshold_flags $cargo_flags
    print_success "lcov report generated at lcov.info"
}

# Generate text summary
generate_text() {
    local threshold_flags="$1"
    local cargo_flags="$2"
    local package_flags="$3"
    print_info "Generating coverage summary..."
    cargo llvm-cov $package_flags $threshold_flags $cargo_flags
}

# Main script
main() {
    local clean=false
    local format="html"
    local check_thresholds=false
    local threshold_flags=""
    local feature_flags=""
    local no_default_features=false
    local threshold_lines=80
    local threshold_regions=80
    local threshold_functions=80
    local target_package=""
    local all_packages=true

    # Parse arguments
    if [ $# -eq 0 ]; then
        format="html"
    else
        while [ $# -gt 0 ]; do
            case "$1" in
                --package=*)
                    target_package="${1#*=}"
                    all_packages=false
                    shift
                    ;;
                --package)
                    if [ $# -lt 2 ]; then
                        print_error "--package requires a package name"
                        exit 1
                    fi
                    target_package="$2"
                    all_packages=false
                    shift 2
                    ;;
                --all-packages)
                    all_packages=true
                    target_package=""
                    shift
                    ;;
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
                --threshold=*)
                    local thresh_value="${1#*=}"
                    threshold_lines="$thresh_value"
                    threshold_regions="$thresh_value"
                    threshold_functions="$thresh_value"
                    shift
                    ;;
                --threshold)
                    if [ $# -lt 2 ]; then
                        print_error "--threshold requires a value"
                        exit 1
                    fi
                    threshold_lines="$2"
                    threshold_regions="$2"
                    threshold_functions="$2"
                    shift 2
                    ;;
                --threshold-lines=*)
                    threshold_lines="${1#*=}"
                    shift
                    ;;
                --threshold-lines)
                    if [ $# -lt 2 ]; then
                        print_error "--threshold-lines requires a value"
                        exit 1
                    fi
                    threshold_lines="$2"
                    shift 2
                    ;;
                --threshold-regions=*)
                    threshold_regions="${1#*=}"
                    shift
                    ;;
                --threshold-regions)
                    if [ $# -lt 2 ]; then
                        print_error "--threshold-regions requires a value"
                        exit 1
                    fi
                    threshold_regions="$2"
                    shift 2
                    ;;
                --threshold-functions=*)
                    threshold_functions="${1#*=}"
                    shift
                    ;;
                --threshold-functions)
                    if [ $# -lt 2 ]; then
                        print_error "--threshold-functions requires a value"
                        exit 1
                    fi
                    threshold_functions="$2"
                    shift 2
                    ;;
                --no-default-features)
                    no_default_features=true
                    shift
                    ;;
                --features=*)
                    local feat_value="${1#*=}"
                    if [ -z "$feature_flags" ]; then
                        feature_flags="--features $feat_value"
                    else
                        feature_flags="$feature_flags,$feat_value"
                    fi
                    shift
                    ;;
                --features)
                    if [ $# -lt 2 ]; then
                        print_error "--features requires a feature name"
                        exit 1
                    fi
                    # Support multiple --features flags by appending
                    if [ -z "$feature_flags" ]; then
                        feature_flags="--features $2"
                    else
                        # If already has --features, append with comma
                        feature_flags="$feature_flags,$2"
                    fi
                    shift 2
                    ;;
                --all-features)
                    feature_flags="--all-features"
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

    # Build feature flags string
    local cargo_flags=""
    if [ "$no_default_features" = true ]; then
        cargo_flags="--no-default-features"
    fi
    if [ -n "$feature_flags" ]; then
        cargo_flags="$cargo_flags $feature_flags"
    fi

    # Build package flags
    local package_flags=""
    if [ -n "$target_package" ]; then
        package_flags="--package $target_package"
    else
        package_flags="--workspace"
    fi

    # Set threshold flags if requested
    if [ "$check_thresholds" = true ]; then
        threshold_flags="--fail-under-lines $threshold_lines --fail-under-regions $threshold_regions --fail-under-functions $threshold_functions"
        print_info "Coverage thresholds enabled: Lines >= ${threshold_lines}%, Regions >= ${threshold_regions}%, Functions >= ${threshold_functions}%"
    fi

    # Display configuration
    if [ -n "$target_package" ]; then
        print_info "Target package: $target_package"
    else
        print_info "Target: all packages (workspace)"
    fi

    if [ -n "$cargo_flags" ]; then
        print_info "Cargo flags: $cargo_flags"
    else
        print_info "Using default features"
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
            generate_html "$threshold_flags" "$cargo_flags" "$package_flags"
            ;;
        lcov)
            generate_lcov "$threshold_flags" "$cargo_flags" "$package_flags"
            ;;
        text)
            generate_text "$threshold_flags" "$cargo_flags" "$package_flags"
            ;;
        all)
            generate_text "$threshold_flags" "$cargo_flags" "$package_flags"
            echo ""
            generate_html "$threshold_flags" "$cargo_flags" "$package_flags"
            echo ""
            generate_lcov "$threshold_flags" "$cargo_flags" "$package_flags"
            ;;
    esac

    echo ""
    print_success "Coverage generation complete!"
}

main "$@"
