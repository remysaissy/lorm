# Decisions — finish-mrpine-vision

## [2026-04-29] From Plan Analysis

- `is_set` must migrate from `syn::Expr` to `darling::util::Callable` (BREAKING)
- Upsert only for `pk_type = "manual"` — `Generated` pk_type behavior unchanged (owner explicitly forbade)
- Single-field manual pk → `by_<field>()`, composite → `by_key()` (or custom via `pk_selector`)
- `#[sqlx(json(nullable))]` explicitly NOT supported this round
- No new dependencies beyond what's already in Cargo.toml (serde_json to dev-deps only for T3)
- `#[automatically_derived]` goes on every generated `impl ... { ... }` block
- CHANGELOG migration blocks required for T2 (is_set) and T5/T6 (composite pk)
