# How to contribute

So, you've decided to contribute, that's great!

You can use this document to figure out how and where to start.

## Getting started

- Make sure you have a [GitHub account](https://github.com/join).
- Take a look at [existing issues](https://github.com/remysaissy/lorm/issues).
- If you need to create an issue:
    - Make sure to clearly describe it.
    - Including steps to reproduce when it is a bug.
    - Include the version of LOrm used.

## Signing Your Commits

This repository requires signed commits. We use [gitsign](https://github.com/sigstore/gitsign) for **keyless signing** via [Sigstore](https://www.sigstore.dev/), which means you don't need to manage GPG or SSH keys. You sign commits with your existing identity (GitHub, Google, or Microsoft account).

### How It Works

1. You make a commit
2. Gitsign opens a browser for OIDC authentication (GitHub/Google/Microsoft)
3. [Fulcio](https://github.com/sigstore/fulcio) issues a short-lived certificate (valid ~10 minutes)
4. Your commit is signed with that certificate
5. The signature is recorded in [Rekor](https://github.com/sigstore/rekor), a public transparency log

The certificate expires quickly, but the signature remains verifiable because Rekor proves it was created during the certificate's validity period.

### Setup

Install gitsign:

```bash
# macOS
brew install gitsign

# Go install (any platform)
go install github.com/sigstore/gitsign@latest

# Other platforms: https://github.com/sigstore/gitsign#installation
```

Configure git to use gitsign for this repository:

```bash
cd lorm

# Set gitsign as the signing program
git config --local gpg.x509.program gitsign
git config --local gpg.format x509

# Enable automatic signing for commits and tags
git config --local commit.gpgsign true
git config --local tag.gpgsign true
```

> **Tip**: Use `--global` instead of `--local` to enable gitsign for all your repositories.

### Verifying Your Setup

```bash
# Check your configuration
git config --local --list | grep -E 'gpg|sign'

# Expected output:
# gpg.x509.program=gitsign
# gpg.format=x509
# commit.gpgsign=true
# tag.gpgsign=true
```

Make a test commit. A browser window will open for authentication. After signing in, the commit is signed and the signature is logged in Rekor.

### Verifying Signatures

```bash
# Verify the latest commit
gitsign verify --certificate-identity=your-email@example.com \
  --certificate-oidc-issuer=https://github.com/login/oauth HEAD

# View signature details in the git log
git log --show-signature -1
```

### Alternative: GPG or SSH Signing

If you prefer traditional signing (e.g., in air-gapped environments), GPG and SSH signatures are also accepted. However, we recommend gitsign for its simplicity — no key management, no expiration, no revocation headaches.

### CI Verification

All pull requests are automatically checked for valid commit signatures via the `verify-signatures` CI workflow. Unsigned commits will cause the check to fail.

## Making changes

- Fork the repository on GitHub.
- Create a branch on your fork.
    - You can usually base it on the `main` branch.
    - Make sure not to commit directly to `main`.
- Make commits of logical and atomic units.
- **Sign all your commits** (see [Signing Your Commits](#signing-your-commits) above).
- **Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)** for all commit messages. This is enforced by CI on pull requests.
- Make sure you have added the necessary tests for your changes.
- Push your changes to a topic branch in your fork of the repository.
- Submit a pull request to the original repository.

### Commit Message Format

All commits must follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification:

```
<type>(<optional scope>): <description>

[optional body]

[optional footer(s)]
```

Common types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`.

Examples:
```
feat(query): add DISTINCT support to select builder
fix: prevent panic on empty WHERE clause
docs: update README with HAVING examples
chore(deps): bump sqlx to 0.8.3
```

This format is used by [git-cliff](https://git-cliff.org/) to auto-generate the changelog.

## What to work on

We try to mark issues with a suggested level of experience (in Rust/SQL).
Where possible we try to spell out how to go about implementing the feature.

To start with, check out:
- Issues labeled as ["good first issue"](https://github.com/remysaissy/lorm/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22).
- Issues labeled as ["Easy"](https://github.com/remysaissy/lorm/issues?q=is%3Aopen+is%3Aissue+label%3AE-easy).

Additionally, it's always good to work on improving/adding examples and documentation.

## Development Setup

### Prerequisites
- Rust 1.75 or later (Edition 2024)
- SQLite (for running tests)
- cargo-expand (optional, for inspecting generated code): `cargo install cargo-expand`
- act (optional, for running CI tests locally): `brew install act` (macOS) or see [act installation](https://github.com/nektos/act)
- Docker (required if using act)

### Building the Project
```bash
# Clone the repository
git clone https://github.com/remysaissy/lorm.git
cd lorm

# Build all workspace members
cargo build

# Run tests
cargo test

# Build documentation
cargo doc --open
```

### Running Examples
```bash
# Run a specific example (from workspace root)
cargo run --example basic_crud -p lorm

# List all examples
ls examples/
```

### Testing Changes

Tests are in `lorm/tests/`. When adding new features, add corresponding tests following the existing patterns in `lorm/tests/main.rs`.

See [Running Tests](#running-tests) for all test commands and coverage details.

### Inspecting Generated Code

To see what code Lorm generates, use `cargo-expand`:

```bash
# Install cargo-expand
cargo install cargo-expand

# Expand macros in tests
cd lorm
cargo expand --test main
```

This is helpful for:
- Understanding how the macro works
- Debugging macro issues
- Verifying generated code correctness

### Code Style

- Follow standard Rust formatting: `cargo fmt`
- Ensure code passes clippy: `cargo clippy -- -D warnings`
- Add documentation comments for public APIs
- Keep generated code clean and readable

### Documentation

When contributing:
- Update README.md if adding user-facing features
- Add rustdoc comments for public APIs
- Create examples for significant features
- Update CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/) format

## Communication

If you're unsure about your contribution or simply want to ask a question about anything, you can:
- Discuss something directly in the [Github issue](https://github.com/remysaissy/lorm/issues).

## Running Tests

### Unit Tests

```bash
cargo test                              # Run all tests
cargo test test_user_is_created         # Run a specific test
cargo test -- --nocapture               # Run with stdout output
```

### Using the Helper Scripts

```bash
./test.sh                # Run unit tests (default features)
./test.sh --all-features # Run with all feature flags
./format.sh --check      # Check formatting (rustfmt)
./format.sh --fix        # Auto-fix formatting
./check.sh               # Run clippy with deny-warnings
./check.sh --all-features
```

### Code Coverage

```bash
cargo install cargo-llvm-cov            # One-time setup

./coverage.sh                           # HTML report (opens in browser)
./coverage.sh --lcov                    # LCOV report for CI
./coverage.sh --text                    # Terminal summary
./coverage.sh --check-thresholds        # Verify ≥ 80% coverage
```

CI enforces **≥ 80%** line, region, and function coverage. PRs that reduce coverage below these thresholds will fail.

## Bumping the Version

To bump the version **without** performing a full release (e.g., to prepare a version bump commit):

```bash
./bump-version.sh --revision   # 0.2.2 → 0.2.3
./bump-version.sh --minor      # 0.2.2 → 0.3.0
./bump-version.sh --major      # 0.2.2 → 1.0.0
```

This updates `Cargo.toml` (workspace version) and regenerates `CHANGELOG.md` via [git-cliff](https://git-cliff.org/), then stages both files. You still need to commit, tag, and push manually.

**Prerequisite**: `cargo install git-cliff`

## Releasing

To perform a full release (version bump → changelog → commit → tag → push → publish → GitHub release):

```bash
./release.sh --revision       # Patch release
./release.sh --minor          # Minor release
./release.sh --major          # Major release
./release.sh --dry-run --revision  # Preview without side effects
```

The release script performs these steps in order:

1. Runs `cargo test --workspace` to verify nothing is broken
2. Bumps the version in `Cargo.toml`
3. Regenerates `CHANGELOG.md` with git-cliff
4. Creates a signed commit (`chore(release): prepare for vX.Y.Z`)
5. Creates an annotated tag (`vX.Y.Z`)
6. Pushes the commit and tag to `origin`
7. Publishes `lorm-macros` then `lorm` to crates.io (in dependency order)
8. Creates a GitHub release with the changelog as release notes

After the tag is pushed, the `release.yml` CI workflow runs automatically to verify the build and generate [GitHub Artifact Attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations) linking the crate packages to the specific commit and workflow that produced them.

**Prerequisites**: `git-cliff`, `gh` (GitHub CLI), `cargo login` or `CARGO_REGISTRY_TOKEN` env var.

Use `--dry-run` to validate everything locally (runs tests, bumps version, creates commit and tag) without pushing, publishing, or creating a release.

## Repository Rules

This repository enforces the following policies via GitHub branch protection and CI:

- **Signed commits required**: All commits pushed to `main` (and included in pull requests) must be cryptographically signed. We recommend [gitsign](#signing-your-commits) for keyless signing, but GPG and SSH signatures are also accepted.
- **CI must pass**: Format, clippy, tests, coverage (≥ 80%), and signature verification checks must all pass before merging.
- **Vigilant mode**: Repository maintainers use GitHub's vigilant mode, which marks all unsigned commits as "Unverified."

If you're a first-time contributor and need help setting up commit signing, feel free to open an issue — we're happy to help.

## Code of Conduct

Be respectful, constructive, and welcoming to all contributors.