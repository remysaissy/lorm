# lorm Org, Governance & Community

## TL;DR

> **Quick Summary**: Migrate the lorm repository to a `lorm-rs/lorm` GitHub organization for bus-factor signal, recruit a second maintainer, publish MAINTAINERS.md with a response-time commitment, set up GitHub Sponsors, define a non-pushy community engagement policy, and adopt DCO for inbound contributions. Re-wire crates.io Trusted Publishing for the new repo URL (CRITICAL — this is `lorm-1.0-stability.md` T13's blocker).
>
> **Deliverables**:
> - GitHub organization `lorm-rs` (or fallback name if taken) with `lorm-rs/lorm` repository
> - MAINTAINERS.md with named maintainers and response-time SLA
> - DCO sign-off requirement (alternative to CLA)
> - GOVERNANCE.md (lightweight)
> - SECURITY.md
> - GitHub Sponsors / Open Collective setup (optional but documented)
> - All URL references atomically updated (Cargo.toml repository, README badges, CONTRIBUTING.md, Trusted Publishing workflow)
> - Community Engagement Policy (`docs/community-engagement-policy.md` or `.github/COMMUNITY.md`)
> - Identified candidate community discussion threads tracked in `.sisyphus/drafts/community-engagement-tracker.md`
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES — 4 waves
> **Critical Path**: T1 (org name verify) → T5 (atomic URL migration PR) → T6 (Trusted Publishing re-wire) → T8 (recruit 2nd maintainer)

---

## Context

### Original Request
Implement feedback items #4 and #7:
- **#4 Bus-factor signal**: *"Add a second maintainer or transfer to an org (even a personal one like `lorm-rs/lorm`). Solo-maintainer crates get filtered out of production dep lists. A `MAINTAINERS.md` with a stated response-time commitment helps."*
- **#7 Active in discussion spaces**: *"Answer every 'what ORM should I use' question on Reddit r/rust, the Rust users forum, Discord, with a non-pushy 'lorm might fit if you want X, here's why.' Be the maintainer who shows up. Adoption compounds from a hundred small conversations."*

### Metis Review (addressed)
- **B2 (Critical)**: GitHub org `lorm-rs` availability is **unverified**. Task 1 verifies before any public naming.
- **B3 (Critical)**: Trusted Publishing breaks on URL change. Task 6 atomically re-wires.
- **Trademark gap**: Apache-2.0 doesn't grant trademark rights; informal name claim only. Documented in GOVERNANCE.md, not blocking.
- **CLA vs DCO**: Adopt DCO (commit `Signed-off-by:` line) rather than a heavyweight CLA. Already requires signed commits.
- **Response-time SLA**: Don't commit publicly to a SLA before 2nd maintainer onboards (one person can't realistically guarantee 48h). Task 7 (MAINTAINERS.md) sequenced AFTER Task 8 (2nd maintainer).
- **crates.io ownership model**: crates.io supports individual owners + teams (no native "org-level" ownership). Use co-ownership.
- **Community engagement spam risk**: Codify a policy (Task 11) before engaging.

### Research Findings (from stability audit)
- Current repository: `github.com/remysaissy/lorm` (per Cargo.toml `repository` field)
- Workflow `.github/workflows/release.yml` uses Trusted Publishing (id-token: write) — tied to `repository_owner/repository_name`
- CONTRIBUTING.md already requires signed commits (GPG/SSH/S/MIME) — DCO is additive, not replacement
- Apache-2.0 license (no CLA needed for inbound)
- Issue templates already present at `.github/ISSUE_TEMPLATE/`

---

## Work Objectives

### Core Objective
Migrate from solo-maintained `remysaissy/lorm` to organization-owned `lorm-rs/lorm` (or fallback), bootstrap governance documents that signal a sustainable project, and establish a written non-spammy community engagement policy.

### Concrete Deliverables
- `lorm-rs` GitHub org (or fallback name) — verified, created, configured
- `lorm-rs/lorm` repository (transferred from `remysaissy/lorm` with redirect)
- `MAINTAINERS.md` listing named maintainers, contact methods, response-time SLA (≤7 calendar days for issues, ≤14 for PRs)
- `GOVERNANCE.md` — decision-making process, role expectations, escalation path
- `SECURITY.md` — vulnerability disclosure policy (private email)
- `.github/COMMUNITY.md` or `docs/community-engagement-policy.md` — codified non-pushy outreach rules
- Updated `Cargo.toml` (workspace + per-crate) `repository = "https://github.com/lorm-rs/lorm"`
- Updated README badges, CONTRIBUTING.md, all internal URL references
- Re-wired Trusted Publishing on `.github/workflows/release.yml` for the new path
- Second maintainer onboarded (named, GitHub handle public on MAINTAINERS.md, with crates.io co-owner status on both crates)
- DCO bot added to `.github/` and required on PRs
- Optional: GitHub Sponsors button enabled, FUNDING.yml configured
- Engagement tracker at `.sisyphus/drafts/community-engagement-tracker.md`

