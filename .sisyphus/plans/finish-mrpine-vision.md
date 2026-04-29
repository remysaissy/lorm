# Finish MrPine PR #2 Vision (lorm 0.3.0)

## TL;DR

> **Quick Summary**: MrPine's PR #2 proposed 7 conceptual changes to the lorm crate. He delivered 3 atomic PRs (#6 darling, #7 sqlx attrs, #8 query argument types — all merged). This plan finishes the remaining accepted-but-unbuilt items: composite primary keys, `#[sqlx(json)]` support, `#[sqlx(flatten)]` wiring, conditional upsert (manual pk only), the `is_set` Callable migration, plus owner-mandated cleanups, culminating in a 0.3.0 release.
>
> **Deliverables**:
> - Owner-required cleanups: fix `updated_at` error message, add `#[automatically_derived]`, scrub dead code, clean examples
> - BREAKING: `is_set` migrates from `syn::Expr` to `darling::util::Callable` (e.g. `is_set = "Uuid::is_nil"`)
> - `#[sqlx(json)]` support — bare form only
> - `#[sqlx(flatten)]` + `#[lorm(flattened(field: Ty, field2: Ty2 = "renamed_col", ...))]` wiring with `Option<T>` support
> - Composite primary keys via `#[lorm(pk_type = "manual")]` + multiple `#[lorm(pk)]` + optional `#[lorm(pk_selector = "...")]`
> - Conditional upsert: `Manual` pk_type only; postgres/sqlite `ON CONFLICT (..) DO UPDATE`, mysql `ON DUPLICATE KEY UPDATE`, all-pk edge case → `DO NOTHING` / `INSERT IGNORE`
> - 0.3.0 release: bumped Cargo workspace version, regenerated CHANGELOG via `bump-version.sh --minor`, updated README + examples
>
> **Estimated Effort**: Large
> **Parallel Execution**: NO — sequential feature branches (per user's PR organization choice). Each branch is its own PR upstream and must merge before the next starts.
> **Critical Path**: T1 (cleanup) → T2 (is_set Callable) → T3 (json) → T4 (flatten) → T5 (composite pk) → T6 (manual upsert) → T7 (release) → F1-F4 (final review) → user okay

---

## Context

### Original Request
> "Analyze the https://github.com/remysaissy/lorm/pull/2 discussion with MrPine. Ideas were good but he had only provided a single 3 small PRs from this discussion. Create a plan to finish implementing the full vision discussed that I've agreed upon."

The user is the owner of `remysaissy/lorm`. PR #2 was MrPine's exploratory "Various Changes" PR. The owner reviewed it on 2026-03-20 and split the response into "what I'd accept", "what I'd need modified", and "small fixes needed". MrPine agreed and shipped #6/#7/#8 in March; the rest of the agreed-upon work has not been done. This plan finishes it.

### Interview Summary

**Key Discussions**:
- Already-merged (no rework): PR #6 (darling), PR #7 (sqlx-attrs), PR #8 (column-value-type bounds)
- Outstanding from MrPine vision: items 3 (upsert), 5 (json), 6 (flatten), 7 (composite pk) + scattered cleanups
- Owner's concession on item 3 (upsert): keep INSERT/UPDATE for `pk_type = "generated"`; use upsert ONLY for `pk_type = "manual"` because composite-pk save() has no other way to disambiguate INSERT vs UPDATE — owner explicitly approved this in PR thread comment 4104296957
- BREAKING: `is_set` migration from `Expr` (`is_set = "is_nil()"`) to `Callable` (`is_set = "Uuid::is_nil"`). Documented in CHANGELOG with migration block.
- Single-field `pk_type = "manual"` generates `by_<field>` (NOT `by_key`) — minimal API surprise vs. composite which uses `by_key` (overridable via `pk_selector`)
- JSON support: bare `#[sqlx(json)]` only, no `#[sqlx(json(nullable))]` extension
- Flatten supports both `T` and `Option<T>` nested structs

**Research Findings**:
- MrPine's full reference impl is locally fetched as branch `mrpine-big` for design pattern reference (NOT to copy verbatim — owner constraints differ)
- Current code at `lorm-macros/src/orm/column.rs` already has `is_flattened: bool` field on `Column` — scaffolding kept from earlier MrPine merges; never set true today
- darling 0.23.0 is already pinned, Cargo.lock already committed
- `lorm-macros/src/models.rs:41` enforces "exactly one primary key" — must relax to `>= 1` when `pk_type = "manual"`
- CI matrix (`.github/workflows/ci.yml`) already exercises sqlite + postgres + mysql via service containers
- All scripts (`./check.sh`, `./test.sh`, `./format.sh`, `./coverage.sh`) accept `--package=lorm --feature=<backend>` flags
- `bump-version.sh --minor` + git-cliff are already wired for the release commit

### Self Gap-Analysis (Metis returned empty; performed manually)

**Edge cases addressed via guardrails:**
- Empty struct + manual pk → ERROR "at least one #[lorm(pk)] required when pk_type=manual"
- pk field that is also flattened → ERROR (rejected)
- json-typed pk → forbidden (binding semantics conflict)
- composite pk where one field is `updated_at`/`created_at` → ERROR (timestamps cannot be part of pk)
- Manual pk + `is_set`/`new` attribute on a pk field → ERROR (these only make sense in `Generated` mode)
- `is_full_key` in upsert (every column is a pk column → empty SET clause): postgres/sqlite use `ON CONFLICT (...) DO NOTHING`, mysql uses `INSERT IGNORE INTO`. Returning-equivalent fetch must still happen.
- MySQL `ON DUPLICATE KEY UPDATE` does not support `RETURNING` — fall back to `SELECT WHERE pk_cols = ...` after `INSERT` (already the existing MySQL pattern; reuse `Copy` executor bound)
- Flatten + Option<T>: when nested struct is None, all flattened bind values must be NULL (use `as_ref().map(|b| &b.field)`)
- Composite pk + `delete()`: WHERE clause must AND all pk columns
- Composite pk + `by_key()`: function takes (executor, pk1, pk2, …) parameters in declaration order
- Cross-feature: flatten fields can be `#[lorm(by)]` → must generate `by_<field>` even though field lives nested
- Cross-feature: json field can be `#[lorm(by)]` → query bind must wrap `value` in `sqlx::types::Json(value)`

---

## Work Objectives

### Core Objective
Complete the agreed-upon design from PR #2 review across 7 sequential feature branches, each landing as its own GitHub PR with conventional commits, culminating in a tagged 0.3.0 release.

### Concrete Deliverables
- 7 feature branches, each with its own GitHub PR (see TODOs)
- All 3 backend feature flags (`sqlite`, `postgres`, `mysql`) compile, lint, and pass tests at every PR
- README.md attribute reference reflects every new attribute and option
- CHANGELOG.md `## [0.3.0]` section enumerates Added / Changed / Fixed / **BREAKING CHANGES**
- `lorm/tests/main.rs` (and resource migrations) cover composite-pk, flattened, and json models
- `examples/` includes a composite-pk example
- `Cargo.toml` workspace version bumped to 0.3.0
- All 3 backends green in CI, including coverage thresholds (80% per `coverage.sh --check-thresholds`)

### Definition of Done
- [ ] `gh pr list --repo remysaissy/lorm --state merged --limit 10` shows the 7 PRs from this plan, all merged
- [ ] `git tag --list "v0.3.0"` returns `v0.3.0` (after release commit + tag)
- [ ] `cargo build --workspace --no-default-features --features sqlite` exits 0
- [ ] Same for `--features postgres` and `--features mysql`
- [ ] `cargo test --no-default-features --features sqlite -p lorm` passes (all old tests + new tests for composite/flatten/json)
- [ ] CI matrix passes for sqlite/postgres/mysql on the final merge to main
- [ ] `cargo run --example basic_crud -p lorm`, `query_builder`, `transactions`, and a new `composite_pk` example all run successfully
- [ ] No `proc-macro-error2` or `proc-macro2` (explicit) in `lorm-macros/Cargo.toml`
- [ ] `grep -r "is_set = \"is_nil()\"" lorm/ examples/` returns nothing (migration complete)

### Must Have
- All 4 unimplemented MrPine items (composite pk, flatten, json, conditional upsert) shipped
- BREAKING `is_set` migration shipped with explicit CHANGELOG migration block
- `#[automatically_derived]` on every generated `impl` block
- `updated_at` error message bug fixed in `lorm-macros/src/models.rs`
- All 3 SQLx backends supported for every new feature
- Conventional commits throughout; one feature per branch/PR
- `syn::Error::combine()` (NOT `proc-macro-error2`) for any multi-error accumulation
- Cargo.lock committed at every change (per supply-chain auditing rule)

### Must NOT Have (Guardrails)
- ❌ NO `proc-macro-error2` dependency (dormant since Sept 2024)
- ❌ NO explicit `proc-macro2` in `lorm-macros/Cargo.toml` (transitively included)
- ❌ NO upsert for `pk_type = "generated"` (breaks Active Record semantics; owner forbade)
- ❌ NO MySQL `ON CONFLICT` syntax (use `ON DUPLICATE KEY UPDATE`)
- ❌ NO `log` / `test-log` runtime dependencies (dev-deps only if added at all)
- ❌ NO commented-out code in examples
- ❌ NO unused test helpers (e.g., dead `create_pairings()` if found)
- ❌ NO multi-plan splits — this is ONE plan even though it produces 7 PRs (per Single Plan Mandate)
- ❌ NO copy-paste of MrPine's code verbatim — adapt to project conventions (`Column` not `LogicalField`, no `proc_macro_error2`, heck not Inflector)
- ❌ NO scope creep: no new query operators, no migrations, no relationships/joins
- ❌ NO `#[sqlx(json(nullable))]` extension this round
- ❌ NO new dependencies beyond what's already in `Cargo.toml` (darling already at 0.23.0 ✓)

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.
> Acceptance criteria requiring "user manually tests/confirms" are FORBIDDEN.

### Test Decision
- **Infrastructure exists**: YES — cargo test + sqlx + tokio. Migrations under `lorm/tests/resources/migrations/{sqlite,postgres,mysql}/`.
- **Automated tests**: YES (TDD per task)
- **Framework**: cargo test
- **Per task**: each feature task adds at least one model + migration trio (sqlite/postgres/mysql) + at least one test asserting behavior

### QA Policy
Every task MUST include agent-executed QA scenarios. Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

> **Evidence Directory Precondition**: Before running any QA scenario, ensure the evidence directory exists:
> ```bash
> mkdir -p .sisyphus/evidence
> ```
> Each task's scenarios may also create subdirectories (e.g., `.sisyphus/evidence/task-4-bad-flatten-pk/`). The `mkdir -p` in each scenario step handles those.

- **Library/Macro behavior**: Use Bash with `cargo build`, `cargo test`, `cargo expand`, `cargo clippy`. Capture stdout/stderr to evidence file. For macro expansion sanity-checks, save expanded code to evidence files.
- **CLI scripts**: Use Bash to run `./check.sh`, `./test.sh`, `./format.sh --check`, `./coverage.sh --check-thresholds`.
- **PR creation**: Use Bash + `gh pr create`. Capture PR URL.
- **CI verification**: Use Bash + `gh pr checks {url} --watch`. Save final status JSON.

---

## Execution Strategy

### Sequential Branch Sequence (MANDATORY — user's choice)

> User chose: "One plan, multiple feature branches" — each task that produces a PR MUST wait for upstream merge before the next task starts. This honors the owner's request to MrPine for "smaller PRs using conventional commits".

```
T1: chore/cleanup-and-fixes        → PR #1 → merge → main
T2: feat/is-set-callable           → PR #2 → merge → main  [BREAKING]
T3: feat/sqlx-json                 → PR #3 → merge → main
T4: feat/sqlx-flatten              → PR #4 → merge → main
T5: feat/composite-pk              → PR #5 → merge → main  [BREAKING]
T6: feat/manual-pk-upsert          → PR #6 → merge → main
T7: chore/release-0.3.0            → PR #7 → merge → main → tag v0.3.0
F1-F4: Final verification (parallel after T7) → user okay

Critical Path: T1 → T2 → T3 → T4 → T5 → T6 → T7 → F1-F4 → user okay
Parallel Speedup: NONE intentionally (sequential PRs per user choice)
Max Concurrent: 1 task at a time + 4 final review agents in parallel
```

### Dependency Matrix

- **T1**: blocks T2, T3, T4, T5, T6, T7
- **T2**: depends T1 — blocks T3, T4, T5, T6, T7
- **T3**: depends T2 — blocks T4, T5, T6, T7
- **T4**: depends T3 — blocks T5, T6, T7
- **T5**: depends T4 — blocks T6, T7
- **T6**: depends T5 — blocks T7
- **T7**: depends T6 — blocks F1-F4
- **F1, F2, F3, F4**: depend T7 — blocks final user okay

### Agent Dispatch Summary

- **T1** → `quick` (small atomic fixes)
- **T2** → `unspecified-high` (proc-macro refactor; touches multiple files)
- **T3** → `unspecified-high` (macro feature with binding semantics)
- **T4** → `deep` (most complex: parses arbitrary nested struct field lists; affects all generated query methods)
- **T5** → `deep` (ORM core refactor: PrimaryKey enum across save/delete/by/with/select/save)
- **T6** → `deep` (3-backend SQL dialect handling)
- **T7** → `quick` (mechanical bumping)
- **F1** → `oracle` (plan compliance)
- **F2** → `unspecified-high` (code quality)
- **F3** → `unspecified-high` (real manual QA)
- **F4** → `deep` (scope fidelity)

---

## TODOs

> Implementation + Test = ONE Task. Never separate.
> Each task IS one feature branch IS one PR. Land it upstream before starting the next.

- [ ] 1. **chore/cleanup-and-fixes — small fixes called out in PR #2 review**

  **What to do**:
  - Branch from main: `git checkout -b chore/cleanup-and-fixes`
  - **Fix `updated_at` error message bug**: in `lorm-macros/src/models.rs` (or wherever the duplicate-field check lives in current code; see References), the duplicate-field check for `updated_at` says "Only one field can hold the #[lorm(created_at)] attribute". Add the same check for `updated_at` if missing, OR fix the wrong message. Verify with: search current code via `grep -n "Only one field can hold" lorm-macros/src/`. Note: current `models.rs` does NOT yet have a duplicate-`created_at`/`updated_at` check (only enforces single pk); ADD a duplicate-detection check for both timestamps using `syn::Error::combine()`.
  - **Add `#[automatically_derived]` to all generated `impl` blocks** in `lorm-macros/src/orm/{by,with,select,delete,save}.rs`. Apply to every `impl<...> Foo for Bar { ... }` produced by the macro. Trait definitions stay unchanged.
  - **Remove dead code**: search the codebase for unused helpers. Run `cargo clippy --workspace --no-default-features --features sqlite -- -W dead_code` and `grep -rn "create_pairings\|fn unused_" lorm/ lorm-macros/` to find leftovers. Remove anything that hits.
  - **Clean examples**: scan `examples/{basic_crud,query_builder,transactions,test}.rs` for commented-out code blocks (lines beginning `// let`, `// fn`, `// pool.`, etc. for non-explanatory comments). Delete commented-out code; keep doc comments.
  - **Verify deps**: confirm `lorm-macros/Cargo.toml` does NOT contain a top-level `proc-macro2 = ...` line (only via syn/quote transitively). It already doesn't (verified at `lorm-macros/Cargo.toml:21-26`) — make this an explicit acceptance assertion.
  - **CHANGELOG**: append entries under `## [Unreleased]` `### Fixed` for the error message and `### Changed` for `#[automatically_derived]`.
  - **Commits** (atomic):
    - `fix(macros): correct updated_at duplicate-field check error message`
    - `feat(macros): add #[automatically_derived] to generated impl blocks`
    - `chore: remove dead code and commented-out examples`
    - `docs(changelog): add unreleased entries for cleanup work`
  - Push branch: `git push -u origin chore/cleanup-and-fixes`
  - Create PR via `gh pr create` with title `chore: pre-0.3.0 cleanup and fixes from PR #2 review`. Body includes a checklist mapping to PR #2 review's "Small fixes needed" section.
  - Wait for CI green: `gh pr checks <pr-url> --watch`
  - **Wait for upstream merge before unblocking T2.** This is the user's explicit choice (sequential PRs).

  **Must NOT do**:
  - Do NOT touch any code related to flatten, json, composite pk, upsert, or `is_set` — those have dedicated tasks
  - Do NOT add any new dependencies
  - Do NOT modify the `is_set` attribute API
  - Do NOT bump the version

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Straightforward atomic fixes; no architectural decisions
  - **Skills**: [`git-master`]
    - `git-master`: Branch creation, conventional commits, atomic staging, gitsign signing, PR creation
  - **Skills Evaluated but Omitted**:
    - `ai-slop-remover`: not needed — changes are small and targeted
    - `dev-browser` / `playwright`: no UI to test

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (kicks off the chain)
  - **Blocks**: T2, T3, T4, T5, T6, T7
  - **Blocked By**: None — ready to start immediately

  **References**:

  *Pattern References (existing code to follow)*:
  - `lorm-macros/src/models.rs:37-46` — current single-pk validation pattern; reuse the `syn::Error::new(...)` pattern (no `proc_macro_error2`) and combine via `syn::Error::combine()` if accumulating
  - `lorm-macros/src/orm/by.rs:33-44` — pattern for trait/impl emission; the `impl_code` block is where `#[automatically_derived]` must be added in the outer `impl ... { ... }` quote
  - `lorm-macros/src/orm/save.rs:200-210` — same pattern in save trait/impl emission
  - `lorm-macros/src/orm/{with,delete,select}.rs` — same pattern; apply uniformly

  *API/Type References*:
  - `syn::Error::combine` — std method for accumulating errors. Documented at https://docs.rs/syn/2/syn/struct.Error.html#method.combine
  - `quote!` macro — wrap impls as `impl ... { ... }` becomes `#[automatically_derived] impl ... { ... }`. Verify with `cargo expand --test main`.

  *Test References*:
  - `lorm/tests/main.rs` — current test structure. Existing tests must still pass after this task.
  - `lorm/tests/resources/migrations/sqlite/01_users_table.sql` — schema reference

  *External References*:
  - PR #2 owner review: https://github.com/remysaissy/lorm/pull/2#issuecomment-4098053100 — section "Small fixes needed"
  - MrPine reference branch (local): `git show mrpine-big:lorm-macros/src/orm/by.rs` shows his `#[automatically_derived]` placement (commit `94a88a5`)

  *WHY each reference matters*:
  - The existing `syn::Error` patterns in `models.rs` are what we want to keep — DO NOT regress to `proc_macro_error2::emit_error!` which MrPine used
  - MrPine's `94a88a5` commit shows exactly where `#[automatically_derived]` goes, but adapt to project's `Column` naming

  **Acceptance Criteria**:

  *If TDD (tests enabled)*:
  - [ ] All existing tests in `lorm/tests/main.rs` still pass: `./test.sh --package=lorm --feature=sqlite`
  - [ ] No new test failures on postgres/mysql (CI matrix verifies)

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: cargo build is clean for all 3 backends after the fix
    Tool: Bash
    Preconditions: chore/cleanup-and-fixes branch checked out
    Steps:
      1. Run: cargo build --workspace --no-default-features --features sqlite 2>&1 | tee .sisyphus/evidence/task-1-build-sqlite.log
      2. Run: cargo build --workspace --no-default-features --features postgres 2>&1 | tee .sisyphus/evidence/task-1-build-postgres.log
      3. Run: cargo build --workspace --no-default-features --features mysql 2>&1 | tee .sisyphus/evidence/task-1-build-mysql.log
    Expected Result: All 3 builds exit 0; logs contain no "error[" or "warning: unused"
    Failure Indicators: any "error[", any new "warning:" beyond pre-existing baseline
    Evidence: .sisyphus/evidence/task-1-build-{sqlite,postgres,mysql}.log

  Scenario: #[automatically_derived] is present on all generated impl blocks
    Tool: Bash
    Preconditions: cargo-expand installed (`cargo install cargo-expand`)
    Steps:
      1. Run: cargo expand --test main --no-default-features --features sqlite > .sisyphus/evidence/task-1-expand.rs 2>&1
      2. Run: grep -c "#\[automatically_derived\]" .sisyphus/evidence/task-1-expand.rs
    Expected Result: count >= 5 (at least one per generated trait: SaveTrait, DeleteTrait, ByTrait, WithTrait, SelectTrait)
    Failure Indicators: count < 5 or grep returns 0
    Evidence: .sisyphus/evidence/task-1-expand.rs

  Scenario: PR is created and CI green
    Tool: Bash
    Preconditions: branch pushed to origin
    Steps:
      1. Write the PR body file:
         cat > .sisyphus/evidence/task-1-pr-body.md << 'PREOF'
         ## Summary
         Pre-0.3.0 cleanup addressing items from PR #2 review:
         - Fix `updated_at` duplicate-field check error message
         - Add `#[automatically_derived]` to all generated impl blocks
         - Remove dead code and commented-out examples
         - Update CHANGELOG with unreleased entries
         PREOF
      2. Create PR and capture URL:
         gh pr create --title "chore: pre-0.3.0 cleanup and fixes from PR #2 review" --body-file .sisyphus/evidence/task-1-pr-body.md | tee .sisyphus/evidence/task-1-pr-url.txt
      3. Wait for CI and capture check results:
         PR_URL=$(cat .sisyphus/evidence/task-1-pr-url.txt)
         gh pr checks "$PR_URL" --watch
         gh pr checks "$PR_URL" --json name,state,conclusion > .sisyphus/evidence/task-1-pr-checks.json
    Expected Result: All required CI checks pass (format, clippy, test sqlite/postgres/mysql, examples, coverage); task-1-pr-checks.json contains all checks with conclusion "SUCCESS"
    Failure Indicators: any check conclusion != "SUCCESS" in JSON output
    Evidence: .sisyphus/evidence/task-1-pr-body.md, .sisyphus/evidence/task-1-pr-url.txt, .sisyphus/evidence/task-1-pr-checks.json
  ```

  **Evidence to Capture**:
  - [ ] `.sisyphus/evidence/task-1-build-{sqlite,postgres,mysql}.log`
  - [ ] `.sisyphus/evidence/task-1-expand.rs`
  - [ ] `.sisyphus/evidence/task-1-pr-url.txt`
  - [ ] `.sisyphus/evidence/task-1-pr-checks.json`

  **Commit**: 4 atomic commits in this branch (see "What to do" above), one PR
  - Branch: `chore/cleanup-and-fixes`
  - Pre-commit per commit: `./format.sh && ./check.sh --package=lorm --feature=sqlite`
  - Pre-push: `./test.sh --package=lorm --feature=sqlite`

- [ ] 2. **feat/is-set-callable — migrate `is_set` from `Expr` to `darling::util::Callable` (BREAKING)**

  **What to do**:
  - Wait for T1 PR to be merged before starting (sequential rule)
  - Branch from main: `git pull origin main && git checkout -b feat/is-set-callable`
  - **Update `ColumnPropertyAttrs.is_set_expression`** in `lorm-macros/src/attributes.rs:46` from `Option<Expr>` to `Option<darling::util::Callable>`
  - **Update `ColumnProperties.is_set_expression`** field type accordingly
  - **Update `ColumnProperties::is_set()` method** (currently at `attributes.rs:139-144`): the new shape is a callable that takes `&T` and returns `bool`. Default expression when not provided: `(|val: &T| val == &<T as Default>::default())` — match MrPine's pattern but use project's `Column.ty` for `T`. The emitted call site (in `save.rs`) becomes `(#callable)(#self_accessor)` rather than `(#self_accessor).#expr`.
  - **Refactor save.rs accordingly**: at `lorm-macros/src/orm/save.rs:24-26`, the call site of `is_set()` must produce `(#is_set_callable)(#pk_value)` returning `bool`. Verify the `match` at line 147 still receives a `bool`.
  - **Update test models in `lorm/tests/main.rs`**: change `#[lorm(is_set = "is_nil()")]` to `#[lorm(is_set = "Uuid::is_nil")]` for the `User` model. There are exactly 2 occurrences: line 40 (`#[cfg(any(feature = "sqlite", feature = "postgres"))] mod models`) and line 101 (`#[cfg(feature = "mysql")] mod models`). Verify with: `grep -n 'is_set = "is_nil()"' lorm/tests/main.rs`.
  - **Update README.md** attribute table row for `#[lorm(is_set="...")]`: change example from `#[lorm(is_set="is_nil()")]` to `#[lorm(is_set="Uuid::is_nil")]`.
  - **Update CHANGELOG.md** under `## [Unreleased]`:
    - `### Changed` entry: "**BREAKING**: `#[lorm(is_set = ...)]` now expects a callable path (e.g. `Uuid::is_nil`) instead of a method-call expression (e.g. `is_nil()`). The callable is invoked as `(callable)(&value)` and must return `bool`. The default behavior (compare with `Default::default()`) is unchanged when `is_set` is not specified."
    - Migration block: a code-fence showing before/after.
  - **Update `lorm/src/lib.rs` doc comment** if it mentions the old syntax.
  - **Update macro doc comment** in `lorm-macros/src/lib.rs` (currently line 61-64): reflect the callable form.
  - **Commits** (atomic):
    - `feat(macros)!: migrate is_set attribute to darling Callable`
      - Body explains BREAKING with migration example
      - Footer: `BREAKING CHANGE: #[lorm(is_set = "is_nil()")] becomes #[lorm(is_set = "Uuid::is_nil")]`
    - `test: update test models for new is_set Callable syntax`
    - `docs: update README and CHANGELOG for is_set Callable migration`
  - Push, `gh pr create` with breaking-change emoji 💥 in title and clear migration block in body
  - Wait for CI green and upstream merge before T3

  **Must NOT do**:
  - Do NOT touch flatten, json, composite pk, or upsert logic
  - Do NOT remove the existing `is_set` field — it stays, only the type changes
  - Do NOT introduce `proc-macro-error2`
  - Do NOT change the public `predicates::Where`/`Having` API

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Touches multiple files (`attributes.rs`, `save.rs`, tests, docs); breaking change requires careful diff hygiene
  - **Skills**: [`git-master`]
    - `git-master`: BREAKING-CHANGE footer formatting per Conventional Commits, atomic commits

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: T3, T4, T5, T6, T7
  - **Blocked By**: T1 (must be merged first)

  **References**:

  *Pattern References*:
  - `lorm-macros/src/attributes.rs:31-47` — current `ColumnPropertyAttrs` struct; mutate `is_set_expression` field
  - `lorm-macros/src/attributes.rs:139-144` — current `ColumnProperties::is_set` method; rewrite to invoke callable
  - `lorm-macros/src/orm/save.rs:24-26` — call site emitting `pk_is_set` token; update accordingly
  - `mrpine-big:lorm-macros/src/attributes.rs` (line ~125 in his branch) — reference for `default_is_set_expression` pattern returning `(|val| val == &<#ty as Default>::default())`

  *API/Type References*:
  - `darling::util::Callable` — https://docs.rs/darling/0.23/darling/util/struct.Callable.html
  - `syn::Expr` — https://docs.rs/syn/2/syn/enum.Expr.html

  *Test References*:
  - `lorm/tests/main.rs:40` — User model uses `is_set = "is_nil()"`. After this task: `is_set = "Uuid::is_nil"`
  - `lorm/tests/main.rs:101` — same in mysql models block
  - `lorm/tests/main.rs:259-275` — `test_user_is_created` test exercises the is_set logic; must still pass

  *External References*:
  - Conventional Commits BREAKING-CHANGE format: https://www.conventionalcommits.org/en/v1.0.0/#commit-message-with-and-to-draw-attention-to-breaking-change

  *WHY each reference matters*:
  - `Callable` is darling's purpose-built type for `path::to::function` — semantically distinct from arbitrary `Expr` and produces clearer error messages
  - The default expression pattern from MrPine's branch is correct but adapt: produce `quote! { (|val: &#ty| val == &<#ty as Default>::default()) }`
  - The existing test at line 259 implicitly verifies migration: `User::default()` has nil UUID, save() must INSERT not UPDATE

  **Acceptance Criteria**:

  *If TDD (tests enabled)*:
  - [ ] `./test.sh --package=lorm --feature=sqlite` passes (existing tests still green after model migration)
  - [ ] `./test.sh --package=lorm --feature=postgres` passes
  - [ ] `./test.sh --package=lorm --feature=mysql` passes
  - [ ] Cargo expand of `User` model shows callable invocation: `cargo expand --test main --no-default-features --features sqlite | grep -E "Uuid::is_nil|val == &"` returns at least one match

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: Default is_set still works (no attribute → Default::default comparison)
    Tool: Bash
    Preconditions: branch checked out, sqlite feature
    Steps:
      1. Expand the existing User model (which uses explicit is_set) and also any model WITHOUT is_set (e.g. AltUser or Profile if available) to verify the default path:
         cargo expand --test main --no-default-features --features sqlite 2>&1 > .sisyphus/evidence/task-2-default-expand.txt
      2. Grep for the default closure pattern:
         grep -E "val == &.*Default::default" .sisyphus/evidence/task-2-default-expand.txt | head -5
      3. Also grep for the explicit callable pattern on User:
         grep -E "Uuid::is_nil|is_nil" .sisyphus/evidence/task-2-default-expand.txt | head -5
    Expected Result: The default closure pattern `(|val: &T| val == &<T as Default>::default())` appears for models without explicit is_set; the explicit callable `Uuid::is_nil` appears for User
    Failure Indicators: No default closure pattern found, or expansion panics
    Evidence: .sisyphus/evidence/task-2-default-expand.txt

  Scenario: User is_set = "Uuid::is_nil" makes UUID-default save() INSERT not UPDATE
    Tool: Bash
    Preconditions: branch checked out, models migrated
    Steps:
      1. Run: ./test.sh --package=lorm --feature=sqlite -- test_user_is_created 2>&1 | tee .sisyphus/evidence/task-2-test-created.log
    Expected Result: test passes; log shows INSERT INTO users (id) VALUES (...) (not UPDATE)
    Failure Indicators: test fails or wrong SQL emitted
    Evidence: .sisyphus/evidence/task-2-test-created.log

  Scenario: Old syntax produces a clear compile error (regression guard)
    Tool: Bash
    Preconditions: feat/is-set-callable branch checked out
    Steps:
      1. Create a temporary test model file that uses the OLD is_set syntax:
         mkdir -p .sisyphus/evidence/task-2-old-syntax
         cat > /tmp/task-2-old-syntax-test.rs << 'RSEOF'
         // Temporarily patch a test model to use old syntax and attempt compilation
         // We verify by editing main.rs temporarily, building, then reverting
         RSEOF
      2. Temporarily modify one is_set in lorm/tests/main.rs to the OLD syntax:
         cp lorm/tests/main.rs .sisyphus/evidence/task-2-old-syntax/main.rs.bak
         sed -i '' 's/is_set = "Uuid::is_nil"/is_set = "is_nil()"/' lorm/tests/main.rs
      3. Attempt build (expect failure):
         cargo build --no-default-features --features sqlite -p lorm --tests 2>&1 | tee .sisyphus/evidence/task-2-old-syntax/error.log ; true
      4. Restore original:
         cp .sisyphus/evidence/task-2-old-syntax/main.rs.bak lorm/tests/main.rs
      5. Verify error log contains a darling/path parse error:
         grep -iE "expected.*path|callable|parse|error" .sisyphus/evidence/task-2-old-syntax/error.log
    Expected Result: Step 3 fails with darling parse error mentioning "expected path" or "Callable"; step 5 finds the error message
    Failure Indicators: Step 3 succeeds (regression!) or error log contains a panic instead of a clear error
    Evidence: .sisyphus/evidence/task-2-old-syntax/error.log, .sisyphus/evidence/task-2-old-syntax/main.rs.bak

  Scenario: PR is created and CI green
    Tool: Bash
    Preconditions: branch pushed to origin
    Steps:
      1. Create PR and capture URL:
         gh pr create --title "feat(macros)!: migrate is_set attribute to darling Callable" --body "$(cat <<'PREOF'
         ## Summary
         BREAKING: Migrates `#[lorm(is_set = ...)]` from `syn::Expr` to `darling::util::Callable`.

         ### Migration
         ```diff
         - #[lorm(is_set = "is_nil()")]
         + #[lorm(is_set = "Uuid::is_nil")]
         ```

         BREAKING CHANGE: #[lorm(is_set = "is_nil()")] becomes #[lorm(is_set = "Uuid::is_nil")]
         PREOF
         )" | tee .sisyphus/evidence/task-2-pr-url.txt
      2. Wait for CI and capture:
         PR_URL=$(cat .sisyphus/evidence/task-2-pr-url.txt)
         gh pr checks "$PR_URL" --watch
         gh pr checks "$PR_URL" --json name,state,conclusion > .sisyphus/evidence/task-2-pr-checks.json
    Expected Result: all CI checks pass
    Evidence: .sisyphus/evidence/task-2-pr-url.txt, .sisyphus/evidence/task-2-pr-checks.json
  ```

  **Evidence to Capture**:
  - [ ] `.sisyphus/evidence/task-2-default-expand.txt`
  - [ ] `.sisyphus/evidence/task-2-test-created.log`
  - [ ] `.sisyphus/evidence/task-2-old-syntax/error.log`
  - [ ] `.sisyphus/evidence/task-2-old-syntax/main.rs.bak`
  - [ ] `.sisyphus/evidence/task-2-pr-url.txt`
  - [ ] `.sisyphus/evidence/task-2-pr-checks.json`

  **Commit**: 3 atomic commits, branch `feat/is-set-callable`, breaking change PR
  - Pre-commit: `./format.sh && ./check.sh --package=lorm --feature=sqlite`
  - Pre-push: `./test.sh --package=lorm --feature=sqlite`

- [ ] 3. **feat/sqlx-json — read `#[sqlx(json)]` and wrap binds with `sqlx::types::Json`**

  **What to do**:
  - Wait for T2 PR merge before starting
  - Branch from main: `git pull origin main && git checkout -b feat/sqlx-json`
  - **Extend `SqlxColumnAttributes`** in `lorm-macros/src/attributes.rs:78-83`: add `pub is_json: Flag,` (use `darling::util::Flag`, NOT a sub-struct, since we're rejecting `nullable` extension this round). Update the `#[darling(attributes(sqlx), allow_unknown_fields)]` derive to match — no further attribute changes needed.
  - **Extend `ColumnProperties`** to track `pub use_json: bool,` (mirror style of existing `skip`/`readonly` flags); populate it in `ColumnProperties::from(...)` from `sqlx_attrs.is_json.is_present()`.
  - **Update bind-emitting sites** to wrap json values:
    - `lorm-macros/src/orm/save.rs` `column_value()` closure (lines 68-78): when `column.column_properties.use_json` is true, emit `sqlx::types::Json(#self_accessor)` instead of `#self_accessor`. Note: `Json` takes ownership-or-ref depending on Encode impl; verify with `cargo expand`.
    - `lorm-macros/src/orm/by.rs:38-44` and `with.rs:37-44` and `select.rs` where `.bind(#param_use)` is emitted: when the column being bound is a json column, the param itself must be wrapped at call site too. Concretely: emit `.bind(sqlx::types::Json(#param_use))` when `column.column_properties.use_json`. **However**, the user-provided argument type for `by_<field>(executor, value)` becomes the inner T (not `Json<T>`) — the wrapping is internal. Update `get_bind_param_type_and_usage` in `utils.rs` OR add a thin wrapper: prefer keeping `utils.rs` agnostic and doing the wrap at call sites in `by.rs`/`with.rs`/`select.rs`.
  - **Update type-bound emission** in `utils.rs::get_bind_type_where_constraint`: when use_json, the constraint becomes `sqlx::types::Json<T>: sqlx::Encode<...> + sqlx::Type<...>` — but the user passes raw T. Simplest: when `use_json`, accept any `T: serde::Serialize` and let `Json<T>` provide Encode/Type. Add an alternative constraint generator path in `utils.rs` (new helper `get_json_bind_constraint`) and have callers branch on `use_json`.
  - **Test model**: add `Profile` (or extend an existing model) in `lorm/tests/main.rs` `mod models` (both sqlite/postgres and mysql blocks). Example:
    ```rust
    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    pub struct Profile {
        #[lorm(pk)] #[lorm(new = "Uuid::new_v4()")] #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        #[lorm(by)]
        pub user_id: Uuid,
        #[sqlx(json)]
        pub preferences: serde_json::Value,
    }
    ```
  - **Add migrations**: `lorm/tests/resources/migrations/sqlite/05_profiles_table.sql`, `postgres/05_profiles_table.sql`, `mysql/05_profiles_table.sql`. Schema:
    - sqlite: `preferences TEXT NOT NULL` (sqlx Json encodes to TEXT)
    - postgres: `preferences JSONB NOT NULL`
    - mysql: `preferences JSON NOT NULL`
  - **Add tests**: `test_profile_save_with_json`, `test_profile_by_user_id_returns_json` in `lorm/tests/main.rs`. Insert a profile with a serde_json::json!({"theme":"dark"}) value, save, fetch, assert round-trip.
  - **Update Cargo.toml**: add `serde_json = { version = "1.0" }` to `lorm/Cargo.toml [dev-dependencies]` (only dev — example isn't required). Use workspace-level dep declaration to keep version centralization.
  - **Update README.md**:
    - Add a row to "SQLx Attributes (consumed by Lorm)" table for `#[sqlx(json)]` with description and example
    - Add a paragraph in FAQ: "Does Lorm support JSON columns? Yes — annotate the field with `#[sqlx(json)]` and ensure your database column is a JSON type (TEXT for SQLite, JSONB for Postgres, JSON for MySQL). Lorm will wrap binds with `sqlx::types::Json`."
  - **Update CHANGELOG.md** under `## [Unreleased]`:
    - `### Added`: "Support `#[sqlx(json)]` attribute on fields. Lorm wraps bind values with `sqlx::types::Json` automatically. The bare `#[sqlx(json)]` form is supported; the `#[sqlx(json(nullable))]` extension is intentionally not implemented in this release."
  - **Commits** (atomic):
    - `feat(macros): parse #[sqlx(json)] attribute via darling Flag`
    - `feat(macros): wrap json field binds with sqlx::types::Json in save and queries`
    - `test: add Profile model and json round-trip tests for all 3 backends`
    - `docs: document #[sqlx(json)] support in README and CHANGELOG`
  - Push, `gh pr create`, watch CI, wait for merge

  **Must NOT do**:
  - Do NOT support `#[sqlx(json(nullable))]` — out of scope this round
  - Do NOT touch flatten, composite pk, or upsert
  - Do NOT add `serde_json` to runtime deps — dev only
  - Do NOT allow `#[sqlx(json)]` on a field that is also `#[lorm(pk)]` — emit a clear `syn::Error::new` rejecting this combination (json-typed pks defeat WHERE comparisons in upsert later)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Multi-file macro feature with type-bound nuance and 3-backend SQL schema
  - **Skills**: [`git-master`]
    - `git-master`: Atomic commits, PR creation

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: T4, T5, T6, T7
  - **Blocked By**: T2 (must be merged)

  **References**:

  *Pattern References*:
  - `lorm-macros/src/attributes.rs:78-83` — `SqlxColumnAttributes` is where the new `is_json: Flag` field goes
  - `lorm-macros/src/orm/save.rs:68-78` — `column_value` closure; this is where the json wrap happens for inserts
  - `lorm-macros/src/orm/by.rs:38-44` — `.bind(#param_value)` emission; mirror pattern for json wrapping
  - `lorm-macros/src/utils.rs:159-176` — `get_bind_type_where_constraint`; either branch internally on a `use_json` parameter or add a sibling helper
  - `mrpine-big:lorm-macros/src/orm/save.rs` (lines ~95-105 in his branch) — shows `if field.column_properties.use_json { quote! {sqlx::types::Json(#value)} }` pattern; adapt this exact idea

  *API/Type References*:
  - `sqlx::types::Json<T>` — https://docs.rs/sqlx/0.8/sqlx/types/struct.Json.html
  - `darling::util::Flag` — https://docs.rs/darling/0.23/darling/util/struct.Flag.html

  *Test References*:
  - `lorm/tests/main.rs:259-275` — pattern for save+fetch round-trip tests
  - `lorm/tests/resources/migrations/sqlite/01_users_table.sql` — schema file naming convention (NN_name.sql)
  - `lorm/tests/resources/migrations/postgres/01_users_table.sql` — same for postgres
  - `lorm/tests/resources/migrations/mysql/01_users_table.sql` — same for mysql

  *External References*:
  - SQLx JSON docs (Postgres JSONB): https://docs.rs/sqlx/0.8/sqlx/types/struct.Json.html
  - serde_json: https://docs.rs/serde_json/1

  *WHY each reference matters*:
  - The `column_value` closure in `save.rs` is the SINGLE point where insert/update binds are emitted — wrap there to cover both INSERT and UPDATE
  - `by.rs`/`with.rs` are simpler — they have one `.bind()` site each
  - SQLx's `Json<T>` requires `T: Serialize + DeserializeOwned`; this is the new constraint to emit when `use_json`

  **Acceptance Criteria**:

  *If TDD (tests enabled)*:
  - [ ] New `Profile` model compiles with `#[sqlx(json)]` annotation under all 3 backends
  - [ ] `test_profile_save_with_json` and `test_profile_by_user_id_returns_json` PASS for sqlite/postgres/mysql
  - [ ] Existing tests still pass

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: JSON round-trip survives save → fetch on sqlite
    Tool: Bash
    Preconditions: feat/sqlx-json branch checked out
    Steps:
      1. Run: ./test.sh --package=lorm --feature=sqlite -- test_profile 2>&1 | tee .sisyphus/evidence/task-3-test-sqlite.log
    Expected Result: 2 tests pass; log shows the round-tripped JSON value matches input
    Failure Indicators: test fails, JSON deserializes to wrong type, encoded as wrong column type
    Evidence: .sisyphus/evidence/task-3-test-sqlite.log

  Scenario: JSON round-trip on Postgres uses JSONB column type
    Tool: Bash
    Preconditions: postgres CI service or local postgres
    Steps:
      1. Run tests:
         ./test.sh --package=lorm --feature=postgres -- test_profile 2>&1 | tee .sisyphus/evidence/task-3-test-postgres.log
      2. Copy schema to evidence and verify JSONB:
         cp lorm/tests/resources/migrations/postgres/05_profiles_table.sql .sisyphus/evidence/task-3-postgres-schema.sql
         grep -i "jsonb" .sisyphus/evidence/task-3-postgres-schema.sql
    Expected Result: tests pass; grep finds JSONB column type in the schema file
    Failure Indicators: tests fail, or grep finds no JSONB (means wrong column type used)
    Evidence: .sisyphus/evidence/task-3-test-postgres.log, .sisyphus/evidence/task-3-postgres-schema.sql

  Scenario: JSON round-trip on MySQL uses JSON column type
    Tool: Bash
    Steps:
      1. Run: ./test.sh --package=lorm --feature=mysql -- test_profile 2>&1 | tee .sisyphus/evidence/task-3-test-mysql.log
    Expected Result: tests pass
    Evidence: .sisyphus/evidence/task-3-test-mysql.log

  Scenario: cargo expand shows sqlx::types::Json wrapping in save
    Tool: Bash
    Steps:
      1. Run: cargo expand --test main --no-default-features --features sqlite > .sisyphus/evidence/task-3-expand.rs 2>&1
      2. Run: grep -n "sqlx::types::Json" .sisyphus/evidence/task-3-expand.rs
    Expected Result: at least 2 matches (insert and update bind sites for the Profile model)
    Failure Indicators: 0 matches
    Evidence: .sisyphus/evidence/task-3-expand.rs

  Scenario: #[sqlx(json)] on a #[lorm(pk)] field rejected at compile time
    Tool: Bash
    Preconditions: feat/sqlx-json branch checked out
    Steps:
      1. Temporarily patch the Profile model to combine #[lorm(pk)] and #[sqlx(json)] on the same field:
         mkdir -p .sisyphus/evidence/task-3-bad-pk-json
         cp lorm/tests/main.rs .sisyphus/evidence/task-3-bad-pk-json/main.rs.bak
         sed -i '' 's/#\[sqlx(json)\]/#[lorm(pk)] #[sqlx(json)]/' lorm/tests/main.rs
      2. Attempt build (expect failure):
         cargo build --no-default-features --features sqlite -p lorm --tests 2>&1 | tee .sisyphus/evidence/task-3-bad-pk-json/error.log ; true
      3. Restore original:
         cp .sisyphus/evidence/task-3-bad-pk-json/main.rs.bak lorm/tests/main.rs
      4. Verify the error message:
         grep -iE "json.*primary.key|primary.key.*json|cannot be" .sisyphus/evidence/task-3-bad-pk-json/error.log
    Expected Result: compile error mentioning "json column cannot be a primary key" or similar; step 4 finds the error
    Failure Indicators: build succeeds (missing validation!) or error is a panic without clear message
    Evidence: .sisyphus/evidence/task-3-bad-pk-json/error.log, .sisyphus/evidence/task-3-bad-pk-json/main.rs.bak

  Scenario: PR is created and CI green
    Tool: Bash
    Preconditions: branch pushed to origin
    Steps:
      1. Create PR and capture URL:
         gh pr create --title "feat(macros): support #[sqlx(json)] attribute for JSON column binding" --body "$(cat <<'PREOF'
         ## Summary
         - Parse `#[sqlx(json)]` via darling Flag
         - Wrap json field binds with `sqlx::types::Json` in save, by, with, select
         - Add Profile model + json round-trip tests for sqlite/postgres/mysql
         - Reject `#[sqlx(json)]` on `#[lorm(pk)]` fields at compile time
         PREOF
         )" | tee .sisyphus/evidence/task-3-pr-url.txt
      2. Wait for CI and capture:
         PR_URL=$(cat .sisyphus/evidence/task-3-pr-url.txt)
         gh pr checks "$PR_URL" --watch
         gh pr checks "$PR_URL" --json name,state,conclusion > .sisyphus/evidence/task-3-pr-checks.json
    Expected Result: all CI checks pass
    Evidence: .sisyphus/evidence/task-3-pr-url.txt, .sisyphus/evidence/task-3-pr-checks.json
  ```

  **Evidence to Capture**:
  - [ ] `.sisyphus/evidence/task-3-test-{sqlite,postgres,mysql}.log`
  - [ ] `.sisyphus/evidence/task-3-postgres-schema.sql`
  - [ ] `.sisyphus/evidence/task-3-expand.rs`
  - [ ] `.sisyphus/evidence/task-3-bad-pk-json/error.log`
  - [ ] `.sisyphus/evidence/task-3-bad-pk-json/main.rs.bak`
  - [ ] `.sisyphus/evidence/task-3-pr-url.txt`
  - [ ] `.sisyphus/evidence/task-3-pr-checks.json`

  **Commit**: 4 atomic commits, branch `feat/sqlx-json`
  - Pre-commit per commit: `./format.sh && ./check.sh --package=lorm --feature=sqlite`
  - Pre-push: `./test.sh --package=lorm --feature=sqlite`

- [ ] 4. **feat/sqlx-flatten — wire `#[sqlx(flatten)]` + `#[lorm(flattened(...))]` to expand one Rust field into multiple SQL columns**

  **What to do**:
  - Wait for T3 PR merge
  - Branch: `git pull origin main && git checkout -b feat/sqlx-flatten`
  - **Parse `#[sqlx(flatten)]`**: extend `SqlxColumnAttributes` (in `lorm-macros/src/attributes.rs`) with `pub flatten: Flag,` — already present in MrPine's reference (line ~63 in his branch)
  - **Add `FlattenedFields` darling-parsed nested attribute**: declare a helper struct in `attributes.rs` to parse `#[lorm(flattened(field1: Ty1, field2: Ty2 = "renamed_col", ...))]`. The list contains entries with: field ident, type, and optional rename string. Use `FromMeta` impl that walks `NestedMeta` items. Reference MrPine's `mrpine-big:lorm-macros/src/attributes.rs` `FlattenedFields` block for grammar; **adapt** to use `syn::Error::combine` not `proc_macro_error2`.
  - **Wire `process_struct_field`** in `lorm-macros/src/models.rs:103-129`: when both `#[sqlx(flatten)]` and `#[lorm(flattened(...))]` are present, EXPAND the single base field into multiple `Column` entries (one per declared nested field). Each expanded Column has:
    - `base_field`: the parent field (e.g. `address`)
    - `field`: the nested field ident (e.g. `street`)
    - `ty`: the nested type (e.g. `String`)
    - `column_name`: the `= "renamed_col"` if provided, else nested field name in snake_case
    - `is_flattened`: true
    - `column_properties`: inherits parent's `column_properties` BUT clears `primary_key` (flattened fields cannot be pk; reject explicitly), keeps `generate_by`/`readonly`/`use_json` etc.
  - **Update `Column::self_accessor()` in `column.rs:21-33`**: already supports `is_flattened` via `self.#base_ident.#field_ident` and `Option`-aware variant — verify the existing implementation matches; no change needed if so
  - **Reject conflicts**:
    - `#[lorm(flattened(...))]` without `#[sqlx(flatten)]` → error
    - `#[sqlx(flatten)]` without `#[lorm(flattened(...))]` → error (we don't auto-detect nested struct fields)
    - flattened field with `#[lorm(pk)]` → error
    - flattened field with `#[lorm(created_at)]` or `#[lorm(updated_at)]` → error
  - **Test model**: add an `Address` nested struct (FromRow only; not ToLOrm) and a `Customer` model:
    ```rust
    #[derive(Debug, Default, Clone, FromRow)]
    pub struct Address {
        pub street: String,
        #[sqlx(rename = "zip_code")] pub zip: String,
    }
    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    pub struct Customer {
        #[lorm(pk)] #[lorm(new = "Uuid::new_v4()")] #[lorm(is_set = "Uuid::is_nil")]
        pub id: Uuid,
        #[lorm(by)] pub email: String,
        #[sqlx(flatten)]
        #[lorm(flattened(street: String, zip: String = "zip_code"))]
        pub address: Address,
    }
    ```
  - **Test model with Option**: add `OptCustomer` with `#[sqlx(flatten)] #[lorm(flattened(...))] pub address: Option<Address>` to verify Option handling
  - **Migrations**: `06_customers_table.sql` + `07_opt_customers_table.sql` for each backend (with `street`, `zip_code` columns; nullable for OptCustomer)
  - **Tests**: `test_customer_save_with_flatten`, `test_customer_by_email_returns_flattened`, `test_opt_customer_with_none_address` — assert that flattened columns round-trip correctly and that None → NULL columns
  - **Update README.md**: add `#[lorm(flattened(...))]` row to attribute table and `#[sqlx(flatten)]` row to SQLx-attributes table; replace FAQ entry "Does Lorm support relationships/joins?" sub-paragraph or add adjacent FAQ entry "How do I use flattened fields?" matching MrPine's wording
  - **Update CHANGELOG**: `### Added` "Support `#[sqlx(flatten)]` with `#[lorm(flattened(field: Type, field: Type = \"renamed_col\", ...))]` to expand a nested struct into multiple SQL columns. `Option<NestedStruct>` is supported (None → all NULL)."
  - **Commits**:
    - `feat(macros): parse #[sqlx(flatten)] and #[lorm(flattened(...))] attributes`
    - `feat(macros): expand flattened fields into multiple Column entries`
    - `feat(macros): support Option<NestedStruct> for flattened fields`
    - `test: add Customer and OptCustomer models with flatten coverage`
    - `docs: document flatten support in README and CHANGELOG`
  - Push, `gh pr create`, CI green, merge

  **Must NOT do**:
  - Do NOT auto-derive flattened columns — require explicit `#[lorm(flattened(...))]`
  - Do NOT support `#[lorm(pk)]` on a flattened sub-field (composite pk task is separate AND nested structs in pk add a layer too far for this round)
  - Do NOT touch json or composite pk
  - Do NOT introduce reflection / runtime parsing — pure compile-time

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Most complex parse work (nested attribute grammar) + iteration logic (1 base field → N Columns) + 3-backend SQL schemas + Option<T> nuance
  - **Skills**: [`git-master`]
    - `git-master`: Atomic commits

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: T5, T6, T7
  - **Blocked By**: T3

  **References**:

  *Pattern References*:
  - `lorm-macros/src/orm/column.rs:21-33` — `self_accessor()` already implements is_flattened (with Option!); reuse
  - `lorm-macros/src/models.rs:103-129` — `process_struct_field`; this is the pivot point where 1 field becomes N Columns
  - `mrpine-big:lorm-macros/src/attributes.rs` `FlattenedFields` definition — grammar reference
  - `mrpine-big:lorm-macros/src/models.rs:process_struct_field` — fan-out logic reference

  *API/Type References*:
  - `darling::FromMeta` for parsing nested attributes
  - `syn::TypeTuple` / `syn::Type` for the type expressions inside `flattened(field: Ty)`
  - `syn::parse::Parse` for custom token-stream parsing if `FromMeta` doesn't suffice

  *Test References*:
  - `lorm/tests/main.rs:35-86` — model patterns
  - `lorm/tests/resources/migrations/postgres/01_users_table.sql` — schema file format

  *External References*:
  - `darling` macro syntax: https://docs.rs/darling/0.23/darling/derive.FromMeta.html

  *WHY each reference matters*:
  - `column.rs::self_accessor` already handles the `Option<T>` flatten correctly — DON'T reimplement, the ground work was laid in earlier merges
  - MrPine's grammar `flattened(field: Ty, field: Ty = "name")` is good but his code uses `proc_macro_error2` — adapt to `syn::Error::combine`

  **Acceptance Criteria**:

  *If TDD (tests enabled)*:
  - [ ] `test_customer_save_with_flatten` PASS on sqlite/postgres/mysql
  - [ ] `test_customer_by_email_returns_flattened` PASS
  - [ ] `test_opt_customer_with_none_address` PASS (NULL columns)

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: Flattened columns round-trip for non-Option case
    Tool: Bash
    Steps:
      1. Run: ./test.sh --package=lorm --feature=sqlite -- test_customer 2>&1 | tee .sisyphus/evidence/task-4-test-flatten.log
    Expected Result: tests pass; saved street/zip values match retrieved values
    Evidence: .sisyphus/evidence/task-4-test-flatten.log

  Scenario: Option<NestedStruct> with None saves NULL columns
    Tool: Bash
    Steps:
      1. Run: ./test.sh --package=lorm --feature=sqlite -- test_opt_customer 2>&1 | tee .sisyphus/evidence/task-4-test-opt-flatten.log
    Expected Result: pass; SQL log shows NULL bound for street and zip_code when address is None
    Evidence: .sisyphus/evidence/task-4-test-opt-flatten.log

  Scenario: cargo expand shows multiple columns for the flattened field
    Tool: Bash
    Steps:
      1. Run: cargo expand --test main --no-default-features --features sqlite > .sisyphus/evidence/task-4-expand.rs 2>&1
      2. Run: grep -E "self\.address\.street|self\.address\.zip" .sisyphus/evidence/task-4-expand.rs | wc -l
    Expected Result: count >= 4 (insert + update for each of 2 nested fields)
    Evidence: .sisyphus/evidence/task-4-expand.rs

  Scenario: Compile-time error when flattened sub-field has #[lorm(pk)]
    Tool: Bash
    Preconditions: feat/sqlx-flatten branch checked out
    Steps:
      1. Temporarily patch Customer model — add #[lorm(pk)] to the flatten field:
         mkdir -p .sisyphus/evidence/task-4-bad-flatten-pk
         cp lorm/tests/main.rs .sisyphus/evidence/task-4-bad-flatten-pk/main.rs.bak
         perl -i -pe 'print "        #[lorm(pk)]\n" if /\#\[sqlx\(flatten\)\]/ && !$done++' lorm/tests/main.rs
      2. Attempt build (expect failure):
         cargo build --no-default-features --features sqlite -p lorm --tests 2>&1 | tee .sisyphus/evidence/task-4-bad-flatten-pk/error.log ; true
      3. Restore original:
         cp .sisyphus/evidence/task-4-bad-flatten-pk/main.rs.bak lorm/tests/main.rs
      4. Verify error message:
         grep -iE "flatten.*pk|pk.*flatten|primary.key.*flatten" .sisyphus/evidence/task-4-bad-flatten-pk/error.log
    Expected Result: clear compile error mentioning flattened + pk conflict
    Evidence: .sisyphus/evidence/task-4-bad-flatten-pk/error.log, .sisyphus/evidence/task-4-bad-flatten-pk/main.rs.bak

  Scenario: Compile-time error when #[sqlx(flatten)] is set but #[lorm(flattened(...))] is missing
    Tool: Bash
    Preconditions: feat/sqlx-flatten branch checked out
    Steps:
      1. Temporarily patch Customer model — remove #[lorm(flattened(...))] but keep #[sqlx(flatten)]:
         mkdir -p .sisyphus/evidence/task-4-missing-flattened
         cp lorm/tests/main.rs .sisyphus/evidence/task-4-missing-flattened/main.rs.bak
         sed -i '' '/#\[lorm(flattened(/d' lorm/tests/main.rs
      2. Attempt build (expect failure):
         cargo build --no-default-features --features sqlite -p lorm --tests 2>&1 | tee .sisyphus/evidence/task-4-missing-flattened/error.log ; true
      3. Restore original:
         cp .sisyphus/evidence/task-4-missing-flattened/main.rs.bak lorm/tests/main.rs
      4. Verify error message:
         grep -iE "flatten.*flattened|requires.*flattened|both.*attributes" .sisyphus/evidence/task-4-missing-flattened/error.log
    Expected Result: error mentions both attributes are required together
    Evidence: .sisyphus/evidence/task-4-missing-flattened/error.log, .sisyphus/evidence/task-4-missing-flattened/main.rs.bak

  Scenario: PR created, CI green
    Tool: Bash
    Preconditions: branch pushed to origin
    Steps:
      1. Create PR and capture URL:
         gh pr create --title "feat(macros): support #[sqlx(flatten)] with #[lorm(flattened(...))]" --body "$(cat <<'PREOF'
         ## Summary
         - Parse `#[sqlx(flatten)]` + `#[lorm(flattened(field: Type, ...))]` attributes
         - Expand flattened fields into multiple Column entries in code generation
         - Support `Option<NestedStruct>` (None -> all NULL columns)
         - Add Customer and OptCustomer models with tests for all 3 backends
         - Reject conflicts: flatten+pk, sqlx(flatten) without lorm(flattened), and vice versa
         PREOF
         )" | tee .sisyphus/evidence/task-4-pr-url.txt
      2. Wait for CI and capture:
         PR_URL=$(cat .sisyphus/evidence/task-4-pr-url.txt)
         gh pr checks "$PR_URL" --watch
         gh pr checks "$PR_URL" --json name,state,conclusion > .sisyphus/evidence/task-4-pr-checks.json
    Expected Result: all CI checks pass
    Evidence: .sisyphus/evidence/task-4-pr-url.txt, .sisyphus/evidence/task-4-pr-checks.json
  ```

  **Evidence to Capture**:
  - [ ] `.sisyphus/evidence/task-4-test-flatten.log`
  - [ ] `.sisyphus/evidence/task-4-test-opt-flatten.log`
  - [ ] `.sisyphus/evidence/task-4-expand.rs`
  - [ ] `.sisyphus/evidence/task-4-bad-flatten-pk/error.log`
  - [ ] `.sisyphus/evidence/task-4-missing-flattened/error.log`
  - [ ] `.sisyphus/evidence/task-4-pr-url.txt`
  - [ ] `.sisyphus/evidence/task-4-pr-checks.json`

  **Commit**: 5 atomic commits, branch `feat/sqlx-flatten`
  - Pre-commit: `./format.sh && ./check.sh --package=lorm --feature=sqlite`
  - Pre-push: `./test.sh --package=lorm --feature=sqlite`

- [ ] 5. **feat/composite-pk — `#[lorm(pk_type = "manual")]`, multiple `#[lorm(pk)]` fields, optional `#[lorm(pk_selector = "...")]`**

  **What to do**:
  - Wait for T4 PR merge
  - Branch: `git pull origin main && git checkout -b feat/composite-pk`
  - **Add `PrimaryKeyType` enum** in `lorm-macros/src/attributes.rs`:
    ```rust
    #[derive(Debug, Copy, Clone, Eq, PartialEq, FromMeta)]
    pub enum PrimaryKeyType { Generated, Manual }
    ```
    Default is `Generated` (backward compatible).
  - **Extend `TableAttributes`** in `attributes.rs` to include:
    - `#[darling(default = default_pk_type)] pub pk_type: PrimaryKeyType,` (default `Generated`)
    - `#[darling(rename = "pk_selector")] primary_key_selector: Option<syn::Ident>,`
    - `manual_primary_key_selector(&self, pk: &PrimaryKey) -> Ident` method that returns: provided selector if Some, else field-name-based `format_ident!("by_{}", field.field)` for single-field manual, else `format_ident!("by_key")` for composite.
  - **Replace single-pk validation with `PrimaryKey` enum** in `lorm-macros/src/models.rs`:
    ```rust
    pub(crate) enum PrimaryKey<'a> {
        Generated(Box<Column<'a>>),
        Manual(Vec<Column<'a>>),
    }
    impl<'a> PrimaryKey<'a> {
        pub fn is_generated(&self) -> bool { matches!(self, PrimaryKey::Generated(_)) }
        pub fn fields(&'a self) -> &'a [Column<'a>] { match self { PrimaryKey::Generated(f) => slice::from_ref(f), PrimaryKey::Manual(v) => v } }
        pub fn column_names(&self) -> impl Iterator<Item = &str> { ... }
    }
    ```
  - **Update `OrmModel`** to hold `primary_key: PrimaryKey<'a>` and `primary_key_selector: Ident` instead of (currently implicit) single pk.
  - **Refactor `OrmModel::from_fields`** in `models.rs:21-54`:
    - Read `pk_type` from `TableAttributes`
    - Filter `#[lorm(pk)]`-annotated columns; validate based on pk_type:
      - `Generated`: exactly 1 pk column required (current rule)
      - `Manual`: at least 1 pk column required; multiple allowed
    - Construct `PrimaryKey::Generated(Box<...>)` or `PrimaryKey::Manual(Vec<...>)` accordingly
    - Reject: `pk_type = "generated"` + multiple pk columns → error
    - Reject: pk column with `created_at`/`updated_at` → error (timestamps cannot be pk)
    - Reject: pk column that is also flattened → error (already enforced in T4 but reaffirm)
    - Reject: pk column with `is_set`/`new` when `pk_type = "manual"` (these only make sense for generated)
    - Reject: pk column with `#[sqlx(json)]` → error (already enforced in T3)
  - **Update all callers** of `model.primary_key()`:
    - `lorm-macros/src/orm/save.rs:15`: replace with iteration over `model.primary_key.fields()`
    - `lorm-macros/src/orm/delete.rs:12-19`: WHERE clause must AND all pk columns. Use placeholders `$1, $2, ...` (postgres/sqlite) or `?, ?, ...` (mysql). Bind values must be the `self_accessor()` of each pk column.
  - **Refactor `Column::should_generate_query_function()` → `should_generate_selector(pk)`** in `column.rs:40-45`: returns true if `generate_by` || `created_at` || `updated_at` || (single-field PK and that field is the only pk). For composite pks, individual pk fields don't auto-get `by_<field>` (since the composite key is the meaningful unit) — UNLESS they also have explicit `#[lorm(by)]`. This matches MrPine's `mrpine-big:lorm-macros/src/orm/logical_field.rs:36-46`.
  - **Generate composite key selector method** in `lorm-macros/src/orm/by.rs`: when pk is `Manual` and has 2+ fields, emit a function `<by_key or pk_selector>(executor, pk1: T1, pk2: T2, ...) -> Result<Self>`. SQL: `SELECT cols FROM table WHERE pk1_col = $1 AND pk2_col = $2`. Use the same `get_bind_param_type_and_usage`/`get_bind_type_where_constraint` helpers per pk field. Place the new method in the `<Struct>ByTrait` alongside existing `by_<field>` methods.
  - **Test models**: add `UserRole` (composite pk join table) to `lorm/tests/main.rs`:
    ```rust
    #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
    #[lorm(pk_type = "manual")]
    pub struct UserRole {
        #[lorm(pk)] pub user_id: Uuid,
        #[lorm(pk)] pub role_id: Uuid,
        pub assigned_at: chrono::DateTime<FixedOffset>,
    }
    ```
    And `UserRoleNamed` with `#[lorm(pk_type = "manual", pk_selector = "by_user_role")]`. Both for sqlite/postgres/mysql.
  - **Add migrations**: `08_user_roles_table.sql` and `09_user_roles_named_table.sql` for each backend
  - **Tests**:
    - `test_user_role_save_inserts` — new UserRole, save, verify it lands in DB
    - `test_user_role_save_updates` — modify a non-pk column, save again, verify UPDATE not INSERT (note: this requires the upsert from T6; until T6 lands, save() for Manual pk will fail compile or always INSERT — this test can be added in T5 but skipped until T6 with `#[ignore]` and re-enabled there. **Actually**: leave save() unchanged for Manual in T5 — just keep it as INSERT for now and assert that. T6 turns it into upsert and updates the test.)
    - `test_user_role_by_key_returns_match` — by_key(executor, user_id, role_id) returns the row
    - `test_user_role_named_by_user_role_returns_match` — verify `pk_selector = "by_user_role"` works
    - `test_user_role_delete_uses_composite_where` — delete and verify gone
  - **Update README**:
    - Replace FAQ "How do I handle composite primary keys? Lorm currently supports single-field primary keys only" with the answer from MrPine's branch (composite pk via pk_type=manual)
    - Add struct-level attributes table rows for `pk_type` and `pk_selector` (mirror MrPine's table, adapted)
    - Update Limitations section: remove "Primary key field name detection is attribute-based, not convention-based" if no longer accurate (still accurate; keep)
  - **Update CHANGELOG**:
    - `### Added`: composite pk support, `pk_type` and `pk_selector` attributes, `by_key()` (or custom selector) generated method
    - `### Changed`: `Column::should_generate_query_function` → `should_generate_selector(pk)` (internal API)
  - **Commits**:
    - `refactor(macros): introduce PrimaryKey enum (Generated | Manual)`
    - `feat(macros): support pk_type attribute (defaults to Generated)`
    - `feat(macros): allow multiple #[lorm(pk)] fields with pk_type = "manual"`
    - `feat(macros): generate composite key selector (by_key or custom via pk_selector)`
    - `feat(macros): update delete() to AND all pk columns for composite keys`
    - `test: add UserRole composite pk model and tests for all 3 backends`
    - `docs: document composite pk support in README and CHANGELOG`
  - Push, PR, CI, merge

  **Must NOT do**:
  - Do NOT modify save() to do upsert here — T6 owns that. In T5, Manual pk save() may simply INSERT; tests that need UPDATE behavior for Manual pk are ignored until T6.
  - Do NOT remove the `Generated` default — backward compat
  - Do NOT touch json or flatten code
  - Do NOT introduce `proc-macro-error2`
  - Do NOT auto-generate `by_<field>` for individual fields of a composite key unless `#[lorm(by)]` is explicit on them

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core ORM type refactor that touches every code-gen module; many cross-feature interactions
  - **Skills**: [`git-master`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: T6, T7
  - **Blocked By**: T4

  **References**:

  *Pattern References*:
  - `lorm-macros/src/models.rs:37-46` — current single-pk enforcement (must be replaced)
  - `lorm-macros/src/orm/delete.rs:11-19` — current single-pk WHERE; must AND all pk columns
  - `lorm-macros/src/orm/save.rs:14-26` — current single-pk handling in save (touched in T5 only minimally; T6 does the upsert)
  - `mrpine-big:lorm-macros/src/models.rs` `PrimaryKey` enum — adapt to project conventions
  - `mrpine-big:lorm-macros/src/orm/by.rs` — selector generation including composite case
  - `mrpine-big:lorm-macros/src/attributes.rs:TableAttributes::manual_primary_key_selector` — selector default logic

  *API/Type References*:
  - `darling::FromMeta` impl on the new `PrimaryKeyType` enum
  - `syn::Ident::new` / `format_ident!` for emitted method names

  *Test References*:
  - `lorm/tests/main.rs:389-397` — `test_automatic_pk_and_ts_insertion_update_is_working` pattern; new tests follow same shape
  - `lorm/tests/resources/migrations/sqlite/03_alt_users_table.sql` — composite pk SQL example for SQLite

  *External References*:
  - PR #2 owner review section "Composite primary keys": "A feature I wanted to do for a long time, thanks for it! Having the default behaviour as PrimaryKey::Generated is the right way."

  *WHY each reference matters*:
  - The owner explicitly approved `PrimaryKey::Generated` as default — bake this into the `default_pk_type()` helper
  - MrPine's selector method default logic (single-field → field name, composite → by_key) matches user's choice in this plan's interview

  **Acceptance Criteria**:

  - [ ] `UserRole` model compiles with `#[lorm(pk_type = "manual")]` + 2 `#[lorm(pk)]` fields under all 3 backends
  - [ ] `UserRole::by_key(&pool, user_id, role_id)` is a generated method
  - [ ] `UserRoleNamed::by_user_role(&pool, user_id, role_id)` is a generated method (custom selector)
  - [ ] `UserRole.delete(&pool)` produces `WHERE user_id = $1 AND role_id = $2` (verify via cargo expand)
  - [ ] Single-field manual pk model generates `by_<field>` (per user's choice — verify with a test model `SingleManualPk` whose pk field is `code`, expect `by_code`)
  - [ ] Existing single-pk models (User, AltUser, Profile, Customer) still work unchanged
  - [ ] Tests pass on sqlite/postgres/mysql

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: Composite pk by_key fetch works on all 3 backends
    Tool: Bash
    Steps:
      1. ./test.sh --package=lorm --feature=sqlite -- test_user_role_by_key 2>&1 | tee .sisyphus/evidence/task-5-test-by-key-sqlite.log
      2. ./test.sh --package=lorm --feature=postgres -- test_user_role_by_key 2>&1 | tee .sisyphus/evidence/task-5-test-by-key-postgres.log
      3. ./test.sh --package=lorm --feature=mysql -- test_user_role_by_key 2>&1 | tee .sisyphus/evidence/task-5-test-by-key-mysql.log
    Expected Result: all 3 pass
    Evidence: .sisyphus/evidence/task-5-test-by-key-{sqlite,postgres,mysql}.log

  Scenario: pk_selector custom name generates expected method
    Tool: Bash
    Steps:
      1. cargo expand --test main --no-default-features --features sqlite | grep "fn by_user_role" > .sisyphus/evidence/task-5-custom-selector.txt
    Expected Result: file contains the function signature for by_user_role
    Evidence: .sisyphus/evidence/task-5-custom-selector.txt

  Scenario: delete() emits AND-joined WHERE for composite pk
    Tool: Bash
    Steps:
      1. cargo expand --test main --no-default-features --features sqlite | grep -A 1 "DELETE FROM user_roles" > .sisyphus/evidence/task-5-delete-where.txt
    Expected Result: file shows `WHERE user_id = $1 AND role_id = $2` (or `?` for mysql equivalent)
    Evidence: .sisyphus/evidence/task-5-delete-where.txt

  Scenario: Backward compat — pk_type defaults to Generated, existing User model still works
    Tool: Bash
    Steps:
      1. ./test.sh --package=lorm --feature=sqlite -- test_user 2>&1 | tee .sisyphus/evidence/task-5-backcompat.log
    Expected Result: all existing user-model tests still pass (single pk preserved)
    Evidence: .sisyphus/evidence/task-5-backcompat.log

  Scenario: Compile error when pk_type = "generated" with 2 pk fields
    Tool: Bash
    Preconditions: feat/composite-pk branch checked out
    Steps:
      1. Temporarily patch User model — add a second #[lorm(pk)] on email (keeping default Generated pk_type):
         mkdir -p .sisyphus/evidence/task-5-bad-generated-multi
         cp lorm/tests/main.rs .sisyphus/evidence/task-5-bad-generated-multi/main.rs.bak
         perl -i -pe 'print "        #[lorm(pk)]\n" if /^\s+pub email: String/ && !$done++' lorm/tests/main.rs
      2. Attempt build (expect failure):
         cargo build --no-default-features --features sqlite -p lorm --tests 2>&1 | tee .sisyphus/evidence/task-5-bad-generated-multi/error.log ; true
      3. Restore original:
         cp .sisyphus/evidence/task-5-bad-generated-multi/main.rs.bak lorm/tests/main.rs
      4. Verify error message:
         grep -iE "exactly.one.*primary|single.*primary|one.*pk" .sisyphus/evidence/task-5-bad-generated-multi/error.log
    Expected Result: error mentions exactly-one-pk requirement for Generated pk_type
    Evidence: .sisyphus/evidence/task-5-bad-generated-multi/error.log, .sisyphus/evidence/task-5-bad-generated-multi/main.rs.bak

  Scenario: Compile error when pk_type = "manual" + zero pk fields
    Tool: Bash
    Preconditions: feat/composite-pk branch checked out
    Steps:
      1. Temporarily patch UserRole model — remove all #[lorm(pk)] annotations:
         mkdir -p .sisyphus/evidence/task-5-bad-manual-empty
         cp lorm/tests/main.rs .sisyphus/evidence/task-5-bad-manual-empty/main.rs.bak
         perl -i -ne 'if (/pub struct UserRole/ .. /^\s*\}/) { print unless /^\s*#\[lorm\(pk\)\]/ } else { print }' lorm/tests/main.rs
      2. Attempt build (expect failure):
         cargo build --no-default-features --features sqlite -p lorm --tests 2>&1 | tee .sisyphus/evidence/task-5-bad-manual-empty/error.log ; true
      3. Restore original:
         cp .sisyphus/evidence/task-5-bad-manual-empty/main.rs.bak lorm/tests/main.rs
      4. Verify error message:
         grep -iE "at.least.one.*pk|at.least.one.*primary|requires.*pk" .sisyphus/evidence/task-5-bad-manual-empty/error.log
    Expected Result: error mentions at-least-one-pk requirement for Manual pk_type
    Evidence: .sisyphus/evidence/task-5-bad-manual-empty/error.log, .sisyphus/evidence/task-5-bad-manual-empty/main.rs.bak

  Scenario: PR created, CI green
    Tool: Bash
    Preconditions: branch pushed to origin
    Steps:
      1. Create PR and capture URL:
         gh pr create --title "feat(macros): support composite primary keys via pk_type = manual" --body "$(cat <<'PREOF'
         ## Summary
         - Introduce `PrimaryKey` enum (`Generated` | `Manual`) and `pk_type` struct-level attribute
         - Allow multiple `#[lorm(pk)]` fields when `pk_type = "manual"`
         - Generate composite key selector method (`by_key` or custom via `pk_selector`)
         - Update `delete()` to AND all pk columns for composite keys
         - Add UserRole and UserRoleNamed test models for all 3 backends
         PREOF
         )" | tee .sisyphus/evidence/task-5-pr-url.txt
      2. Wait for CI and capture:
         PR_URL=$(cat .sisyphus/evidence/task-5-pr-url.txt)
         gh pr checks "$PR_URL" --watch
         gh pr checks "$PR_URL" --json name,state,conclusion > .sisyphus/evidence/task-5-pr-checks.json
    Expected Result: all CI checks pass
    Evidence: .sisyphus/evidence/task-5-pr-url.txt, .sisyphus/evidence/task-5-pr-checks.json
  ```

  **Evidence to Capture**:
  - [ ] `.sisyphus/evidence/task-5-test-by-key-{sqlite,postgres,mysql}.log`
  - [ ] `.sisyphus/evidence/task-5-custom-selector.txt`
  - [ ] `.sisyphus/evidence/task-5-delete-where.txt`
  - [ ] `.sisyphus/evidence/task-5-backcompat.log`
  - [ ] `.sisyphus/evidence/task-5-bad-generated-multi/error.log`
  - [ ] `.sisyphus/evidence/task-5-bad-manual-empty/error.log`
  - [ ] `.sisyphus/evidence/task-5-pr-url.txt`
  - [ ] `.sisyphus/evidence/task-5-pr-checks.json`

  **Commit**: 7 atomic commits, branch `feat/composite-pk`, BREAKING-flagged PR (refactors `Column::should_generate_query_function` → `should_generate_selector` even though end-user API is purely additive; mark BREAKING because composite pk fields lose auto-by_<field> generation by default)
  - Pre-commit: `./format.sh && ./check.sh --package=lorm --feature=sqlite`
  - Pre-push: `./test.sh --package=lorm --feature=sqlite`

- [ ] 6. **feat/manual-pk-upsert — conditional upsert: Manual pk only; postgres/sqlite ON CONFLICT, mysql ON DUPLICATE KEY UPDATE; full-key DO NOTHING / INSERT IGNORE**

  **What to do**:
  - Wait for T5 PR merge
  - Branch: `git pull origin main && git checkout -b feat/manual-pk-upsert`
  - **Update `lorm-macros/src/orm/save.rs`** to branch on `model.primary_key.is_generated()`:
    - **Generated path** (current behavior, unchanged): the existing INSERT-or-UPDATE logic; `is_set` callable decides which branch
    - **Manual path** (NEW): always UPSERT in a single SQL statement
  - **Manual upsert SQL generation** — three sub-strategies by feature flag:
    - **postgres / sqlite**:
      - Compute `is_full_key`: are all non-readonly columns part of the pk?
      - If NOT full key: `INSERT INTO {table} ({insert_cols}) VALUES ({placeholders}) ON CONFLICT ({pk_cols}) DO UPDATE SET {non_pk_col} = EXCLUDED.{non_pk_col}, ... RETURNING {full_select}`
      - If full key: `INSERT INTO {table} ({insert_cols}) VALUES ({placeholders}) ON CONFLICT ({pk_cols}) DO NOTHING RETURNING {full_select}` — note `DO NOTHING` returns 0 rows on conflict, so add a fallback `SELECT WHERE pk = ...` if returning is empty (or just guarantee the SELECT for full-key case unconditionally)
    - **mysql**:
      - Compute `is_full_key` similarly
      - If NOT full key: `INSERT INTO {table} ({insert_cols}) VALUES ({placeholders}) ON DUPLICATE KEY UPDATE {non_pk_col} = VALUES({non_pk_col}), ...`
      - If full key: `INSERT IGNORE INTO {table} ({insert_cols}) VALUES ({placeholders})`
      - Followed by `SELECT WHERE pk = ?` to fetch the row (mysql has no RETURNING)
  - **Helper functions** to add in `lorm-macros/src/orm/save.rs`:
    - `fn build_upsert_clause_pg_sqlite(pk_cols: &[&str], non_pk_cols: &[&str]) -> String`
    - `fn build_upsert_clause_mysql(non_pk_cols: &[&str]) -> String`
    - `fn is_full_key(model: &OrmModel) -> bool` (returns true when every non-readonly column is in pk)
  - **Update tests added in T5**:
    - Re-enable `test_user_role_save_updates` (was `#[ignore]`-marked in T5): create UserRole, save (insert), modify `assigned_at`, save (upsert → update). Assert exactly one row exists with the new `assigned_at`.
    - Add `test_user_role_save_idempotent`: save same UserRole twice; assert still exactly one row.
    - Add `test_user_role_concurrent_save_different_keys`: saves to two distinct (user_id, role_id) pairs both succeed.
    - Add a "full key" model `Tag(name PK)` test where ALL columns are pk: `test_tag_full_key_upsert_idempotent` — save twice, assert one row, no error.
  - **Add migrations**: `10_tags_table.sql` for each backend (table with single column `name` as PRIMARY KEY)
  - **Update README**:
    - Update FAQ "How does composite primary keys work?" to mention save() now upserts for manual pks
    - Update Quickstart docs if any single-pk-only language is now wrong
  - **Update CHANGELOG**:
    - `### Added`: "save() now performs upsert for `pk_type = \"manual\"` models. Postgres/SQLite use `ON CONFLICT (pk_cols) DO UPDATE`; MySQL uses `ON DUPLICATE KEY UPDATE`. When all columns are pk columns, the operation degrades to `ON CONFLICT DO NOTHING` (postgres/sqlite) or `INSERT IGNORE` (mysql) and refetches the row."
    - `### Changed`: explicitly note that `pk_type = "generated"` save() behavior is unchanged (still INSERT-or-UPDATE based on `is_set`)
  - **Commits**:
    - `feat(macros): branch save() on PrimaryKey type (Generated vs Manual)`
    - `feat(macros): emit ON CONFLICT DO UPDATE for postgres/sqlite manual pk save`
    - `feat(macros): emit ON DUPLICATE KEY UPDATE for mysql manual pk save`
    - `feat(macros): handle is_full_key edge case (DO NOTHING / INSERT IGNORE)`
    - `test: enable composite pk upsert tests and add full-key Tag model`
    - `docs: document manual pk upsert behavior in README and CHANGELOG`
  - Push, PR, CI green, merge

  **Must NOT do**:
  - Do NOT change save() behavior for `Generated` pk type — owner explicitly forbade
  - Do NOT add a separate `upsert()` method — keep the API as `save()` (save semantics differ by pk_type, documented in CHANGELOG)
  - Do NOT silently emit empty SET clauses — full-key edge case must use `DO NOTHING` / `INSERT IGNORE`
  - Do NOT use postgres-only syntax for mysql or vice versa
  - Do NOT introduce new dependencies

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 3-backend SQL dialect handling; full-key edge case has subtle semantics; touches the most-exercised code path (save)
  - **Skills**: [`git-master`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: T7
  - **Blocked By**: T5

  **References**:

  *Pattern References*:
  - `lorm-macros/src/orm/save.rs:142-197` — current `(executor_bound, save_body)` cfg branching for mysql vs others; mirror the pattern for the new pk_type branching INSIDE each db branch
  - `lorm-macros/src/orm/save.rs:214-243` — `create_insert_placeholders` and `create_update_placeholders` helpers; reuse
  - `lorm-macros/src/utils.rs:106-117` — `db_placeholder` for db-specific `$N` vs `?`
  - `mrpine-big:lorm-macros/src/orm/save.rs` (his `Manual(..)` branch around line 75-110) — reference for upsert SQL building, but DO NOT copy MrPine's "always upsert" approach — he upserted for Generated too, which the owner forbade

  *API/Type References*:
  - SQLx postgres `ON CONFLICT`: https://docs.rs/sqlx/0.8/sqlx/postgres/index.html (also any SQLite docs reference for the same syntax)
  - SQLx mysql `ON DUPLICATE KEY UPDATE`: https://dev.mysql.com/doc/refman/8.0/en/insert-on-duplicate.html

  *Test References*:
  - `lorm/tests/main.rs:259-296` — save+update test patterns
  - T5's UserRole model (added in previous PR; will be in main when this branch starts)

  *External References*:
  - PostgreSQL `INSERT ... ON CONFLICT ... DO UPDATE SET col = EXCLUDED.col`: https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT
  - SQLite supports the same `ON CONFLICT` syntax: https://www.sqlite.org/lang_upsert.html
  - MySQL `INSERT IGNORE`: https://dev.mysql.com/doc/refman/8.0/en/insert.html
  - PR #2 thread comment 4103996426 (MrPine's clarification request) and 4104296957 (owner's "Yes we can do that"): the upsert-only-for-manual decision

  *WHY each reference matters*:
  - The dual-dialect branching is the heart of the SQL portability requirement
  - The full-key edge case is explicitly called out by owner ("this should be ON CONFLICT DO NOTHING, not silently omitted")
  - The MySQL fallback `SELECT` after `INSERT IGNORE` is necessary because MySQL has no RETURNING

  **Acceptance Criteria**:

  - [ ] `test_user_role_save_updates` passes on sqlite/postgres/mysql (was ignored in T5)
  - [ ] `test_user_role_save_idempotent` passes
  - [ ] `test_tag_full_key_upsert_idempotent` passes
  - [ ] `cargo expand` for UserRole save shows `ON CONFLICT (user_id, role_id) DO UPDATE SET assigned_at = EXCLUDED.assigned_at` (postgres/sqlite) or `ON DUPLICATE KEY UPDATE assigned_at = VALUES(assigned_at)` (mysql)
  - [ ] `cargo expand` for Tag save shows `ON CONFLICT (name) DO NOTHING` (postgres/sqlite) or `INSERT IGNORE` (mysql)
  - [ ] Generated pk models (User, AltUser, Profile, Customer) show UNCHANGED save SQL — verify with `cargo expand` diff against pre-T6 reference

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: Manual pk save() upserts on postgres
    Tool: Bash
    Steps:
      1. ./test.sh --package=lorm --feature=postgres -- test_user_role_save 2>&1 | tee .sisyphus/evidence/task-6-test-postgres.log
    Expected Result: tests pass; SQL log shows ON CONFLICT
    Evidence: .sisyphus/evidence/task-6-test-postgres.log

  Scenario: Manual pk save() upserts on sqlite
    Tool: Bash
    Steps:
      1. ./test.sh --package=lorm --feature=sqlite -- test_user_role_save 2>&1 | tee .sisyphus/evidence/task-6-test-sqlite.log
    Expected Result: pass; SQL log shows ON CONFLICT
    Evidence: .sisyphus/evidence/task-6-test-sqlite.log

  Scenario: Manual pk save() upserts on mysql
    Tool: Bash
    Steps:
      1. ./test.sh --package=lorm --feature=mysql -- test_user_role_save 2>&1 | tee .sisyphus/evidence/task-6-test-mysql.log
    Expected Result: pass; SQL log shows ON DUPLICATE KEY UPDATE
    Evidence: .sisyphus/evidence/task-6-test-mysql.log

  Scenario: Full-key model degrades to DO NOTHING / INSERT IGNORE
    Tool: Bash
    Steps:
      1. ./test.sh --package=lorm --feature=sqlite -- test_tag 2>&1 | tee .sisyphus/evidence/task-6-fullkey-sqlite.log
      2. ./test.sh --package=lorm --feature=postgres -- test_tag 2>&1 | tee .sisyphus/evidence/task-6-fullkey-postgres.log
      3. ./test.sh --package=lorm --feature=mysql -- test_tag 2>&1 | tee .sisyphus/evidence/task-6-fullkey-mysql.log
    Expected Result: all 3 pass; double-save is idempotent (1 row); no SQL error
    Evidence: .sisyphus/evidence/task-6-fullkey-{sqlite,postgres,mysql}.log

  Scenario: Generated pk save behavior is preserved (no regression)
    Tool: Bash
    Steps:
      1. ./test.sh --package=lorm --feature=sqlite -- test_user_is_created test_user_is_updated 2>&1 | tee .sisyphus/evidence/task-6-generated-unchanged.log
      2. cargo expand --test main --no-default-features --features sqlite | grep -A 3 "INSERT INTO users" > .sisyphus/evidence/task-6-user-insert-sql.txt
      3. Diff with reference: ensure no ON CONFLICT in the User model's INSERT (Generated pk)
    Expected Result: tests pass; User INSERT does NOT contain ON CONFLICT
    Evidence: .sisyphus/evidence/task-6-generated-unchanged.log, .sisyphus/evidence/task-6-user-insert-sql.txt

  Scenario: PR created, CI green
    Tool: Bash
    Preconditions: branch pushed to origin
    Steps:
      1. Create PR and capture URL:
         gh pr create --title "feat(macros): conditional upsert for manual pk save()" --body "$(cat <<'PREOF'
         ## Summary
         - Branch save() on PrimaryKey type: Generated (unchanged) vs Manual (upsert)
         - Postgres/SQLite: `ON CONFLICT (pk_cols) DO UPDATE SET ...`
         - MySQL: `ON DUPLICATE KEY UPDATE ...`
         - Full-key edge case: `DO NOTHING` / `INSERT IGNORE` when all columns are pk
         - Add Tag full-key model; enable composite pk upsert tests
         PREOF
         )" | tee .sisyphus/evidence/task-6-pr-url.txt
      2. Wait for CI and capture:
         PR_URL=$(cat .sisyphus/evidence/task-6-pr-url.txt)
         gh pr checks "$PR_URL" --watch
         gh pr checks "$PR_URL" --json name,state,conclusion > .sisyphus/evidence/task-6-pr-checks.json
    Expected Result: all CI checks pass
    Evidence: .sisyphus/evidence/task-6-pr-url.txt, .sisyphus/evidence/task-6-pr-checks.json
  ```

  **Evidence to Capture**:
  - [ ] `.sisyphus/evidence/task-6-test-{sqlite,postgres,mysql}.log`
  - [ ] `.sisyphus/evidence/task-6-fullkey-{sqlite,postgres,mysql}.log`
  - [ ] `.sisyphus/evidence/task-6-generated-unchanged.log`
  - [ ] `.sisyphus/evidence/task-6-user-insert-sql.txt`
  - [ ] `.sisyphus/evidence/task-6-pr-url.txt`
  - [ ] `.sisyphus/evidence/task-6-pr-checks.json`

  **Commit**: 6 atomic commits, branch `feat/manual-pk-upsert`
  - Pre-commit: `./format.sh && ./check.sh --package=lorm --feature=sqlite`
  - Pre-push: `./test.sh --package=lorm --feature=sqlite`

- [ ] 7. **chore/release-0.3.0 — bump version, regenerate CHANGELOG, add composite_pk example, tag**

  **What to do**:
  - Wait for T6 PR merge
  - Branch: `git pull origin main && git checkout -b chore/release-0.3.0`
  - **Add new example `examples/composite_pk.rs`**: a runnable example demonstrating the UserRole composite-pk pattern with save/upsert/by_key/delete on in-memory SQLite. Add `[[example]] name = "composite_pk" path = "../examples/composite_pk.rs"` block to `lorm/Cargo.toml`. Add to `examples/README.md`. Add a step to the `examples` job in `.github/workflows/ci.yml` to run the new example.
  - **Run `./bump-version.sh --minor`** to bump workspace version from 0.2.2 → 0.3.0 and regenerate CHANGELOG.md via git-cliff. Review the auto-generated CHANGELOG; clean up to ensure all the manual `## [Unreleased]` entries from T1-T6 are preserved and reorganized under `## [0.3.0] - 2026-MM-DD` with sections: `### Added`, `### Changed` (including BREAKING), `### Fixed`. Add a top-level "Migration Guide" callout linking to:
    - `is_set` Callable change (T2)
    - composite pk additions (T5)
    - upsert behavior for manual pk (T6)
  - **Verify all 4 examples compile and run** on default sqlite:
    ```bash
    cargo run --example basic_crud -p lorm
    cargo run --example query_builder -p lorm
    cargo run --example transactions -p lorm
    cargo run --example composite_pk -p lorm
    ```
  - **Re-read README**: ensure attribute reference table is consistent end-to-end after all 6 feature merges. Replace any "single-pk only" wording surviving in lib.rs doc comments. Verify FAQ entries for composite-pk, flatten, json are present.
  - **Verify forbidden patterns absent**:
    ```bash
    ! grep -q "proc-macro-error" lorm-macros/Cargo.toml lorm-macros/src/**/*.rs
    ! grep -E '^proc-macro2 *=' lorm-macros/Cargo.toml
    ! grep -rn 'is_set = "is_nil()"' lorm/ examples/ lorm-macros/
    ```
  - **Commits**:
    - `docs(examples): add composite_pk example`
    - `ci: run composite_pk example in examples CI job`
    - `chore(release): prepare for v0.3.0`
  - Push, `gh pr create` titled `chore(release): prepare for v0.3.0`, body lists migration guide, all merged feature PRs, breaking changes
  - **After PR merge to main**:
    - `git checkout main && git pull`
    - `git tag -s v0.3.0 -m "Release v0.3.0 - composite pk, flatten, json, manual pk upsert"`
    - `git push origin v0.3.0` — triggers release workflow if configured

  **Must NOT do**:
  - Do NOT introduce any new feature in this branch — pure release plumbing
  - Do NOT skip running the examples (they're a regression smoke test)
  - Do NOT publish to crates.io as part of this branch — owner controls that step manually post-merge

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Mostly mechanical (script-driven version bump, example creation, tag)
  - **Skills**: [`git-master`]
    - `git-master`: Tag signing, conventional commit footers

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential (last task before final review wave)
  - **Blocks**: F1, F2, F3, F4
  - **Blocked By**: T6

  **References**:

  *Pattern References*:
  - `bump-version.sh` (root) — automation
  - `cliff.toml` (root) — git-cliff template
  - `lorm/Cargo.toml:38-52` — `[[example]]` blocks; mirror format
  - `examples/basic_crud.rs` — runnable-example structure
  - `.github/workflows/ci.yml` `examples` job — add `composite_pk` step
  - Past release commits: `2143eb8 chore(release): prepare for v0.2.0`, `2c8b69b chore(release): prepare for v0.1.0`, `0836b91 chore(release): prepare for v0.2.2`

  *External References*:
  - Keep a Changelog: https://keepachangelog.com/en/1.1.0/
  - SemVer pre-1.0 rules: https://semver.org/#spec-item-4

  *WHY each reference matters*:
  - `bump-version.sh` already exists and integrates git-cliff — DO NOT manually edit Cargo.toml versions; use the script
  - The `examples/README.md` and CI job both must list the new example or it'll silently rot

  **Acceptance Criteria**:

  - [ ] `Cargo.toml` workspace `version = "0.3.0"`
  - [ ] `lorm/Cargo.toml` and `lorm-macros/Cargo.toml` `lorm-macros = { ..., version = "0.3.0", ... }` reference updated
  - [ ] CHANGELOG.md has `## [0.3.0] - 2026-MM-DD` block with all changes from T1-T6 organized
  - [ ] `composite_pk` example compiles, runs, and is referenced in `lorm/Cargo.toml`, `examples/README.md`, and CI workflow
  - [ ] All 4 examples run successfully on sqlite (capture output)
  - [ ] `git tag --list "v0.3.0"` returns `v0.3.0` (after push)
  - [ ] No forbidden patterns survive in main

  **QA Scenarios (MANDATORY)**:

  ```
  Scenario: Workspace builds and tests pass at v0.3.0
    Tool: Bash
    Steps:
      1. cargo build --workspace --no-default-features --features sqlite 2>&1 | tee .sisyphus/evidence/task-7-build-sqlite.log
      2. cargo build --workspace --no-default-features --features postgres 2>&1 | tee .sisyphus/evidence/task-7-build-postgres.log
      3. cargo build --workspace --no-default-features --features mysql 2>&1 | tee .sisyphus/evidence/task-7-build-mysql.log
      4. ./test.sh --package=lorm --feature=sqlite 2>&1 | tee .sisyphus/evidence/task-7-test-sqlite.log
    Expected Result: all builds + tests succeed
    Evidence: .sisyphus/evidence/task-7-build-{sqlite,postgres,mysql}.log, task-7-test-sqlite.log

  Scenario: All 4 examples run and exit 0
    Tool: Bash
    Steps:
      1. cargo run --example basic_crud -p lorm 2>&1 | tee .sisyphus/evidence/task-7-example-basic_crud.log
      2. cargo run --example query_builder -p lorm 2>&1 | tee .sisyphus/evidence/task-7-example-query_builder.log
      3. cargo run --example transactions -p lorm 2>&1 | tee .sisyphus/evidence/task-7-example-transactions.log
      4. cargo run --example composite_pk -p lorm 2>&1 | tee .sisyphus/evidence/task-7-example-composite_pk.log
    Expected Result: all 4 exit 0; logs show expected outputs (created, queried, deleted)
    Failure Indicators: any non-zero exit, any panic
    Evidence: .sisyphus/evidence/task-7-example-*.log

  Scenario: Forbidden patterns absent
    Tool: Bash
    Steps:
      1. ! grep -q "proc-macro-error" lorm-macros/Cargo.toml ; echo "no-pme: $?" > .sisyphus/evidence/task-7-forbidden.txt
      2. ! grep -E '^proc-macro2 *=' lorm-macros/Cargo.toml ; echo "no-pm2: $?" >> .sisyphus/evidence/task-7-forbidden.txt
      3. ! grep -rn 'is_set = "is_nil()"' lorm/ examples/ lorm-macros/ ; echo "no-old-is-set: $?" >> .sisyphus/evidence/task-7-forbidden.txt
    Expected Result: all three "no-..." lines = 0 (grep found nothing, ! flipped to success)
    Evidence: .sisyphus/evidence/task-7-forbidden.txt

  Scenario: v0.3.0 tag created and pushed
    Tool: Bash
    Steps:
      1. git tag --list "v0.3.0" > .sisyphus/evidence/task-7-tag.txt
      2. git ls-remote --tags origin | grep v0.3.0 >> .sisyphus/evidence/task-7-tag.txt
    Expected Result: file contains v0.3.0 in both local and remote
    Evidence: .sisyphus/evidence/task-7-tag.txt

  Scenario: PR created, CI green, merged, tagged
    Tool: Bash
    Preconditions: branch pushed to origin
    Steps:
      1. Create PR and capture URL:
         gh pr create --title "chore(release): prepare for v0.3.0" --body "$(cat <<'PREOF'
         ## Summary
         Release v0.3.0 — composite pk, flatten, json, manual pk upsert.

         ### Migration Guide
         - `is_set` Callable: `#[lorm(is_set = "is_nil()")]` → `#[lorm(is_set = "Uuid::is_nil")]`
         - Composite pk: use `#[lorm(pk_type = "manual")]` + multiple `#[lorm(pk)]`
         - Manual pk save() now upserts (ON CONFLICT / ON DUPLICATE KEY UPDATE)

         ### New Features
         - `#[sqlx(json)]` support
         - `#[sqlx(flatten)]` + `#[lorm(flattened(...))]` support
         - Composite primary keys
         - Conditional upsert for manual pk
         PREOF
         )" | tee .sisyphus/evidence/task-7-pr-url.txt
      2. Wait for CI and capture:
         PR_URL=$(cat .sisyphus/evidence/task-7-pr-url.txt)
         gh pr checks "$PR_URL" --watch
         gh pr checks "$PR_URL" --json name,state,conclusion > .sisyphus/evidence/task-7-pr-checks.json
    Expected Result: all CI checks pass
    Evidence: .sisyphus/evidence/task-7-pr-url.txt, .sisyphus/evidence/task-7-pr-checks.json
  ```

  **Evidence to Capture**:
  - [ ] `.sisyphus/evidence/task-7-build-{sqlite,postgres,mysql}.log`
  - [ ] `.sisyphus/evidence/task-7-test-sqlite.log`
  - [ ] `.sisyphus/evidence/task-7-example-{basic_crud,query_builder,transactions,composite_pk}.log`
  - [ ] `.sisyphus/evidence/task-7-forbidden.txt`
  - [ ] `.sisyphus/evidence/task-7-tag.txt`
  - [ ] `.sisyphus/evidence/task-7-pr-url.txt`
  - [ ] `.sisyphus/evidence/task-7-pr-checks.json`

  **Commit**: 3 atomic commits, branch `chore/release-0.3.0`, signed tag `v0.3.0` after merge
  - Pre-commit: `./format.sh && ./check.sh --package=lorm --feature=sqlite`
  - Pre-push: `./test.sh --package=lorm --feature=sqlite` AND all 4 examples run

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
>
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback → fix → re-run → present again → wait for okay.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read this plan end-to-end. For each "Must Have": verify implementation exists in main (`gh api repos/remysaissy/lorm/contents/...` or `git show main:...`). For each "Must NOT Have": grep main for forbidden patterns — reject with file:line if found. Verify all 7 PRs from this plan are present in `gh pr list --state merged` with conventional-commit titles. Verify v0.3.0 tag exists. Check evidence files exist in `.sisyphus/evidence/`.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | PRs Merged [7/7] | Tag v0.3.0 [PRESENT/ABSENT] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `./format.sh --check`, `./check.sh --package=lorm --feature=sqlite`, `./check.sh --package=lorm --feature=postgres`, `./check.sh --package=lorm --feature=mysql`, `./test.sh --package=lorm --feature=sqlite`, plus the CI workflow via `./test.sh` (act). Review all changed files for: `as any`, `unwrap()` in non-test paths, empty catches, dbg!/println! in macro emit, commented-out code, dead helpers, AI slop (excessive comments, over-abstraction, generic names like `data`/`result`/`temp`).
  Output: `Format [PASS/FAIL] | Clippy×3 [PASS/FAIL] | Tests×3 [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high`
  Start from clean checkout of main at v0.3.0 tag. Execute EVERY QA scenario from EVERY task. Run all 4 examples (`basic_crud`, `query_builder`, `transactions`, `composite_pk`). Test cross-task integration: a model that uses BOTH json AND flatten AND composite pk in one struct (the ultimate stress test). Test edge cases: all-pk-column upsert, Option<NestedFlatten> with None, json field in WHERE clause. Save evidence to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Examples [4/4] | Stress Model [PASS/FAIL] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read the "What to do", read actual diffs (`gh pr diff <num>` for each of the 7 PRs). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance per task. Verify each PR's commits use conventional commits. Detect cross-task contamination: Task N touching Task M's files outside of explicit handoffs. Flag any unaccounted changes.
  Output: `Tasks [7/7 compliant] | Conventional Commits [PASS/FAIL] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

Each task IS its own feature branch IS its own PR. Commit hygiene rules:

- **Conventional Commits**: `<type>(<scope>): <subject>` — types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `style`
- **Atomic commits within a PR**: small, reviewable, each green on `./check.sh` + `./test.sh`
- **Branch naming**: `chore/cleanup-and-fixes`, `feat/is-set-callable`, `feat/sqlx-json`, `feat/sqlx-flatten`, `feat/composite-pk`, `feat/manual-pk-upsert`, `chore/release-0.3.0`
- **PR titles** = first commit subject (squash-friendly)
- **PR bodies**: include full commit message body and **BREAKING CHANGE:** footer for T2/T5
- **Pre-commit per branch**: `./format.sh && ./check.sh --package=lorm --feature=sqlite && ./test.sh --package=lorm --feature=sqlite` (and same for postgres + mysql when feasible locally)
- **Signing**: gitsign / GPG / SSH per `CONTRIBUTING.md` — required (CI verifies)
- **Cargo.lock**: committed if it changes

---

## Success Criteria

### Verification Commands

```bash
# Workspace builds for each backend
cargo build --workspace --no-default-features --features sqlite
cargo build --workspace --no-default-features --features postgres
cargo build --workspace --no-default-features --features mysql

# Tests pass per backend
./test.sh --package=lorm --feature=sqlite
./test.sh --package=lorm --feature=postgres
./test.sh --package=lorm --feature=mysql

# Clippy clean per backend
./check.sh --package=lorm --feature=sqlite
./check.sh --package=lorm --feature=postgres
./check.sh --package=lorm --feature=mysql

# Format clean
./format.sh --check

# Coverage thresholds
./coverage.sh --check-thresholds

# Examples all run on default sqlite
cargo run --example basic_crud -p lorm
cargo run --example query_builder -p lorm
cargo run --example transactions -p lorm
cargo run --example composite_pk -p lorm  # NEW

# 7 PRs merged
gh pr list --repo remysaissy/lorm --state merged --search "in:title chore/cleanup-and-fixes OR feat/is-set-callable OR feat/sqlx-json OR feat/sqlx-flatten OR feat/composite-pk OR feat/manual-pk-upsert OR chore/release-0.3.0"

# Tag exists
git tag --list "v0.3.0"

# Forbidden deps absent
! grep -q "proc-macro-error2" lorm-macros/Cargo.toml
! grep -E "^proc-macro2 *=" lorm-macros/Cargo.toml

# Migration scrubbed
! grep -r 'is_set = "is_nil()"' lorm/ examples/
```

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All 3 SQLx backend feature combinations pass tests
- [ ] CHANGELOG.md `## [0.3.0]` complete with Added / Changed / Fixed / **BREAKING CHANGES**
- [ ] README.md attribute table updated for: `pk_type`, `pk_selector`, `flattened`, `#[sqlx(json)]`, `#[sqlx(flatten)]`
- [ ] FAQ section updated: composite pk, flatten, json answers replace stale "currently supports single-field primary keys only" line
- [ ] `Cargo.toml` workspace version = `0.3.0`
- [ ] `v0.3.0` git tag pushed
