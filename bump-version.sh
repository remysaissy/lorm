#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

show_help() {
    cat << 'EOF'
Usage: ./bump-version.sh COMMAND [OPTIONS]

Manage project version following the X.Y.Z-dev → X.Y.Z release cycle.

COMMANDS

  --post-release
        Add -dev suffix to the current version immediately after a release.
        Does NOT bump the version number — the suffix signals "unreleased work
        on top of X.Y.Z". No CHANGELOG change.

        Example: 0.3.0  →  0.3.0-dev

  --release [--minor | --major | --revision]
        Prepare a release commit. Strips -dev, derives the next version from
        conventional commits via git-cliff --bump, and finalises CHANGELOG.
        Pass an explicit bump flag to override the auto-derived type.

        Examples:
          ./bump-version.sh --release              # auto-derived from commits
          ./bump-version.sh --release --minor      # force minor bump
          ./bump-version.sh --release --major      # force major bump
          ./bump-version.sh --release --revision   # force patch bump

        Input → output (auto-derived):
          0.3.0-dev + only fix: commits  →  0.3.1
          0.3.0-dev + feat: commit       →  0.4.0
          0.3.0-dev + feat!: commit      →  1.0.0

  --revision
        Mid-cycle patch re-plan. Keeps -dev suffix.
        Example: 0.3.0-dev  →  0.3.1-dev

  --minor
        Mid-cycle minor re-plan. Keeps -dev suffix.
        Example: 0.3.0-dev  →  0.4.0-dev

  --major
        Mid-cycle major re-plan. Keeps -dev suffix.
        Example: 0.3.0-dev  →  1.0.0-dev

  --help
        Show this message.

TYPICAL LIFECYCLE

  # Immediately after releasing v0.3.0:
  ./bump-version.sh --post-release
  git commit -m 'chore: begin development on next release'
  git push

  # (All feature work lands on main; version stays 0.3.0-dev throughout)

  # Ready to ship:
  ./bump-version.sh --release           # or: --release --minor to override
  git commit -m 'chore(release): prepare for vX.Y.Z'
  # open PR → merge to main → tag → push tag

REQUIREMENTS
  git-cliff must be installed: cargo install git-cliff
EOF
}

print_info()    { echo -e "${GREEN}[INFO]${NC} $1"; }
print_error()   { echo -e "${RED}[ERROR]${NC} $1" >&2; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }

check_dependencies() {
    if ! command -v git-cliff &>/dev/null; then
        print_error "git-cliff is not installed."
        echo "Install with: cargo install git-cliff"
        exit 1
    fi
}

get_raw_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

get_base_version() {
    get_raw_version | sed 's/-dev$//'
}

parse_version() {
    echo "$1" | sed 's/\./ /g'
}

apply_bump() {
    local base="$1" type="$2"
    read -r major minor patch <<< "$(parse_version "$base")"
    case "$type" in
        major)    echo "$((major + 1)).0.0" ;;
        minor)    echo "$major.$((minor + 1)).0" ;;
        revision) echo "$major.$minor.$((patch + 1))" ;;
        *) print_error "Unknown bump type: $type"; exit 1 ;;
    esac
}

