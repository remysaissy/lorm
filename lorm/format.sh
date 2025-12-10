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

Run rustfmt and dx fmt (for Dioxus packages) to check or format Rust and RSX code.

Options:
    --package <name>            Format only specified package (also: --package=<name>)
    --all-packages              Format all packages (default)
    --check                     Check formatting without modifying files (default)
    --fix                       Format code in place
    --help                      Display this help message

Examples:
    $0                                  # Check formatting on all packages
    $0 --all-packages                   # Check formatting on all packages
    $0 --package outreach-web           # Check formatting only on outreach-web
    $0 --package=outreach-web           # Same as above (POSIX style)
    $0 --check                          # Check formatting
    $0 --fix                            # Format code
    $0 --package=outreach-web --fix     # Format only outreach-web package

Requirements:
  - rustfmt must be installed (rustup component add rustfmt)
  - dx (Dioxus CLI) should be installed for RSX formatting (cargo install dioxus-cli)

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

# Check if dx is installed
check_dx() {
    if ! command -v dx &> /dev/null; then
        print_warning "dx (Dioxus CLI) is not installed."
        echo ""
        echo "Please install it with:"
        echo "  cargo install dioxus-cli"
        echo ""
        return 1
    fi
    return 0
}

# Get all workspace packages from Cargo.toml
get_workspace_packages() {
    # Extract package names from workspace members
    cargo metadata --no-deps --format-version 1 2>/dev/null | \
        grep -o '"name":"[^"]*"' | \
        sed 's/"name":"//g; s/"//g' || true
}

# Check if a package uses Dioxus (has Dioxus.toml)
is_dioxus_package() {
    local package_name="$1"
    local package_path

    # Get the package path from cargo metadata
    package_path=$(cargo metadata --no-deps --format-version 1 2>/dev/null | \
        grep -o "\"manifest_path\":\"[^\"]*${package_name}/Cargo.toml\"" | \
        sed 's/"manifest_path":"//g; s/\/Cargo.toml"//g' | head -1)

    if [ -n "$package_path" ] && [ -f "${package_path}/Dioxus.toml" ]; then
        return 0
    fi
    return 1
}

# Get all Dioxus packages in the workspace
get_dioxus_packages() {
    local packages
    packages=$(get_workspace_packages)

    for pkg in $packages; do
        if is_dioxus_package "$pkg"; then
            echo "$pkg"
        fi
    done
}

# Main script
main() {
    local check_mode=true
    local target_package=""
    local all_packages=true

    # Parse arguments
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
            --check)
                check_mode=true
                shift
                ;;
            --fix)
                check_mode=false
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

    # Check dependencies
    check_rustfmt

    # Display configuration
    if [ -n "$target_package" ]; then
        print_info "Target package: $target_package"
    else
        print_info "Target: all packages"
    fi

    # Check if dx is available
    local dx_available=false
    if check_dx; then
        dx_available=true
    fi

    # Build package filter for cargo fmt
    local package_flags=""
    if [ -n "$target_package" ]; then
        package_flags="--package $target_package"
    else
        package_flags="--all"
    fi

    # Run format check or fix
    if [ "$check_mode" = true ]; then
        print_info "Checking code formatting with cargo fmt..."
        cargo fmt $package_flags -- --check

        if [ "$dx_available" = true ]; then
            print_info "Checking RSX formatting with dx fmt..."

            local packages_to_format=()

            if [ -n "$target_package" ]; then
                # Check if the target package is a Dioxus package
                if is_dioxus_package "$target_package"; then
                    packages_to_format+=("$target_package")
                else
                    print_info "  Package '$target_package' is not a Dioxus package, skipping dx fmt"
                fi
            else
                # Get all Dioxus packages
                while IFS= read -r pkg; do
                    packages_to_format+=("$pkg")
                done < <(get_dioxus_packages)
            fi

            for pkg in ${packages_to_format[@]+"${packages_to_format[@]}"}; do
                print_info "  Checking package: $pkg"
                dx fmt --package "$pkg" --check
            done
        fi

        print_success "Format check complete!"
    else
        print_info "Formatting code with cargo fmt..."
        cargo fmt $package_flags

        if [ "$dx_available" = true ]; then
            print_info "Formatting RSX with dx fmt..."

            local packages_to_format=()

            if [ -n "$target_package" ]; then
                # Check if the target package is a Dioxus package
                if is_dioxus_package "$target_package"; then
                    packages_to_format+=("$target_package")
                else
                    print_info "  Package '$target_package' is not a Dioxus package, skipping dx fmt"
                fi
            else
                # Get all Dioxus packages
                while IFS= read -r pkg; do
                    packages_to_format+=("$pkg")
                done < <(get_dioxus_packages)
            fi

            for pkg in ${packages_to_format[@]+"${packages_to_format[@]}"}; do
                print_info "  Formatting package: $pkg"
                dx fmt --package "$pkg"
            done
        fi

        print_success "Code formatted!"
    fi
}

main "$@"
