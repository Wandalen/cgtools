# Cite asset_uri_resolve's pre-existing native test coverage in docs/layer/005's d3 row

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:45:30
- **expires_at:** 2026-08-20 00:45:30
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** null
- **unit_type:** repository
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-19 22:45:30
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **unverified_at:** 2026-08-19 22:37:54
- **unverified_by:** system

## Goal

`docs/layer/005_l4_scene_model.md`'s Occupants-per-Stack table, d3 row
(line 31), says the glTF loader "is not off-GPU-validatable end-to-end,
though pure sub-surfaces are: light-extraction (`light_list_get`) is now
natively tested off-GPU (task 118)" — naming exactly one pure sub-surface.
This undersells the loader's actual off-GPU coverage: the same file
(`module/helper/renderer/src/webgl/loaders/gltf.rs`, line 450) also
exports `asset_uri_resolve`, a pure URI-resolution function with its own,
separate, pre-existing native test suite —
`module/helper/renderer/tests/gltf_loader_tests.rs`, 6 cases covering
relative-path joining, `blob:` URIs, `data:` URIs, `https://` URLs,
absolute paths, and an empty-folder-path edge case — confirmed present
by direct re-read this session. That test file's own header comment
states it was "Relocated from inline `src/webgl/loaders/gltf.rs` per the
all-tests-in-tests/ convention" with no task-number citation, meaning it
predates `light_list_get`'s task-118 coverage and was simply never named
on this page. `docs/layer/005`'s own Sources table (lines 43-51) has the
same gap one level down: it cites `src/webgl/loaders/gltf.rs` for "glTF
ingestion" but has no row at all for `tests/gltf_loader_tests.rs`. Fix by
naming both pure sub-surfaces on line 31 and adding the missing Sources
row — this is gap #5a from the 2026-08-17 docs/layer round-3 gap audit
(refined from an earlier, broader mis-framing: the original round-3
sweep first suspected this page undersold glTF-loader coverage
generically, but on fresh re-read the page already correctly cites
`light_list_get`/task 118 — the real, narrower gap is that
`asset_uri_resolve`'s separate, older coverage is the one omission).
Testable: `grep -c "asset_uri_resolve" docs/layer/005_l4_scene_model.md`
returns ≥1 (was: 0).

## In Scope

- `docs/layer/005_l4_scene_model.md` line 31 (d3 row, State column):
  extend the "pure sub-surfaces are:" clause to also name
  `asset_uri_resolve` and its `gltf_loader_tests.rs` coverage, alongside
  the existing `light_list_get`/task 118 mention.
- `docs/layer/005_l4_scene_model.md`'s Sources table (lines 43-51): add a
  row for `module/helper/renderer/tests/gltf_loader_tests.rs`.

## Out of Scope

- `light_list_get`'s existing citation — accurate, not touched.
- Any change to `module/helper/renderer/src/webgl/loaders/gltf.rs` or
  `tests/gltf_loader_tests.rs` — both already correct; this task adds a
  documentation citation only.
- `module/helper/renderer/src/webgl/animation/loaders/gltf.rs` (the
  animation-specific loader, already cited in the Sources table) — its
  own native-coverage gap is separately scoped as gap #5b (a distinct
  task, not this one); `asset_uri_resolve` lives in the *main* d3 loader,
  not the animation loader.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Non-code task: test-related items omitted.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Line 31 names both pure sub-surfaces (`light_list_get` and
    `asset_uri_resolve`) with accurate coverage citations
-   Sources table cites `tests/gltf_loader_tests.rs`
-   No file under `module/helper/renderer/src/` or
    `module/helper/renderer/tests/` modified
-   Independent verification passes per `§ Acceptance Verification :
    Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to
    `task/completed/`

## Test Matrix

*(Non-code documentation task — rows are text-consistency checks, not
`cargo test` cases.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -c "asset_uri_resolve" docs/layer/005_l4_scene_model.md` | Updated d3 row | ≥1 (was: 0) |
| T02 | `grep -c "light_list_get" docs/layer/005_l4_scene_model.md` | Existing citation preserved | ≥1 (unchanged) |
| T03 | `grep -c "gltf_loader_tests.rs" docs/layer/005_l4_scene_model.md` | New Sources row | ≥1 (was: 0) |
| T04 | `git diff --stat -- module/helper/renderer/src/ module/helper/renderer/tests/` | Out-of-scope source/test tree | Empty (untouched) |

## Acceptance Criteria

