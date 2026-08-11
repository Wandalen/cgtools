# Fix module/blank/cgtools readme's copy-paste identity error

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/blank/cgtools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`module/blank/cgtools/readme.md` was found during the audit to contain a copy-paste identity error —
text describing a different crate's identity/purpose rather than this one's own (P5 — remaining doc
drift, Fix-in-place). Note this crate's own name collides with the workspace's own top-level name
(`cgtools`), which is plausibly exactly how the copy-paste error happened — worth checking whether
`module/blank/cg_tools` (the similarly-named sibling) is the crate whose text got pasted here by mistake.
**Exact wrong text was not preserved precisely through this session's context compaction — re-read the
file fresh at pickup to confirm before rewriting.**

## Verification

### Checklist

- [x] C1 — Does `cargo check -p cgtools` still succeed under the corrected identity, re-deriving the
  task's own original evidence against the current repo state? `cargo check -p cgtools` (via
  `longrun .launch`, this session) → exit 0.
- [x] C2 — Does `readme.md`'s H1 announce this crate's own correct identity (`# cgtools`), not the
  sibling's (`# cg_tools`)? Read `module/blank/cgtools/readme.md` line 1 → `# cgtools`.
- [x] C3 — Is the reservation-crate description sentence present and intact (not just the heading
  fixed)? Read confirms lines 5-6: "Reserved crate holding the toolkit's aggregate `cgtools` name. No
  implementation yet — the toolkit itself ships as the workspace's `module/` crates."
- [x] C4 — Does `Cargo.toml`'s `repository` field point at this crate's own repo (`Wandalen/cgtools`),
  not the sibling's (`Wandalen/cg_tools`)? Read confirms line 8:
  `repository = "https://github.com/Wandalen/cgtools"`.
- [x] C5 — Are there zero remaining `cg_tools` references anywhere inside `module/blank/cgtools/`?
  `grep -rn cg_tools module/blank/cgtools/` → exit 1, zero matches.
- [x] C6 — Does the crate still genuinely diverge from its sibling
  `module/blank/cg_tools/readme.md` (i.e. the fix isn't itself a fresh re-paste of something else)?
  `diff module/blank/cgtools/readme.md module/blank/cg_tools/readme.md` → differs on the H1
  (`cgtools` vs `cg_tools`) and the added reservation paragraph; the sibling still independently
  reads `# cg_tools`.

### Measurements

- [x] M1 — `readme.md` H1 heading: `# cgtools` (was: `# cg_tools` — confirmed via
  `git show dc8c8c1f:module/blank/cgtools/readme.md`, the repo's initial commit, matching the
  byte-identical-sibling-paste account in this task's own History).
- [x] M2 — `Cargo.toml` `repository` field: `https://github.com/Wandalen/cgtools` (was:
  `https://github.com/Wandalen/cg_tools` — confirmed via
  `git show dc8c8c1f:module/blank/cgtools/Cargo.toml`).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p cgtools --all-features` → exit 4,
  "error: no tests to run" — NOT exit 0. This is genuinely the crate's expected state, not a
  regression: `tests/basic_test.rs` is a 1-line doc-comment-only file with zero `#[test]` functions,
  and `src/lib.rs`'s `mod_interface!{}` block is empty — confirmed by reading both files directly
  this session. There is no test surface for nextest to collect.
- [x] I2 — Compiler/lints clean: `cargo clippy -p cgtools --all-targets --all-features -- -D warnings`
  → exit 0, zero warnings.

### Anti-faking checks

- [x] AF1 — Guards against the exact bug this task fixed recurring (a future editor re-pasting the
  sibling's identity wholesale): re-running C6's `diff` against `module/blank/cg_tools/readme.md`
  catches a byte-identical recurrence; C5's `grep -rn cg_tools` catches a partial re-paste limited to
  just the heading or just the repository field.
- [x] AF2 — Guards against a shortcut that restores the correct H1 text but drops the reservation
  semantics (making an empty crate look like a real implementation): C3 checks the full "no
  implementation yet" sentence, not merely that the heading string changed.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P5 (doc drift)
  tier, Fix-in-place bucket. Flagged: citation detail needs re-derivation at pickup.
- **[2026-08-10]** `IMPLEMENTED` — Re-derived: `readme.md` was byte-identical to sibling
  `module/blank/cg_tools/readme.md`, announcing `# cg_tools` while the crate's own Cargo.toml name is
  `cgtools` — the Goal's collision hypothesis confirmed exactly. Fixed H1 → `# cgtools` and added one
  honest reservation line ("Reserved crate holding the toolkit's aggregate `cgtools` name. No
  implementation yet…" — matches the crate's actual contents: empty `mod_interface!`, 1-line test file,
  and the reservation pattern the newer blank/ siblings d3_scene/frame_graph use). Body description
  ("Computer Graphics Toolkit.") kept — it matches this crate's own Cargo.toml `description` and is the
  identity being reserved. **Same-paste second hit:** this crate's `Cargo.toml` also carried
  `repository = "https://github.com/Wandalen/cg_tools"` — the sibling's URL; workspace census shows 22
  crates on `Wandalen/cgtools` (incl. every newer one) vs 6 on `Wandalen/cg_tools` (exactly the old
  blank/ crates). Fixed this crate's URL to `Wandalen/cgtools`. **Noted, not fixed (out of this task's
  unit):** the 5 remaining old blank/ crates (`cg_tools`, `mdmath`, `mdmath_ai`, `mdmath_cg`,
  `mdmath_linalg`) still point at `Wandalen/cg_tools`; for `cg_tools` itself that may even be intentional
  (own-name reservation repo) — ownership call surfaced to user rather than swept silently.
  Verification: `cargo check -p cgtools` exit 0 (`-0001_longrun.log` in crate dir); `grep -rn cg_tools
  module/blank/cgtools/` → zero hits; `locales.md` row for this crate already lists the correct
  name/description (generated file, untouched).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — 15-dimension Tier 2 gate passed (see Verification
  Record). One finding resolved in-loop: the Goal named only `readme.md`, but the same paste
  contaminated this crate's `Cargo.toml` repository URL — caught by diffing both identity files against
  the sibling and running a workspace URL census. Moved draft/ → completed/.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟡 | 🟢 | Goal named `readme.md` only; adversarial comparison of ALL identity-bearing files in the crate against the sibling found the pasted repository URL in `Cargo.toml` too | Fixed URL in the same pass; 5 out-of-unit siblings noted for user, not swept |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Goal's re-read-fresh mandate honored; its cg_tools-paste hypothesis confirmed byte-for-byte | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Considered rewriting to the richer d3_scene-style reserved-slot readme; rejected — one reservation line suffices, no docs/layer slot exists for this crate | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Edits confined to `module/blank/cgtools/` + task file | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Readme now states the crate's single responsibility explicitly: name reservation | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | No new files/dirs; Edit-not-Write on both existing files | — |
| B2 | Test-First | 🟢 | 🟢 | Wrong text confirmed against sibling + Cargo.toml BEFORE rewriting | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | Byte-identical readme diff + name field mismatch + 22-vs-6 URL census | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Own identity written in place; no compatibility residue | — |
| B5 | Fix Verification | 🟢 | 🟢 | `cargo check -p cgtools` exit 0; zero `cg_tools` grep hits in crate | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | 5 remaining old-URL blank crates + the cg_tools-may-be-intentional caveat recorded here for user decision | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Doc/manifest lines only; longrun log hyphen-prefixed | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved | 1/1 |
