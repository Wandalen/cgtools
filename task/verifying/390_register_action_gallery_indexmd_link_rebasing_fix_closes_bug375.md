# 390: Register action gallery index.md link rebasing fix closes BUG-375

## Execution State

- **id:** 390
- **title:** Register action gallery index.md link rebasing fix closes BUG-375
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 21:03:30
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** dir
- **unit:** lib/yrd_gamedev/cgtools/action
- **closes:** BUG-375
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 21:04:18
- **expires_at:** 2026-08-18 23:04:18
- **unverified_at:** 2026-08-18 21:04:18
- **unverified_by:** unknown
- **verifying_at:** 2026-08-18 21:04:18
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-375 (`task/bug/verified/375_gallery_index_md_links_not_rebased.md`, Low severity, 🎯
Verified) found `action/gallery`'s md-row builder copied each example readme's extracted
description paragraph into `examples/index.md` verbatim, only pipe-escaping it — any relative
markdown link inside that description was written for the readme's own directory
(`examples/<category>/<example>/readme.md`) but landed, unrebased, in `index.md` two directory
levels higher, dangling. The committed file carried 3 such occurrences. The fix — two new
helpers, `_normalize_path()` (lexical `.`/`..`/empty-segment resolution) and `_rebase_links()`
(rewrites every inline-link target against the example's `examples/`-relative directory, passing
absolute/anchor/root-relative/mailto targets through untouched) — is already applied at
`action/gallery:158-215`, wired into the md-row builder at `:288` before pipe-escaping, with
`examples/index.md` already regenerated to remove all 3 dangling occurrences. Independently
confirmed via a new second test block in `action/tests/gallery_test.sh`. This task performs the
remaining lifecycle bookkeeping — `tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to
Task` (PROC12) — to formally register that already-complete, already-verified fix as a tracked
task, closing BUG-375. Testable (live-confirmed this task's own filing, 2026-08-18, `longrun` pid
785792, exit 0): `bash action/tests/gallery_test.sh` → `PASS: _html_escape produces correct HTML
entities`, `PASS: _rebase_links rebases relative targets and preserves absolute ones`, exit 0.

## In Scope

- `action/gallery` — the already-applied `_normalize_path()`/`_rebase_links()` helpers
  (`:158-215`), the md-row builder's wiring of `_rebase_links` before pipe-escaping (`:288`),
  and the extended header comment documenting the rebasing contract (`:3-11`) — verify present
  via direct read; no further edit expected.
- `examples/index.md` — the already-regenerated file (generated artifact, not hand-edited) with
  all 3 pre-fix dangling occurrences replaced by rebased, existing targets — verify via direct
  grep; no further edit expected.
- `action/tests/gallery_test.sh` — the already-applied second test block extracting
  `_normalize_path()`/`_rebase_links()` verbatim from the live script — verify present via
  direct read; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/375_gallery_index_md_links_not_rebased.md`'s header back to this
  task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `action/gallery` or `action/tests/gallery_test.sh` — the fix is
  complete and verified by the bug's own Verification Record (8/8 PASS, 2026-08-18).
- Re-running or amending BUG-375's own Verification Record — already run and recorded in the bug
  file; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- Auditing other generators in the repo for the same relocated-prose-without-rebasing defect
  class — the bug file's own Generalized Version section already searched and confirmed
  `action/gallery`'s md-row builder is the single confirmed emission site in this repo (the HTML
  builder strips links instead, checked separately); not re-run by this registration task.
- `examples/index.html` — confirmed by the bug file's own H3/E4 to be unaffected (`_strip_markdown`
  removes link syntax before HTML output reaches it); not touched by this fix or this task.
- Diagnosing or fixing the external `mdmath_core`/`ndarray_cg` workspace build blocker
  documented on tasks 385-388 — not applicable here at all: `action/gallery` and its test are
  plain bash, no Cargo build involved.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own Symptom/MRE sections directly
  captured the pre-fix corpus evidence (`grep -c` on the committed `examples/index.md` showing 2
  + 1 dangling occurrences) and the pre-fix test extraction failure (`_rebase_links()` did not
  exist).
- Fix already applied across 3 sites (`action/gallery`'s 2 new helper functions + 1 wiring call,
  `examples/index.md` regenerated), matching this repo's bash-script fix-documentation
  convention (header comment documents the contract; no per-line `Fix(BUG-NNN)` marker convention
  applies to this non-Rust component, consistent with the bug file's own `## Refs: src/` section).
- Green state already confirmed by the bug file's own Verification Record (2026-08-18) AND
  independently re-confirmed live by this task's own filing (`longrun` pid 785792, exit 0,
  2026-08-18 21:03:03): `bash action/tests/gallery_test.sh` → both `PASS:` lines, exit 0.
- No refactor needed — the fix adds 2 helper functions and 1 wiring call, no structural churn to
  the rest of the gallery script.
