## Summary
Pre-0.3.0 cleanup addressing items from PR #2 review:
- Fix `new` expression attribute validation error message (was incorrectly saying "is_set attribute")
- Add duplicate-detection for `#[lorm(created_at)]` and `#[lorm(updated_at)]` attributes
- Add `#[automatically_derived]` to all generated impl blocks
- Remove dead code and clean examples
- Update CHANGELOG with unreleased entries
