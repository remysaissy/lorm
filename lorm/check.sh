#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory for resolving paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

Run clippy and dx check to validate Rust code and Dioxus configuration.

Options:
    --package <name>            Run checks only on specified package (also: --package=<name>)
    --all-packages              Run checks on all packages (default)
    --no-default-features       Disable default features
    --features <name>           Enable specific feature(s) - comma-separated (also: --features=<name>)
    --all-features              Enable all features (default)
    --deny-warnings             Treat warnings as errors (default in CI)
    --fix                       Automatically apply clippy suggestions
    --help                      Display this help message

Examples:
    $0                                          # Check all packages with all features
    $0 --all-packages                           # Check all packages
    $0 --package outreach-web                   # Check only outreach-web package
    $0 --package=outreach-web                   # Same as above (POSIX style)
    $0 --package=outreach-web --all-features    # Check outreach-web with all features
    $0 --no-default-features                    # Check with no features
    $0 --no-default-features --features=my_feature
                                                # Check with only my_feature
    $0 --features=feature1,feature2             # Check with multiple features
    $0 --fix                                    # Auto-fix clippy suggestions

Requirements:
  - clippy must be installed (rustup component add clippy)
  - dx (Dioxus CLI) should be installed for Dioxus checks (cargo install dioxus-cli)

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

# Get the directory path for a package
get_package_dir() {
    local package_name="$1"
    cargo metadata --no-deps --format-version 1 2>/dev/null | \
        grep -o "\"manifest_path\":\"[^\"]*\"" | \
        grep "/${package_name}/Cargo.toml" | \
        sed 's/"manifest_path":"//g; s/\/Cargo.toml"//g' | head -1
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
    local feature_flags="--all-features"
    local no_default_features=false
    local deny_warnings=true
    local fix_mode=false
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
            --no-default-features)
                no_default_features=true
                feature_flags=""
                shift
                ;;
            --features=*)
                local feat_value="${1#*=}"
                if [ -z "$feature_flags" ] || [ "$feature_flags" = "--all-features" ]; then
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
            --deny-warnings)
                deny_warnings=true
                shift
                ;;
            --fix)
                fix_mode=true
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

    # Build cargo flags
    local cargo_flags=""
    if [ "$no_default_features" = true ]; then
        cargo_flags="--no-default-features"
    fi
    if [ -n "$feature_flags" ]; then
        cargo_flags="$cargo_flags $feature_flags"
    fi

    # Add package filter if specified
    local package_flags=""
    if [ -n "$target_package" ]; then
        package_flags="--package $target_package"
    fi

    # Check dependencies
    check_clippy

    # Display configuration
    if [ -n "$target_package" ]; then
        print_info "Target package: $target_package"
    else
        print_info "Target: all packages"
    fi

    if [ -n "$cargo_flags" ]; then
        print_info "Cargo flags: $cargo_flags"
    else
        print_info "Using default features"
    fi

    if [ "$deny_warnings" = true ]; then
        print_info "Treating warnings as errors"
    fi

    if [ "$fix_mode" = true ]; then
        print_info "Auto-fix mode enabled"
    fi

    # Check if dx is available
    local dx_available=false
    if check_dx; then
        dx_available=true
    fi

    # Run clippy
    if [ "$fix_mode" = true ]; then
        print_info "Running clippy with auto-fix..."
        if [ "$deny_warnings" = true ]; then
            cargo clippy --fix --allow-dirty --allow-staged $package_flags $cargo_flags -- -D warnings
        else
            cargo clippy --fix --allow-dirty --allow-staged $package_flags $cargo_flags
        fi
    else
        print_info "Running clippy..."
        if [ "$deny_warnings" = true ]; then
            cargo clippy $package_flags $cargo_flags -- -D warnings
        else
            cargo clippy $package_flags $cargo_flags
        fi
    fi

    # Run dx check if available (only for Dioxus packages)
    # Note: mobile packages are excluded as they require --ios or --android platform flag
    if [ "$dx_available" = true ]; then
        print_info "Running dx check on Dioxus packages..."

        local packages_to_check=()

        if [ -n "$target_package" ]; then
            # Check if the target package is a Dioxus package
            if is_dioxus_package "$target_package"; then
                packages_to_check+=("$target_package")
            else
                print_info "  Package '$target_package' is not a Dioxus package, skipping dx check"
            fi
        else
            # Get all Dioxus packages
            while IFS= read -r pkg; do
                packages_to_check+=("$pkg")
            done < <(get_dioxus_packages)
        fi

        for pkg in ${packages_to_check[@]+"${packages_to_check[@]}"}; do
            # Skip mobile packages as they require platform-specific flags
            if [[ "$pkg" == *"-mobile"* ]] || [[ "$pkg" == *"mobile"* ]]; then
                print_info "  Skipping mobile package: $pkg (requires --ios or --android)"
                continue
            fi
            print_info "  Checking package: $pkg"
            dx check --package "$pkg"
        done
    fi

    print_success "All checks complete!"
}

main "$@"
