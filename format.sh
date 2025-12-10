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

Run rustfmt to check or format code.

Options:
    --check               Check formatting without modifying files (default)
    --fix                 Format code in place
    --package=<name>      Target specific package(s) - can be used multiple times
    -p <name>             Short form of --package
    --help                Display this help message

Examples:
    $0                        # Check formatting for all packages
    $0 --check                # Check formatting
    $0 --fix                  # Format code
    $0 --package=my-crate     # Check formatting for specific package
    $0 -p crate1 -p crate2    # Check formatting for multiple packages

Requirements:
  - rustfmt must be installed (rustup component add rustfmt)

EOF
}

# Check if rustfmt is installed
check_rustfmt() {
    if ! rustup component list | grep -q 'rustfmt.*installed'; then
        print_error "rustfmt is not installed."
        echo ""
        echo "Please install it with:"
        echo "  rustup component add rustfmt"
        echo ""
        exit 1
    fi
}

# Main script
main() {
    local check_mode=true
    local package_flags=""

    # Parse arguments
    while [ $# -gt 0 ]; do
        case "$1" in
            --check)
                check_mode=true
                shift
                ;;
            --fix)
                check_mode=false
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

    # Check dependencies
    check_rustfmt

    # Build cargo fmt flags
    local cargo_flags=""
    if [ -n "$package_flags" ]; then
        cargo_flags="$package_flags"
        print_info "Target packages:$package_flags"
    else
        cargo_flags="--all"
        print_info "Targeting all packages"
    fi

    # Run format check or fix
    if [ "$check_mode" = true ]; then
        print_info "Checking code formatting..."
        cargo fmt $cargo_flags -- --check
        print_success "Format check complete!"
    else
        print_info "Formatting code..."
        cargo fmt $cargo_flags
        print_success "Code formatted!"
    fi
}

main "$@"