update_cargo_version() {
    local ver="$1"
    if [[ "$OSTYPE" == darwin* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$ver\"/" Cargo.toml
    else
        sed -i "s/^version = \".*\"/version = \"$ver\"/" Cargo.toml
    fi
    # Regenerate Cargo.lock to keep it consistent with the new version.
    cargo metadata --format-version 1 --no-deps > /dev/null 2>&1 || true
    print_info "Cargo.toml → version = \"$ver\""
}

update_changelog() {
    local tag="$1"
    print_info "Generating CHANGELOG.md for $tag …"
    if git-cliff --unreleased --tag "$tag" -o CHANGELOG.md; then
        print_info "CHANGELOG.md updated."
    else
        print_error "Failed to generate CHANGELOG.md"
        exit 1
    fi
}

confirm() {
    read -rp "$1 (y/N) " -n 1
    echo
    [[ "$REPLY" =~ ^[Yy]$ ]]
}

# ─────────────────────────────────────────────────────────────────────────────
# Command implementations
# ─────────────────────────────────────────────────────────────────────────────

cmd_post_release() {
    local raw; raw=$(get_raw_version)

    if [[ "$raw" == *-dev ]]; then
        print_warning "Version is already a dev version: $raw"
        exit 0
    fi

    local new_ver="${raw}-dev"
    confirm "Add -dev suffix: $raw → $new_ver?" \
        || { print_warning "Cancelled."; exit 0; }

    update_cargo_version "$new_ver"
    git add Cargo.toml Cargo.lock

    print_info ""
    print_info "Done. Next step:"
    print_info "  git commit -m 'chore: begin development on next release'"
}

cmd_dev_bump() {
    local type="$1"
    local raw; raw=$(get_raw_version)
    local base; base=$(get_base_version)
    local new_ver; new_ver="$(apply_bump "$base" "$type")-dev"

    confirm "Mid-cycle re-plan: $raw → $new_ver?" \
        || { print_warning "Cancelled."; exit 0; }

    update_cargo_version "$new_ver"
    git add Cargo.toml Cargo.lock

    print_info ""
    print_info "Done. Next step:"
    print_info "  git commit -m 'chore: re-plan next release as $new_ver'"
}

cmd_release() {
    local bump_override="$1"

    check_dependencies

    local raw; raw=$(get_raw_version)
    local base; base=$(get_base_version)

    if [[ "$raw" != *-dev ]]; then
        print_warning "Current version '$raw' has no -dev suffix."
        confirm "Proceed anyway?" || { print_warning "Cancelled."; exit 0; }
    fi

    local new_ver
    if [[ -n "$bump_override" ]]; then
        new_ver=$(apply_bump "$base" "$bump_override")
        print_info "Using explicit bump ($bump_override): $base → $new_ver"
    else
        local cliff_out
        cliff_out=$(git-cliff --bumped-version 2>/dev/null || true)
        new_ver="${cliff_out#v}"                  # strip leading 'v' if present
        new_ver="${new_ver//[[:space:]]/}"         # trim whitespace

        if [[ -z "$new_ver" ]]; then
            print_error "git-cliff --bumped-version returned nothing."
            print_error "Specify the bump explicitly: --release --minor | --major | --revision"
            exit 1
        fi
        print_info "git-cliff derived next version: $new_ver"
    fi

    confirm "Release: $raw → $new_ver (will tag v$new_ver)?" \
        || { print_warning "Cancelled."; exit 0; }

    update_cargo_version "$new_ver"
    update_changelog "v$new_ver"
    git add Cargo.toml Cargo.lock CHANGELOG.md

    print_info ""
    print_info "Release prepared. Next steps:"
    print_info "  1. Review staged changes : git diff --cached"
    print_info "  2. Commit                : git commit -m 'chore(release): prepare for v$new_ver'"
    print_info "  3. Open PR → merge to main"
    print_info "  4. Tag on main           : git tag -a v$new_ver -m 'Release v$new_ver'"
    print_info "  5. Push tag              : git push && git push --tags"
    print_info "  6. After release         : ./bump-version.sh --post-release"
}

# ─────────────────────────────────────────────────────────────────────────────
# Argument parsing
# ─────────────────────────────────────────────────────────────────────────────

main() {
    if [[ $# -eq 0 ]]; then
        show_help
        exit 0
    fi

    case "$1" in
        --post-release)
            cmd_post_release
            ;;
        --release)
            local bump_override=""
            if [[ $# -gt 1 ]]; then
                case "$2" in
                    --revision) bump_override="revision" ;;
                    --minor)    bump_override="minor"    ;;
                    --major)    bump_override="major"    ;;
                    *)
                        print_error "Unknown option for --release: $2"
                        echo ""
                        show_help
                        exit 1
                        ;;
                esac
            fi
            cmd_release "$bump_override"
            ;;
        --revision) cmd_dev_bump "revision" ;;
        --minor)    cmd_dev_bump "minor"    ;;
        --major)    cmd_dev_bump "major"    ;;
        --help|-h)  show_help ;;
        *)
            print_error "Unknown option: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

main "$@"