### Definition of Done
- [ ] `gh repo view lorm-rs/lorm` returns the migrated repo
- [ ] CI on `lorm-rs/lorm/main` is green
- [ ] `gh workflow run release.yml --ref main` dry-run (`./release.sh --dry-run` if supported) — Trusted Publishing path correct
- [ ] MAINTAINERS.md lists ≥ 2 named maintainers
- [ ] DCO check is required on PRs
- [ ] Old `remysaissy/lorm` redirects to new repo
- [ ] crates.io shows ≥ 2 owners on `lorm` AND `lorm-macros`
- [ ] Community engagement policy committed and linked from README

### Must Have
- Org name verified before any public commitment
- Atomic URL migration in a single PR (Cargo.toml + README + CONTRIBUTING + workflow paths) to prevent partial states
- Trusted Publishing verified working under new URL BEFORE any 1.0 publish attempt
- ≥ 2 maintainers with crates.io co-owner status
- MAINTAINERS.md drafted AFTER 2nd maintainer onboards (don't promise SLA solo)
- DCO sign-off enforced on PRs
- Community engagement policy committed before active outreach starts (Task 11)

### Must NOT Have (Guardrails)
- NO public mention of `lorm-rs` org name until Task 1 verifies availability
- NO commitment to response-time SLA before Task 8 onboards a 2nd maintainer (single point of failure)
- NO mass community posting before Task 11 publishes the engagement policy
- NO trash-talking other ORMs in any engagement
- NO posting in threads where established solutions (SeaORM, Diesel, sqlx direct) are clearly the right answer
- NO alt accounts or coordinated promotion
- NO transfer of the repository before Trusted Publishing is verified working under new URL
- NO change to license (stays Apache-2.0)
- NO renaming of the crates on crates.io (only repo path changes)
- NO public announcement of org migration on the same week as 1.0 launch (don't dilute the launch signal — schedule them ≥ 2 weeks apart)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (gh CLI, git tooling)
- **Automated tests**: N/A for governance tasks
- **Verification**: Agent-executed `gh` commands + manual screenshot evidence where needed

### QA Policy
Agent-executed via `gh` CLI. Evidence saved to `.sisyphus/evidence/governance-task-{N}-{slug}.{txt,json}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — START IMMEDIATELY, 2 parallel):
├── T1: Verify lorm-rs GitHub org name availability + decide fallback   [quick]
└── T2: Draft GOVERNANCE.md, SECURITY.md, FUNDING.yml templates         [writing]

Wave 2 (Org bootstrap — sequential after T1):
├── T3: Create org + configure settings + import repo (with redirect) [unspecified-high]
└── T4: Set up DCO bot + branch protections                            [quick]

Wave 3 (URL migration + crates.io — 2 parallel):
├── T5: Atomic URL migration PR (Cargo.toml + README + CONTRIBUTING + .github/) [quick]
└── T6: Trusted Publishing re-wire + dry-run verification              [unspecified-high]

Wave 4 (Maintainer recruitment + community policy — 3 parallel):
├── T7: Set up GitHub Sponsors / Open Collective                       [quick]
├── T8: Identify + recruit 2nd maintainer (manual outreach)            [unspecified-high]
├── T9: Add 2nd maintainer as crates.io co-owner on both crates        [quick]
├── T10: Publish MAINTAINERS.md + response-time SLA                    [writing]
└── T11: Publish community engagement policy + populate tracker        [writing]

Wave FINAL (4 parallel reviews → user okay):
├── F1: Plan compliance audit                                          [oracle]
├── F2: Code/config quality review                                     [unspecified-high]
├── F3: Manual QA via gh CLI                                           [unspecified-high]
└── F4: Scope fidelity check                                           [deep]

Critical Path: T1 → T3 → T5 → T6 → T8 → T10 → FINAL → user okay
```

### Dependency Matrix
- T1 → T3 (need verified name)
- T2 → T3 (governance docs go into repo at creation)
- T3 → T4, T5, T6
- T5 → T6 (URLs must be updated before Trusted Publishing re-test)
- T6 → unblocks `lorm-1.0-stability.md` T13
- T7 → T10 (optional sponsor link in MAINTAINERS.md)
- T8 → T9 → T10 (maintainer must be onboarded before MAINTAINERS.md commits to them; crates.io co-ownership before SLA)
- T11 → unblocks `lorm-content-ecosystem.md` community outreach tasks

---

## TODOs

- [x] 1. **Verify `lorm-rs` GitHub org name availability + decide fallback**

  **What to do**:
  - Run `gh api orgs/lorm-rs 2>&1` and `gh api orgs/lorm 2>&1` and `gh api orgs/lorm-project 2>&1`.
  - Check crates.io: `cargo search 'name == "lorm-rs"'` and verify the org name doesn't clash with a published crate.
  - Pick the first available name in priority order: `lorm-rs`, `lorm-org`, `lorm-project`, `lorm-dev`.
  - Record the decision in `.sisyphus/drafts/org-name-decision.md`.

  **Must NOT do**:
  - Do NOT use a name that has a public repo or popular crate
  - Do NOT contact GitHub Support to request a squatted name (slow + uncertain)
  - Do NOT use the user's personal-account-style name (`remysaissy-org`)

  **Recommended Agent Profile**: `quick`. Skills: `[]`.

  **Parallelization**: Wave 1. Blocks T3.

  **References**:
  - `gh api orgs/<name>` returns 404 if not taken, 200 if taken
  - crates.io API: https://crates.io/api/v1/crates?q=lorm

  **WHY**: Public naming is permanent (changing it later breaks bookmarks/badges/SEO). Verify before any commitment.

  **Acceptance Criteria**:
  - [ ] Decision recorded with reasoning
  - [ ] 4 candidate names checked

  **QA Scenarios**:
  ```
  Scenario: Org name availability verified
    Tool: Bash (gh)
    Steps:
      1. for n in lorm-rs lorm-org lorm-project lorm-dev; do echo "$n:"; gh api "orgs/$n" 2>&1 | head -1; done
      2. Pick first 404
    Expected Result: decision file references the chosen name
    Evidence: .sisyphus/evidence/governance-task-1-name-check.txt
  ```

  **Commit**: NO (decision file is in .sisyphus/, not the repo)

- [x] 2. **Draft GOVERNANCE.md, SECURITY.md, FUNDING.yml templates**

  **What to do**:
  - Draft `GOVERNANCE.md` covering:
    - Project roles (Maintainer, Contributor, Triager)
    - Decision-making (lazy consensus on PRs; named maintainer can break ties)
    - Escalation if a maintainer is unresponsive > 2 weeks
    - How to become a maintainer (sustained PR contributions; nominated by existing maintainer; opt-in)
  - Draft `SECURITY.md`:
    - Private disclosure email (e.g., `security@lorm-rs.org` or maintainer GPG-encrypted)
    - 90-day disclosure window
    - Acknowledgment timeline
    - No bug bounty (volunteer project)
  - Draft `.github/FUNDING.yml`:
    - GitHub Sponsors target (per-maintainer or org level)
    - Optional Open Collective link
  - Draft `.github/dependabot.yml` if not present.

  **Must NOT do**:
  - Do NOT commit a bug bounty
  - Do NOT name a security contact who hasn't consented
  - Do NOT enable funding sources before maintainers consent

  **Recommended Agent Profile**: `writing`. Skills: `[]`.

  **Parallelization**: Wave 1. Blocks T3.

  **References**:
  - GitHub default community standards: https://docs.github.com/en/communities
  - tokio-rs/tokio MAINTAINERS.md as a model
  - rust-lang/rust GOVERNANCE.md as a model (lightweight subset)

  **WHY**: Templates ready at org creation = repo looks mature from day 1.

  **Acceptance Criteria**:
  - [ ] All 3 files exist as drafts in `.sisyphus/drafts/`
  - [ ] GOVERNANCE.md has the 4 sub-sections listed

  **QA Scenarios**:
  ```
  Scenario: Templates are reviewable
    Tool: Read
    Steps:
      1. Read each of the 3 draft files
      2. Verify required sections present
    Expected Result: 3/3 files complete
    Evidence: .sisyphus/evidence/governance-task-2-templates.txt
  ```

  **Commit**: NO (drafts only; commit in T3 when repo is ready)

- [x] 3. **Create org + configure settings + import repo (with redirect)**

  **What to do**:
  - `gh api -X POST orgs --raw-field name=<chosen-name>` (or via web UI if API is restricted) — create the org.
  - Configure org settings: 2FA required for members, default repo visibility public, default branch `main`.
  - Transfer `remysaissy/lorm` to `<chosen-name>/lorm`: `gh repo transfer remysaissy/lorm <chosen-name>/lorm` (or web UI). GitHub automatically redirects old URL.
  - **Verify the redirect** by running `curl -sI https://github.com/remysaissy/lorm` and confirming `Location:` header points to the new URL.
  - Commit `GOVERNANCE.md`, `SECURITY.md`, `.github/FUNDING.yml` (from T2 drafts) directly on the new repo.

  **Must NOT do**:
  - Do NOT delete the old `remysaissy/lorm` redirect stub (it's needed for backward links)
  - Do NOT rename the repo while transferring (single-axis change)
  - Do NOT transfer without first committing T2's governance docs (saves CI runs)

  **Recommended Agent Profile**: `unspecified-high`. Skills: `[]`.

  **Parallelization**: Wave 2 (sequential after T1, T2). Blocks T4, T5, T6.

  **References**:
  - gh CLI repo transfer: `gh repo transfer --help`
  - GitHub org creation API: https://docs.github.com/en/rest/orgs

  **WHY**: Transfer + redirect is reversible (within 90 days) but should be done atomically to minimize CI downtime.

  **Acceptance Criteria**:
  - [ ] `gh repo view <new-org>/lorm` succeeds
  - [ ] `curl -sI https://github.com/remysaissy/lorm` returns 301/302 to new URL
  - [ ] GOVERNANCE.md, SECURITY.md, FUNDING.yml present in new repo

  **QA Scenarios**:
  ```
  Scenario: Repo accessible at new URL
    Tool: Bash (gh)
    Steps:
      1. gh repo view <new-org>/lorm --json url
    Expected Result: returns valid JSON with correct URL
    Evidence: .sisyphus/evidence/governance-task-3-newrepo.txt

  Scenario: Redirect from old URL
    Tool: Bash (curl)
    Steps:
      1. curl -sI https://github.com/remysaissy/lorm
    Expected Result: 301/302 with Location pointing to new repo
    Evidence: .sisyphus/evidence/governance-task-3-redirect.txt
  ```

  **Commit**: YES (in new repo): `docs(governance): add GOVERNANCE/SECURITY/FUNDING`

- [x] 4. **Set up DCO bot + branch protections**

  **What to do**:
  - Add DCO via [probot/dco](https://github.com/apps/dco) GitHub App or equivalent (Conform action).
  - Update CONTRIBUTING.md: add a "Sign your commits" section explaining `git commit -s` adds the DCO `Signed-off-by:` trailer.
  - Configure branch protection on `main`:
    - Required reviews: 1
    - Required status checks: format, check, feature-matrix (sqlite/postgres/mysql), coverage, examples, DCO, semver-check (added in `lorm-1.0-stability.md` T12)
    - Require linear history (rebase/squash merges only)
    - Block force-pushes to main

  **Must NOT do**:
  - Do NOT require 2 reviewers before 2nd maintainer exists
  - Do NOT make Trusted Publishing checks "required" (they only run on tag pushes)
  - Do NOT disable signed commits (CONTRIBUTING.md already requires them)

  **Recommended Agent Profile**: `quick`. Skills: `[]`.

  **Parallelization**: Wave 2. Blocked by: T3.

  **References**:
  - probot/dco: https://github.com/probot/dco
  - GitHub branch protection: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches

  **WHY**: DCO is the lightweight alternative to CLA — no per-contributor paperwork. Branch protections enforce review quality.

  **Acceptance Criteria**:
  - [ ] DCO app installed; checks run on a test PR
  - [ ] `gh api repos/<new-org>/lorm/branches/main/protection` returns required-checks config
  - [ ] CONTRIBUTING.md "Sign your commits" updated for DCO

  **QA Scenarios**:
  ```
  Scenario: DCO check enforced
    Tool: Bash (gh)
    Steps:
      1. Open a test PR without -s flag
      2. Inspect: gh pr checks <id>
    Expected Result: DCO check fails
    Evidence: .sisyphus/evidence/governance-task-4-dco.txt

  Scenario: Branch protections active
    Tool: Bash (gh)
    Steps:
      1. gh api repos/<new-org>/lorm/branches/main/protection
    Expected Result: JSON with required_status_checks containing format/check/feature-matrix
    Evidence: .sisyphus/evidence/governance-task-4-protection.txt
  ```

  **Commit**: YES: `docs(contributing): add DCO sign-off requirement`

- [x] 5. **Atomic URL migration PR (Cargo.toml + README + CONTRIBUTING + .github/)**

  **What to do**:
  - In a single PR titled `chore: migrate to lorm-rs org`:
    - Update workspace `Cargo.toml` `[workspace.package].repository = "https://github.com/<new-org>/lorm.git"`
    - Update `lorm/Cargo.toml` and `lorm-macros/Cargo.toml` per-crate `repository` fields (if overridden)
    - Update README badges (CI, docs.rs, crates.io) — replace `remysaissy/lorm` with `<new-org>/lorm`
    - Update CONTRIBUTING.md any hardcoded references
    - Update `cliff.toml` if it references repo URL
    - Update `.github/ISSUE_TEMPLATE/config.yml` if it has external links
    - Update example/test/migration files if they reference the old URL
    - Grep for `remysaissy/lorm` after changes — count must be ≤ 1 (just an attribution in LICENSE if needed)
  - CI must pass before merge.

  **Must NOT do**:
  - Do NOT split this across multiple PRs (atomicity matters — half-updated state confuses CI and crates.io)
  - Do NOT change the crate names (only the repository URL)
  - Do NOT change LICENSE (Apache-2.0 stays)

  **Recommended Agent Profile**: `quick`. Skills: `[]`.

  **Parallelization**: Wave 3 with T6. Blocks T6. Blocked by: T3.

  **References**:
  - `Cargo.toml` workspace section
  - `README.md` badges block
  - `cliff.toml` if URL referenced

  **WHY**: Partial URL updates create flaky CI + crates.io metadata mismatches at publish time.

  **Acceptance Criteria**:
  - [ ] `grep -rn "remysaissy/lorm" . --include="*.toml" --include="*.md" --include="*.yml"` returns ≤ 1 result (LICENSE attribution only)
  - [ ] CI green on the migration PR
  - [ ] After merge, `cargo metadata` reports correct repository URL

  **QA Scenarios**:
  ```
  Scenario: No remaining old URL references
    Tool: Bash (grep)
    Steps:
      1. grep -rn "remysaissy/lorm" . --include="*.toml" --include="*.md" --include="*.yml" | wc -l
    Expected Result: ≤ 1
    Evidence: .sisyphus/evidence/governance-task-5-grep.txt

  Scenario: Cargo metadata reflects change
    Tool: Bash
    Steps:
      1. cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].repository'
    Expected Result: matches new URL
    Evidence: .sisyphus/evidence/governance-task-5-metadata.txt
  ```

  **Commit**: YES: `chore: migrate to <new-org> org`

- [x] 6. **Trusted Publishing re-wire + dry-run verification (UNBLOCKS lorm-1.0-stability T13)**

  **What to do**:
  - On crates.io, update Trusted Publishing config for both `lorm` and `lorm-macros`:
    - Repository: `<new-org>/lorm`
    - Workflow: `release.yml`
    - Environment: same as before (if any)
  - Inspect `.github/workflows/release.yml`: verify `id-token: write` permission, verify it references the crates correctly. No code change usually needed — the auth is bound to the GitHub repo, not the URL.
  - **Dry-run verification**: tag a pre-release tag `v0.4.1-test` on a non-main branch, observe release.yml run, ensure attestation + publish steps succeed up to (but stopping before) the actual `cargo publish` call. Use `--dry-run` flag if release.yml supports it, otherwise modify temporarily and revert.
  - After successful dry-run, update `.sisyphus/drafts/trusted-publishing-status.md` confirming readiness.
  - **Cross-plan signal**: mark `lorm-1.0-stability.md` T13 as unblocked.

  **Must NOT do**:
  - Do NOT actually publish a test version to crates.io (use `--dry-run`)
  - Do NOT skip dry-run "because it should just work"
  - Do NOT modify release.yml beyond what's needed (it's well-tested today)

  **Recommended Agent Profile**: `unspecified-high`. Skills: `[]`.

  **Parallelization**: Wave 3 with T5. Blocks `lorm-1.0-stability.md` T13. Blocked by: T3, T5.

  **References**:
  - crates.io Trusted Publishing docs: https://crates.io/docs/trusted-publishing
  - `.github/workflows/release.yml` lines 1-128

  **WHY**: A broken Trusted Publishing pipeline is the #1 risk to the 1.0 launch. Verify in non-production conditions.

  **Acceptance Criteria**:
  - [ ] Trusted Publishing config updated on crates.io for both crates
  - [ ] Test tag dry-run succeeds (attestation + dry-run publish steps green)
  - [ ] `.sisyphus/drafts/trusted-publishing-status.md` records evidence
  - [ ] Signal sent: 1.0 plan T13 is unblocked

  **QA Scenarios**:
  ```
  Scenario: Dry-run release workflow succeeds
    Tool: Bash (gh)
    Steps:
      1. git tag v0.4.1-test
      2. git push origin v0.4.1-test
      3. gh run watch
      4. Inspect: attestation step + cargo publish --dry-run step both green
      5. Cleanup: git push --delete origin v0.4.1-test; git tag -d v0.4.1-test
    Expected Result: all jobs green up to and including dry-run cargo publish
    Evidence: .sisyphus/evidence/governance-task-6-dry-run.txt

  Scenario: Trusted Publishing config matches new URL
    Tool: Manual (cargo owner)
    Steps:
      1. cargo owner --list lorm — should show the GitHub Actions workflow + repo
      2. Same for lorm-macros
    Expected Result: both reflect new <new-org>/lorm
    Evidence: .sisyphus/evidence/governance-task-6-cratesio.txt
  ```

  **Commit**: YES (if any workflow changes were needed): `ci(release): align Trusted Publishing for new org URL`

- [x] 7. **Set up GitHub Sponsors / Open Collective**

  **What to do**:
  - **OPTIONAL TASK** — proceed only if maintainer(s) opt in.
  - Apply for GitHub Sponsors at the org level (requires GitHub eligibility verification).
  - Configure tiers: $5/mo (supporter), $25/mo (sponsor), $100/mo (corporate).
  - Update `.github/FUNDING.yml` with active sponsor links.
  - Add a "Sponsors" badge to README.
  - Document in MAINTAINERS.md (T10): "Donations are used for X, Y, Z (e.g., domain, CI minutes, conference travel)."

  **Must NOT do**:
  - Do NOT promise specific deliverables in exchange for sponsorship
  - Do NOT name maintainers as sponsorship recipients without their consent
  - Do NOT skip the GitHub eligibility verification

  **Recommended Agent Profile**: `quick`. Skills: `[]`.

  **Parallelization**: Wave 4. Blocked by: T3.

  **References**:
  - GitHub Sponsors for organizations: https://docs.github.com/en/sponsors

  **WHY**: A sustainable funding signal addresses the "is this project here for the long haul?" question. Optional — many successful crates skip it.

  **Acceptance Criteria**:
  - [ ] FUNDING.yml updated (or explicitly empty with comment if maintainer declines)
  - [ ] Sponsors button enabled in repo settings (or decision documented)

  **QA Scenarios**:
  ```
  Scenario: Sponsors enabled or explicit decline
    Tool: Bash (gh)
    Steps:
      1. cat .github/FUNDING.yml
    Expected Result: either has sponsor links or has a comment "intentionally empty"
    Evidence: .sisyphus/evidence/governance-task-7-funding.txt
  ```

  **Commit**: YES (if applicable): `chore: enable GitHub Sponsors`

- [x] 8. **Identify + recruit 2nd maintainer (manual outreach)**

  **What to do**:
  - Identify candidates (criteria: prior PR contributors, active Rust+sqlx community members, interest in lightweight ORMs):
    - Check git log: `git shortlog -sne | head -10` for prior contributors
    - Check `gh issue list --search "comment:>10"` for engaged users
    - Check Reddit r/rust / users.rust-lang.org search for `lorm` mentions
  - Reach out via GitHub DM or email with a clear offer:
    - Role: co-maintainer with merge rights
    - Time commitment expectation (e.g., "1-2 hours/week, no minimum")
    - Crates.io co-owner status
    - Recognition in MAINTAINERS.md
  - Set a 30-day timeout. If no acceptance after first wave, expand to 2nd-wave candidates.
  - Document in `.sisyphus/drafts/maintainer-recruitment.md`: who was contacted, when, response status.

  **Must NOT do**:
  - Do NOT publicly announce names of declined candidates
  - Do NOT promise compensation (this is volunteer)
  - Do NOT skip a written consent (require them to merge an "I accept" PR to MAINTAINERS.md)

  **Recommended Agent Profile**: `unspecified-high` (human outreach component cannot be fully automated). Skills: `[]`.

  **Parallelization**: Wave 4. Blocks T9, T10. Blocked by: T3.

  **References**:
  - tokio-rs/tokio MAINTAINERS history as a model
  - SQLx maintainer onboarding via GitHub announcements

  **WHY**: Bus factor = 1 is the single biggest production-adoption blocker per the feedback.

  **Acceptance Criteria**:
  - [ ] At least 1 candidate has accepted and merged the "I accept" PR
  - [ ] If 30 days elapse without acceptance: a documented "still looking" status note in `.sisyphus/drafts/maintainer-recruitment.md` plus continued outreach

  **QA Scenarios**:
  ```
  Scenario: 2nd maintainer onboarded
    Tool: Bash (git)
    Steps:
      1. git log MAINTAINERS.md --format="%an"
    Expected Result: ≥ 2 distinct authors editing MAINTAINERS.md
    Evidence: .sisyphus/evidence/governance-task-8-onboarded.txt
  ```

  **Commit**: NO (recruitment is async; documentation in drafts)

- [x] 9. **Add 2nd maintainer as crates.io co-owner on both crates**

  **What to do**:
  - After T8 onboards a maintainer, run:
    - `cargo owner --add <github-handle> lorm`
    - `cargo owner --add <github-handle> lorm-macros`
  - Verify with `cargo owner --list lorm` showing both maintainers.

  **Must NOT do**:
  - Do NOT remove the original owner
  - Do NOT add anyone without explicit consent

  **Recommended Agent Profile**: `quick`. Skills: `[]`.

  **Parallelization**: Wave 4. Blocks T10. Blocked by: T8.

  **References**:
  - crates.io owner mgmt: https://doc.rust-lang.org/cargo/reference/publishing.html#cargo-owner

  **WHY**: Co-ownership is the actual bus-factor protection (org alone doesn't grant crates.io publish rights).

  **Acceptance Criteria**:
  - [ ] `cargo owner --list lorm` shows ≥ 2 owners
  - [ ] `cargo owner --list lorm-macros` shows ≥ 2 owners

  **QA Scenarios**:
  ```
  Scenario: Co-ownership active
    Tool: Bash
    Steps:
      1. cargo owner --list lorm 2>&1 | wc -l
      2. cargo owner --list lorm-macros 2>&1 | wc -l
    Expected Result: both ≥ 2
    Evidence: .sisyphus/evidence/governance-task-9-coowner.txt
  ```

  **Commit**: NO (crates.io action, not a code commit)

- [x] 10. **Publish MAINTAINERS.md + response-time SLA**

  **What to do**:
  - Create `MAINTAINERS.md` listing all current maintainers:
    - Name, GitHub handle, primary contact (preferably GitHub @-mentions)
    - Time zone
    - Areas of focus (if specialized)
  - Add response-time commitments (realistic, achievable):
    - Issues: triage within 7 calendar days
    - PRs: first review within 14 calendar days
    - Security disclosures: acknowledged within 72 hours (linked from SECURITY.md)
  - Add escalation: "If a maintainer is unreachable > 2 weeks, please open a 'Maintainer unreachable' issue tagging the other maintainers."
  - Link from README ("Maintained by ..." badge + link).

  **Must NOT do**:
  - Do NOT commit to SLAs harder than 7d issues / 14d PRs (over-promising kills credibility)
  - Do NOT include phone numbers or personal email
  - Do NOT list a maintainer who hasn't accepted via T8

  **Recommended Agent Profile**: `writing`. Skills: `[]`.

  **Parallelization**: Wave 4. Blocked by: T8, T9.

  **References**:
  - `.sisyphus/drafts/maintainer-recruitment.md` for confirmed names

  **WHY**: The feedback explicitly calls this out: *"A MAINTAINERS.md with a stated response-time commitment helps."*

  **Acceptance Criteria**:
  - [ ] MAINTAINERS.md exists with ≥ 2 named maintainers
  - [ ] Response-time SLAs clearly stated
  - [ ] Linked from README

  **QA Scenarios**:
  ```
  Scenario: MAINTAINERS.md complete and linked
    Tool: Bash
    Steps:
      1. test -f MAINTAINERS.md
      2. grep -c "^- " MAINTAINERS.md  # count entries
      3. grep -c "MAINTAINERS.md" README.md  # README links to it
    Expected Result: ≥ 2 maintainers; README links present
    Evidence: .sisyphus/evidence/governance-task-10-maintainers.txt
  ```

  **Commit**: YES: `docs(governance): publish MAINTAINERS.md with response-time SLAs`

- [x] 11. **Publish community engagement policy + populate tracker**

  **What to do**:
  - Create `.github/COMMUNITY.md` (or `docs/community-engagement-policy.md`) codifying:
    - **MUST**: only engage where lorm genuinely solves the stated problem; always disclose authorship; provide substantive code-example answers (not link-drops); wait for 1.0 before broad outreach
    - **MUST NOT**: post in threads where SeaORM/Diesel/sqlx-direct is clearly the right answer; post more than once per thread; coordinate upvotes; post across multiple forums same day
    - **Engagement metrics**: aim for 3-5 substantive engagements per quarter, NOT mass volume
    - **Disclosure template**: "I'm a maintainer of lorm. For your use case (X), it might fit because Y. Alternatives Z are better for W. Code: ```..."
  - Set up `.sisyphus/drafts/community-engagement-tracker.md` to record each engagement:
    - Date, forum, thread URL, what we said, response received, outcome
    - Reviewed quarterly to assess effectiveness
  - Link engagement policy from README's "Contributing" section.

  **Must NOT do**:
  - Do NOT codify volume targets (counter to non-pushy intent)
  - Do NOT permit trash-talking other ORMs
  - Do NOT allow undisclosed shilling

  **Recommended Agent Profile**: `writing`. Skills: `[]`.

  **Parallelization**: Wave 4. Blocks `lorm-content-ecosystem.md` outreach tasks. Blocked by: T3.

  **References**:
  - Rust community moderation guide: https://www.rust-lang.org/policies/code-of-conduct
  - Examples of well-received maintainer-outreach: sqlx maintainers' historic Reddit threads (search: `site:reddit.com/r/rust sqlx site:github.com`)

  **WHY**: Without a written policy, even well-intentioned outreach drifts into spam. Codified guardrails protect the project's reputation.

  **Acceptance Criteria**:
  - [ ] Policy file exists with MUST/MUST NOT sections
  - [ ] Tracker initialized with quarterly review cadence
  - [ ] Linked from README + CONTRIBUTING.md

  **QA Scenarios**:
  ```
  Scenario: Engagement policy is published and linked
    Tool: Bash
    Steps:
      1. test -f .github/COMMUNITY.md
      2. grep -c "COMMUNITY.md" README.md
    Expected Result: file exists; README links
    Evidence: .sisyphus/evidence/governance-task-11-policy.txt

  Scenario: Tracker has the expected structure
    Tool: Read
    Steps:
      1. Read .sisyphus/drafts/community-engagement-tracker.md
      2. Verify columns: Date, Forum, URL, Content, Response, Outcome
    Expected Result: structure correct, ready for entries
    Evidence: .sisyphus/evidence/governance-task-11-tracker.txt
  ```

  **Commit**: YES: `docs(community): publish engagement policy + initial tracker`

---

## Final Verification Wave

- [x] F1. **Plan Compliance Audit** — `oracle`
  Verify every "Must Have" via `gh` commands and file inspection. Search for any reference to the OLD URL anywhere (`grep -rn "remysaissy/lorm" .` should return ≤ 1). Confirm crates.io co-ownership. Confirm MAINTAINERS.md, GOVERNANCE.md, SECURITY.md, COMMUNITY.md all exist and link from README.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [11/11] | VERDICT`

- [x] F2. **Config Quality Review** — `unspecified-high`
  Inspect branch protections, DCO bot, Trusted Publishing settings on crates.io. Validate YAML files (`.github/FUNDING.yml`, workflows). Verify no dead links in governance docs.
  Output: `Settings correct | YAML valid | Links live | VERDICT`

- [x] F3. **Manual QA via gh CLI** — `unspecified-high`
  End-to-end test: open a draft PR without DCO sign-off and confirm it fails the check. Verify `cargo owner --list` shows expected owners. Try the dry-run release workflow from T6 once more to confirm it still works. Open SECURITY.md disclosure email address (a forwarder must actually receive mail).
  Output: `Scenarios pass | Integration | Edge cases | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep`
  Verify NO accidental changes to source code (governance is doc/config only). Verify license unchanged. Verify crate names unchanged. Verify no breaking publish-side metadata changes.
  Output: `Tasks 11/11 compliant | Contamination CLEAN | VERDICT`

---

## Commit Strategy

- **W1 / W2**: T2, T3 → `docs(governance): add GOVERNANCE/SECURITY/FUNDING`
- **W2**: T4 → `docs(contributing): add DCO sign-off requirement`
- **W3**: T5 → `chore: migrate to <new-org> org`; T6 → `ci(release): align Trusted Publishing` (if needed)
- **W4**: T7 → `chore: enable GitHub Sponsors` (if opted in); T10 → `docs(governance): publish MAINTAINERS.md`; T11 → `docs(community): publish engagement policy`

---

## Success Criteria

### Verification Commands
```bash
gh repo view <new-org>/lorm --json url           # confirms migration
curl -sI https://github.com/remysaissy/lorm      # 301 redirect
cargo owner --list lorm                          # ≥ 2 owners
cargo owner --list lorm-macros                   # ≥ 2 owners
grep -rn "remysaissy/lorm" . --include="*.toml" --include="*.md" --include="*.yml" | wc -l   # ≤ 1
test -f MAINTAINERS.md && test -f GOVERNANCE.md && test -f SECURITY.md && test -f .github/COMMUNITY.md
```

### Final Checklist
- [ ] Org migrated, repo accessible at new URL with redirect
- [ ] Trusted Publishing verified for new URL (1.0 plan T13 unblocked)
- [ ] ≥ 2 maintainers (named in MAINTAINERS.md, crates.io co-owners)
- [ ] DCO enforced
- [ ] Community engagement policy published
- [ ] Engagement tracker initialized
- [ ] Funding decision documented (active or "intentionally none")
- [ ] All governance docs linked from README
