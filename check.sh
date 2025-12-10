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

Run clippy to check for common mistakes and improve code quality.

Options:
    --no-default-features       Disable default features
    --feature=<name>            Enable specific feature(s) - comma-separated (alias: --features)
    --features=<name>           Enable specific feature(s) - comma-separated
    --all-features              Enable all features (default)
    --package=<name>            Target specific package(s) - can be used multiple times
    -p <name>                   Short form of --package
    --deny-warnings             Treat warnings as errors (default in CI)
    --help                      Display this help message

Examples:
    $0                                          # Check with all features
    $0 --all-features                           # Check with all features
    $0 --no-default-features                    # Check with no features
    $0 --no-default-features --features=my_feature
                                                # Check with only my_feature
    $0 --features=feature1,feature2             # Check with multiple features
    $0 --package=my-crate                       # Check specific package
    $0 -p crate1 -p crate2 --all-features       # Check multiple packages

Requirements:
  - clippy must be installed (rustup component add clippy)

EOF
}

# Check if clippy is installed
check_clippy() {
    if ! rustup component list | grep -q 'clippy.*installed'; then
        print_error "clippy is not installed."
        echo ""
        echo "Please install it with:"
        echo "  rustup component add clippy"
        echo ""
        exit 1
    fi
}

# Main script
main() {
    local feature_flags="--all-features"
    local no_default_features=false
    local deny_warnings=true
    local package_flags=""

    # Parse arguments
    if [ $# -eq 0 ]; then
        feature_flags="--all-features"
    else
        while [ $# -gt 0 ]; do
            case "$1" in
                --no-default-features)
                    no_default_features=true
                    feature_flags=""
                    shift
                    ;;
                --feature=*|--features=*)
                    local feat_value="${1#*=}"
                    if [ -z "$feature_flags" ] || [ "$feature_flags" = "--all-features" ]; then
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
                    if [ -z "$feature_flags" ] || [ "$feature_flags" = "--all-features" ]; then
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
                        print_error "$1 requires a package name"
                        exit 1
                    fi
                    package_flags="$package_flags --package $2"
                    shift 2
                    ;;
                --deny-warnings)
                    deny_warnings=true
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
    fi

    # Check dependencies
    check_clippy

    # Display configuration
    if [ -n "$cargo_flags" ]; then
        print_info "Cargo flags: $cargo_flags"
    else
        print_info "Using default features"
    fi

    if [ "$deny_warnings" = true ]; then
        print_info "Treating warnings as errors"
    fi

    # Run clippy
    print_info "Running clippy..."
    if [ "$deny_warnings" = true ]; then
        cargo clippy $cargo_flags -- -D warnings
    else
        cargo clippy $cargo_flags
    fi

    print_success "Clippy check complete!"
}

main "$@"
