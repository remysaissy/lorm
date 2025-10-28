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

## Making changes

- Fork the repository on GitHub.
- Create a branch on your fork.
    - You can usually base it on the `main` branch.
    - Make sure not to commit directly to `main`.
- Make commits of logical and atomic units.
- Make sure you have added the necessary tests for your changes.
- Push your changes to a topic branch in your fork of the repository.
- Submit a pull request to the original repository.

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
# Run a specific example
cargo run --example basic_crud

# List all examples
ls examples/
```

### Testing Changes

Lorm has tests in the `lorm/tests` directory that cover various features:

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_user_is_created

# Run tests with output
cargo test -- --nocapture
```

When adding new features, please add corresponding tests following the existing patterns in `lorm/tests/main.rs`.

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

## Code of Conduct

Be respectful, constructive, and welcoming to all contributors.