-   Line 31 names both `light_list_get` and `asset_uri_resolve` as pure,
    off-GPU-tested sub-surfaces of the d3 glTF loader
-   The Sources table cites `tests/gltf_loader_tests.rs`
-   No overclaim: the row still correctly states the loader's `load()` as
    a whole is not off-GPU-validatable end-to-end
-   `module/helper/renderer/src/` and `tests/` remain untouched
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does
NOT self-verify — an independent verifier performs the walk after the
task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Documentation consistency**
- [ ] C1 — Does line 31's d3 row name `asset_uri_resolve` as a pure,
  natively-tested sub-surface?
- [ ] C2 — Does it still name `light_list_get` (unchanged, not replaced)?
- [ ] C3 — Does the Sources table have a row for
  `module/helper/renderer/tests/gltf_loader_tests.rs`?
- [ ] C4 — Does the row's overall claim (loader `load()` as a whole is
  not off-GPU-validatable) remain intact and accurate?

**Out of Scope confirmation**
- [ ] C5 — Is `module/helper/renderer/src/` untouched (`git diff --stat
  -- module/helper/renderer/src/` empty)?
- [ ] C6 — Is `module/helper/renderer/tests/` untouched (`git diff
  --stat -- module/helper/renderer/tests/` empty)?

### Measurements

- [ ] M1 — `grep -c "asset_uri_resolve" docs/layer/005_l4_scene_model.md` → ≥1 (was: 0)
- [ ] M2 — `grep -c "gltf_loader_tests.rs" docs/layer/005_l4_scene_model.md` → ≥1 (was: 0)

### Invariants

- [ ] I1 — source/test tree unaffected: `git diff --stat --
  module/helper/renderer/src/ module/helper/renderer/tests/` → empty
- [ ] I2 — workspace still builds: `cargo check --workspace` → 0 errors
  (doc-only change, unaffected)

### Anti-faking checks

- [ ] AF1 — the new citation names the SPECIFIC test file
  (`gltf_loader_tests.rs`) and function (`asset_uri_resolve`), not a
  generic "also has some tests" restatement — checked by reading the
  literal replacement text

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single file (`docs/layer/005_l4_scene_model.md`); `unit_type: repository` retained for consistency with sibling docs/layer gap tasks since the file is not itself a crate | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-17 03:19:36 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-17 03:20 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 222` → blocked: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)`; left at 🔬 Verifying |
| 2026-08-18 23:47:41 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:12 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:54 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 222` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-17]** `FILED` — Task filed via docs/layer round-3 gap audit (gap #5a, refined scope): cite `asset_uri_resolve`'s pre-existing native test coverage in docs/layer/005's d3 row, alongside the already-cited `light_list_get`/task 118.
- **[2026-08-17]** `EXECUTED` — Verified fresh via `grep -n "asset_uri_resolve"` against both `gltf.rs` (pub fn at line 450, re-exported line 1365) and `gltf_loader_tests.rs` (6 call sites: relative-path, `blob:`, `data:`, `https://`, absolute-path, empty-folder-path — matching the task's claim exactly) before editing. Extended line 31's d3-row "pure sub-surfaces are:" clause to add `asset_uri_resolve` alongside the existing unchanged `light_list_get`/task 118 mention; added a new Sources-table row for `tests/gltf_loader_tests.rs`. Test Matrix: T01 (`grep -c "asset_uri_resolve"` → 2, want ≥1) PASS; T02 (`grep -c "light_list_get"` → 1, want ≥1 unchanged) PASS; T03 (`grep -c "gltf_loader_tests.rs"` → 2, want ≥1) PASS; T04 (`git diff --stat -- module/helper/renderer/src/ module/helper/renderer/tests/`) empty — clean, no caveat needed this time. AF1 confirmed: new text names the specific function (`asset_uri_resolve`) and file (`gltf_loader_tests.rs`) with the 6 concrete case categories, not a generic restatement. C4 (loader `load()` as a whole still correctly stated as not off-GPU-validatable end-to-end) confirmed unchanged — only the "pure sub-surfaces are:" clause was extended. Self-check performed as Tier 2 Dual-Role Self-Check (this repo's MAAV cap).

## Related Documentation

- `module/helper/renderer/tests/gltf_loader_tests.rs` — the pre-existing
  native test suite this task cites
- `task/accepting/118_renderer_gltf_light_extension_parsing_test.md` —
  the task behind the existing, unchanged `light_list_get` citation
