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

Run cargo test with configurable features.

Options:
    --no-default-features       Disable default features
    --feature=<name>            Enable specific feature(s) - comma-separated (alias: --features)
    --features=<name>           Enable specific feature(s) - comma-separated
    --all-features              Enable all features
    --package=<name>            Target specific package(s) - can be used multiple times
    -p <name>                   Short form of --package
    --help                      Display this help message

Examples:
    $0                                          # Run tests with default features
    $0 --all-features                           # Run tests with all features
    $0 --no-default-features                    # Run tests with no features
    $0 --no-default-features --features=feat1
                                                # Run tests with only feat1
    $0 --features=feat1,feat2                   # Run tests with multiple features
    $0 --package=my-crate                       # Run tests for specific package
    $0 -p crate1 -p crate2 --all-features       # Run tests for multiple packages

Note: This script only runs cargo test. For format checking use format.sh,
      for linting use check.sh, and for coverage use coverage.sh.

EOF
}

# Main script
main() {
    local feature_flags=""
    local no_default_features=false
    local package_flags=""

    # Parse arguments
    while [ $# -gt 0 ]; do
        case "$1" in
            --no-default-features)
                no_default_features=true
                shift
                ;;
            --feature=*|--features=*)
                local feat_value="${1#*=}"
                if [ -z "$feature_flags" ]; then
                    feature_flags="--features $feat_value"
                else
                    feature_flags="$feature_flags,$feat_value"
                fi
                shift
                ;;
            --feature|--features)
                if [ $# -lt 2 ]; then
                    print_error "--features requires a feature name"
                    exit 1
                fi
                if [ -z "$feature_flags" ]; then
                    feature_flags="--features $2"
                else
                    feature_flags="$feature_flags,$2"
                fi
                shift 2
                ;;
            --all-features)
                feature_flags="--all-features"
                shift
                ;;
            --package=*)
                package_flags="$package_flags --package ${1#*=}"
                shift
                ;;
            --package|-p)
                if [ $# -lt 2 ]; then
                    print_error "--package requires a package name"
                    exit 1
                fi
                package_flags="$package_flags --package $2"
                shift 2
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

    # Build cargo flags
    local cargo_flags=""
    if [ "$no_default_features" = true ]; then
        cargo_flags="--no-default-features"
    fi
    if [ -n "$feature_flags" ]; then
        cargo_flags="$cargo_flags $feature_flags"
    fi
    if [ -n "$package_flags" ]; then
        cargo_flags="$cargo_flags$package_flags"
        print_info "Target packages:$package_flags"
    fi

    # Display configuration
    if [ -n "$cargo_flags" ]; then
        print_info "Cargo flags: $cargo_flags"
    else
        print_info "Using default features"
    fi

    # Run tests
    print_info "Running tests..."
    cargo test $cargo_flags

    echo ""
    print_success "Tests complete!"
}

main "$@"