- Fix documentation already complete at the bug level: BUG-375 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention/Pitfall/Generalized Version narrative — this task does not
  duplicate it, only cross-links via `closes: BUG-375`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|--------------------|
| T01 | `bash action/tests/gallery_test.sh` | full test script | both `PASS:` lines, exit 0 (live-confirmed this filing) |
| T02 | `_rebase_links "[Hello Triangle](../hello_triangle/readme.md)" "minwebgpu/hello_triangle_quickstart"` | rebase a relative target | rewrites to `[Hello Triangle](./minwebgpu/hello_triangle/readme.md)` |
| T03 | `grep -cF '[Hello Triangle](./minwebgpu/hello_triangle/readme.md)' examples/index.md` | regenerated index.md | 2 (matches the bug's own claimed ×2 dangling-link count, now rebased) |
| T04 | `grep -cF '](../hello_triangle/readme.md)' examples/index.md` | regenerated index.md | 0 (pre-fix dangling form gone) |

## Acceptance Criteria

- `action/gallery` defines `_normalize_path()` and `_rebase_links()`, and the md-row builder
  calls `_rebase_links` before pipe-escaping
- `examples/index.md` carries zero occurrences of the 2 pre-fix dangling link forms
- `action/tests/gallery_test.sh` contains the `_rebase_links`/`_normalize_path` test block and
  passes
- `task/bug/verified/375_gallery_index_md_links_not_rebased.md`'s header states `**Fix Task:**`
  pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row's claim holds against a live run performed during this task's own filing

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `action/gallery` define both `_normalize_path()` and `_rebase_links()`?
- [ ] C2 — Does the md-row builder call `_rebase_links` before the pipe-escape substitution?
- [ ] C3 — Does `examples/index.md` carry zero occurrences of `](../hello_triangle/readme.md)`
  or `](../../../docs/pattern/005_script_as_glue.md)`?
- [ ] C4 — Does `bash action/tests/gallery_test.sh` (via `longrun`) print both `PASS:` lines and
  exit 0?

**Registration correctness**
- [ ] C5 — Does this task's `closes:` field name `BUG-375`?
- [ ] C6 — Does BUG-375's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C7 — No Edit/Write tool call in this task's own execution targeted `action/gallery`,
  `action/tests/gallery_test.sh`, or `examples/index.md`/`examples/index.html` — note this
  repo's working tree carries many pre-existing, unrelated uncommitted changes from other
  concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal here.

### Measurements

- [ ] M1 — `grep -n "_normalize_path()\|_rebase_links()"` in `action/gallery` → 2 function
  definitions (lines 158, 187)
- [ ] M2 — `grep -cF '[Hello Triangle](./minwebgpu/hello_triangle/readme.md)'` in
  `examples/index.md` → 2

### Invariants

- [ ] I1 — `bash action/tests/gallery_test.sh` → exit 0, both `PASS:` lines present

### Anti-faking checks

- [ ] AF1 — the test script's second block extracts `_normalize_path`/`_rebase_links` verbatim
  from the live `action/gallery` source (not a reimplementation or hardcoded stand-in) and
  asserts against real function output — checked by reading the test body itself, not just its
  pass/fail result

## Verification Record

**Gate Round 1** · Tier: 2 · Type: Full · Verdict: OPEN · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | MOST Goal Compliance | — | 🟢 | Confirming: goal states BUG-375, cites PROC12, gives a live-confirmed Testable line. Adversarial: tried to find an overstated claim (e.g. a PASS not actually observed this session) — the Testable line cites the exact `longrun` pid/exit/timestamp from this task's own filing, matches the live log verbatim. | — |
| D2 | Deliverable Verification Completeness | — | 🟢 | Confirming: Verification section carries C1-C7/M1-M2/I1/AF1, each independently checkable. Adversarial: obtained a full, genuine live re-verification — not applicable to the external `mdmath_core`/`ndarray_cg` blocker at all, since `action/gallery` and its test are plain bash with zero Cargo involvement. `longrun .launch` (pid 785792, exit 0, `-0110_longrun.log`) of `bash action/tests/gallery_test.sh` printed both `PASS:` lines verbatim, matching T01/I1/C4 exactly. C1 verified via direct `grep -n` (`_normalize_path()`:158, `_rebase_links()`:187, unchanged from the bug file's own claimed line numbers). C2 verified via direct read of `action/gallery:288-290` — `_rebase_links` call precedes the pipe-escape substitution. C3 verified via `grep -cF` for both exact pre-fix dangling strings (`](../hello_triangle/readme.md)`, `](../../../docs/pattern/005_script_as_glue.md)`) → 0 each. M2 required disambiguating: an initial broad `grep -c './minwebgpu/hello_triangle/readme.md'` returned 4 (conflating the rebased description-link with the unrelated `hello_triangle` example's own primary row self-link, which also happens to resolve to the same path) — resolved by narrowing to the exact link-text-qualified fixed string `[Hello Triangle](./minwebgpu/hello_triangle/readme.md)`, which returned exactly 2, matching the bug's own claimed ×2 dangling-link count precisely. This self-correction is recorded here rather than silently discarded, per the pre-verify-before-write discipline. | Corrected an overly broad grep pattern before it became a wrong claim in this task file — no code fix needed, verification-methodology fix only. |
| D3 | Anti-Cheating Readiness | — | 🟢 | Confirming: AF1 requires reading the actual test body, not trusting pass/fail alone. Adversarial: read `action/tests/gallery_test.sh:41-102` directly this session — the second test block extracts `_normalize_path`/`_rebase_links` verbatim via a function-extraction technique (matching the first block's pattern for `_html_escape`, already used to pin BUG-315), then calls the real extracted functions and asserts on real output — no hardcoded expected-value literal standing in for a function call found. | — |
| D4 | Execution Prerequisites | — | 🟢 | Confirming: `unit_type: dir`, `unit: lib/yrd_gamedev/cgtools/action`, `closes: BUG-375` all set correctly in Execution State — `dir` chosen deliberately over `module` since `action/` is a plain bash tooling directory, not a Cargo crate. Adversarial: checked whether `unit` should instead be the repo root (since `examples/index.md`, outside `action/`, is also touched) — no: `examples/index.md` is a generated artifact of `action/gallery`, not a hand-maintained file with its own responsibility boundary; scoping to `action/` (the actual fix's component, per the bug file's own `**Component:**` field) is correct and consistent with tasks 385-389's pattern of scoping to the crate that owns the fix. | — |
| D5 | Source-of-Truth Alignment | — | 🟢 | Confirming: no `docs/feature`/`docs/invariant`/`docs/api` instance exists for `action/gallery`'s link-emission contract to conflict with. Adversarial: searched for a doc instance that might still describe verbatim-copy as intended behavior — none found; no BLOCKING spec.md/spec/ hygiene violation applies either (dev repo, no spec.md present). | — |
| D6 | Decomposition Fit | — | 🟢 | Confirming: fix spans exactly 1 component (`action/gallery` + its test + the generated `examples/index.md`), no multi-crate split warranted — this bug predates the Cargo-crate framing entirely. Adversarial: checked whether the bug's own Generalized Version implies other generators in the repo need the same fix — no: its own dedup/emission-site search already confirmed `action/gallery`'s md-row builder is the single confirmed emission site (the HTML builder strips links instead); nothing left to decompose into further tasks. | — |
| D7 | Rulebook Compliance | — | 🟢 | Confirming: no `cargo fmt` invoked, no git command run, Edit used exclusively (task file was `tsk .create`-generated then Edited, never Written after initial creation), all temp artifacts (`-0110_longrun.log`) hyphen-prefixed. Adversarial: scanned this task's own tool-call history for a Write call against a pre-existing file, a non-whitelist git invocation, or a non-hyphenated temp file — none found. | — |
| D8 | Traceability | — | 🟢 | Confirming: `closes: BUG-375` set; bug file backlink to be added immediately after this gate (PROC12 Step 4). Adversarial: verified BUG-375's own file does NOT yet carry a `Fix Task` line (checked via prior Read before this edit) — confirming the backlink write is not a duplicate. | — |
| **Total** | | — | 🟢 | 0 open — full live re-verification obtained, no external blocker applicable to this component | 0/0 |

Dual-Role Self-Check per `maav.rulebook.md § MAAV : Verification Tier Selection` — Tier 2 default, this session capped at Tier 2 per standing project convention (never escalate).

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 21:03:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 21:04:18 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 21:04:18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 21:07:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 390 "user1@w002/.../cgtools/"` → exit 1: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)`. Same-actor guard, documented sandbox constraint — not forced/spoofed. |

## History

- **FILED** (2026-08-18 21:03:30): Task created via `tsk .create`, registering the already-complete BUG-375 fix per PROC12.
- **READINESS_GATE_PASS** (2026-08-18 21:07:00): Tier 2 Dual-Role Self-Check, Gate Round 1, 8/8 dimensions PASS, 0 issues (see Verification Record above). D2 records a self-caught grep-precision correction (an overly broad pattern conflating two unrelated link occurrences) before it became a wrong claim, per the pre-verify-before-write discipline. Not applicable to the external `mdmath_core`/`ndarray_cg` blocker at all — `action/gallery` and its test are plain bash, zero Cargo build involved — so a full genuine live test re-run was obtained (pid 785792, exit 0).
- **EXECUTED** (2026-08-18 21:07:00): `tsk .verify_pass` attempted and blocked by same-actor guard, per standard project convention for this sandbox — documented above, not circumvented.
