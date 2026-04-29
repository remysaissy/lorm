# Learnings — finish-mrpine-vision

## [2026-04-29] Initial Setup

### Project Structure
- `lorm-macros/src/` — proc macro crate
  - `attributes.rs` — darling-parsed attribute structs
  - `models.rs` — ORM model parsing (`OrmModel::from_fields`)
  - `orm/{by,with,select,delete,save,column}.rs` — code generators
  - `utils.rs` — shared helpers (bind type constraints, placeholders)
  - `lib.rs` — macro entry point
- `lorm/tests/main.rs` — integration tests
- `lorm/tests/resources/migrations/{sqlite,postgres,mysql}/` — migration files
- `examples/` — runnable examples

### Key Conventions
- Use `syn::Error::new()` and `syn::Error::combine()` — NO `proc-macro-error2`
- No explicit `proc-macro2` in `lorm-macros/Cargo.toml`
- darling 0.23.0 is pinned
- Conventional commits required; signed commits required
- `./format.sh && ./check.sh --package=lorm --feature=sqlite` before each commit
- `./test.sh --package=lorm --feature=sqlite` before each push
- Sequential PRs: each PR must be merged before next task starts

### Branch Sequence
T1: chore/cleanup-and-fixes → T2: feat/is-set-callable → T3: feat/sqlx-json → T4: feat/sqlx-flatten → T5: feat/composite-pk → T6: feat/manual-pk-upsert → T7: chore/release-0.3.0 → F1-F4 final review
