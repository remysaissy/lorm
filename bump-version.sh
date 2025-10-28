#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to display help
show_help() {
    cat << EOF
Usage: $0 [OPTION]

Bump the project version and update the CHANGELOG using git-cliff.

Options:
    --revision    Bump the revision/patch version (0.0.X)
    --minor       Bump the minor version (0.X.0)
    --major       Bump the major version (X.0.0)
    --help        Display this help message

Examples:
    $0 --revision    # 0.0.9 -> 0.0.10
    $0 --minor       # 0.0.9 -> 0.1.0
    $0 --major       # 0.0.9 -> 1.0.0

Note: This script will:
  1. Update the version in Cargo.toml (workspace)
  2. Generate/update CHANGELOG.md using git-cliff
  3. Stage both files for commit (you need to commit manually)

Requirements:
  - git-cliff must be installed (cargo install git-cliff)

EOF
}

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

# Check if git-cliff is installed
check_dependencies() {
    if ! command -v git-cliff &> /dev/null; then
        print_error "git-cliff is not installed."
        echo "Please install it with: cargo install git-cliff"
        exit 1
    fi
}

# Get current version from Cargo.toml
get_current_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Parse version components
parse_version() {
    local version=$1
    echo "$version" | sed 's/\./ /g'
}

# Bump version based on type
bump_version() {
    local current_version=$1
    local bump_type=$2

    read -r major minor patch <<< "$(parse_version "$current_version")"

    case $bump_type in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        revision)
            patch=$((patch + 1))
            ;;
        *)
            print_error "Unknown bump type: $bump_type"
            exit 1
            ;;
    esac

    echo "$major.$minor.$patch"
}

# Update version in Cargo.toml
update_cargo_version() {
    local new_version=$1
    local cargo_file="Cargo.toml"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$cargo_file"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$cargo_file"
    fi

    print_info "Updated $cargo_file to version $new_version"
}

# Generate changelog using git-cliff
update_changelog() {
    local new_version=$1

    print_info "Generating CHANGELOG.md using git-cliff..."

    if git-cliff --unreleased --tag "v$new_version" -o CHANGELOG.md; then
        print_info "CHANGELOG.md updated successfully"
    else
        print_error "Failed to generate CHANGELOG.md"
        exit 1
    fi
}

# Main script
main() {
    # Check if no arguments provided
    if [ $# -eq 0 ]; then
        show_help
        exit 0
    fi

    # Parse arguments
    bump_type=""

    case "$1" in
        --revision)
            bump_type="revision"
            ;;
        --minor)
            bump_type="minor"
            ;;
        --major)
            bump_type="major"
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

    # Check dependencies
    check_dependencies

    # Get current version
    current_version=$(get_current_version)
    print_info "Current version: $current_version"

    # Calculate new version
    new_version=$(bump_version "$current_version" "$bump_type")
    print_info "New version: $new_version"

    # Confirm with user
    read -p "Do you want to proceed with version bump from $current_version to $new_version? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Version bump cancelled"
        exit 0
    fi

    # Update Cargo.toml
    update_cargo_version "$new_version"

    # Update CHANGELOG.md
    update_changelog "$new_version"

    # Stage files
    print_info "Staging updated files..."
    git add Cargo.toml CHANGELOG.md

    print_info ""
    print_info "Version bump complete!"
    print_info ""
    print_info "Next steps:"
    print_info "  1. Review the changes: git diff --cached"
    print_info "  2. Commit the changes: git commit -m 'chore(release): prepare for v$new_version'"
    print_info "  3. Create a git tag: git tag -a v$new_version -m 'Release v$new_version'"
    print_info "  4. Push changes: git push && git push --tags"
}

main "$@"
