#!/usr/bin/env bash

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

show_help() {
    cat << 'EOF'
Usage: ./release.sh [OPTION]

Perform a full release: bump version, update changelog, commit, tag, push,
and create a GitHub release. Publishing to crates.io is handled automatically
by the release workflow via Trusted Publishing.

Options:
    --revision    Bump the patch version    (0.0.X)
    --minor       Bump the minor version    (0.X.0)
    --major       Bump the major version    (X.0.0)
    --dry-run     Run all steps except push, publish, and GitHub release
    --help        Display this help message

Requirements:
    - git-cliff   (cargo install git-cliff)
    - gh           (GitHub CLI, brew install gh)
    - cargo        (Rust toolchain)

EOF
}

print_info()    { echo -e "${GREEN}[INFO]${NC} $1"; }
print_error()   { echo -e "${RED}[ERROR]${NC} $1" >&2; }
print_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_step()    { echo -e "${BLUE}[STEP]${NC} $1"; }

check_dependencies() {
    local missing=0
    for cmd in git-cliff gh cargo; do
        if ! command -v "$cmd" &> /dev/null; then
            print_error "$cmd is not installed."
            missing=1
        fi
    done
    if [ "$missing" -eq 1 ]; then
        exit 1
    fi
}

check_clean_workdir() {
    if [ -n "$(git status --porcelain)" ]; then
        print_error "Working directory is not clean. Commit or stash your changes first."
        exit 1
    fi
}

check_on_main() {
    local branch
    branch=$(git rev-parse --abbrev-ref HEAD)
    if [ "$branch" != "main" ]; then
        print_error "Releases must be made from the 'main' branch (currently on '$branch')."
        exit 1
    fi
}

get_current_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

parse_version() {
    echo "$1" | sed 's/\./ /g'
}

calculate_new_version() {
    local current_version=$1
    local bump_type=$2
    local major minor patch

    # Strip any pre-release suffix (e.g. "-dev") before parsing
    local base_version="${current_version%%-*}"
    read -r major minor patch <<< "$(parse_version "$base_version")"

    case $bump_type in
        major)   major=$((major + 1)); minor=0; patch=0 ;;
        minor)   minor=$((minor + 1)); patch=0 ;;
        revision) patch=$((patch + 1)) ;;
    esac

    echo "$major.$minor.$patch"
}

update_cargo_version() {
    local new_version=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
        sed -i '' "/^lorm-macros = /s/version = \"[^\"]*\"/version = \"$new_version\"/" lorm/Cargo.toml
    else
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
        sed -i "/^lorm-macros = /s/version = \"[^\"]*\"/version = \"$new_version\"/" lorm/Cargo.toml
    fi
}

generate_changelog() {
    local new_version=$1
    git-cliff --unreleased --tag "v$new_version" -o CHANGELOG.md
}

run_tests() {
    print_step "Running tests before release..."
    cargo test --workspace
}

main() {
    if [ $# -eq 0 ]; then
        show_help
        exit 0
    fi

    local bump_type=""
    local dry_run="false"

    while [ $# -gt 0 ]; do
        case "$1" in
            --revision) bump_type="revision" ;;
            --minor)    bump_type="minor" ;;
            --major)    bump_type="major" ;;
            --dry-run)  dry_run="true" ;;
            --help|-h)  show_help; exit 0 ;;
            *)          print_error "Unknown option: $1"; show_help; exit 1 ;;
        esac
        shift
    done

    if [ -z "$bump_type" ]; then
        print_error "Version bump type is required (--revision, --minor, or --major)."
        exit 1
    fi

    check_dependencies
    check_clean_workdir
    check_on_main

    local current_version new_version
    current_version=$(get_current_version)
    new_version=$(calculate_new_version "$current_version" "$bump_type")

    echo ""
    print_info "Current version: $current_version"
    print_info "New version:     $new_version"
    if [ "$dry_run" = "true" ]; then
        print_warning "DRY RUN — no push, publish, or GitHub release will be performed"
    fi
    echo ""

    read -p "Proceed with release v${new_version}? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Release cancelled."
        exit 0
    fi

    echo ""

    print_step "1/6 Running tests..."
    run_tests
    echo ""

    print_step "2/6 Bumping version to $new_version..."
    update_cargo_version "$new_version"
    print_info "Updated Cargo.toml"

    print_step "3/6 Generating changelog..."
    generate_changelog "$new_version"
    print_info "Updated CHANGELOG.md"

    print_step "4/6 Creating release commit and tag..."
    git add Cargo.toml lorm/Cargo.toml CHANGELOG.md
    git commit -m "chore(release): prepare for v$new_version"
    git tag -a "v$new_version" -m "Release v$new_version"
    print_info "Created commit and tag v$new_version"

    if [ "$dry_run" = "true" ]; then
        echo ""
        print_step "5/6 [DRY RUN] Skipping push"
        print_step "6/6 [DRY RUN] Skipping GitHub release"
        echo ""
        print_info "Dry run complete. To finalize:"
        print_info "  git push && git push --tags"
        print_info "  gh release create v$new_version --generate-notes"
        return
    fi

    print_step "5/6 Pushing to remote..."
    git push
    git push --tags
    print_info "Pushed commit and tag"

    print_step "6/6 Creating GitHub release..."
    gh release create "v$new_version" \
        --title "v$new_version" \
        --notes-file CHANGELOG.md
    print_info "GitHub release created"

    echo ""
    print_info "Release v$new_version complete!"
    print_info ""
    print_info "The release workflow will now run on the pushed tag to:"
    print_info "  - Verify the build"
    print_info "  - Generate build attestations"
    print_info "  - Publish to crates.io via Trusted Publishing"
}

main "$@"
