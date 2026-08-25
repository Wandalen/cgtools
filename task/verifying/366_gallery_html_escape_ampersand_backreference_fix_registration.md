# Register action/gallery html-escape ampersand-backreference fix (closes BUG-315)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-20 09:57:36
- **expires_at:** 2026-08-20 11:57:36
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** BUG-315
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-20 09:57:36
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **unverified_at:** 2026-08-20 09:57:12
- **unverified_by:** system

## Goal

BUG-315 (`task/bug/verified/315_gallery_html_escape_ampersand_backreference.md`, Medium
severity, 🎯 Verified) found `action/gallery`'s `_html_escape()` function leaving `<`, `>`,
and `"` completely unescaped in the generated `examples/index.html` gallery page and
appending garbage entity-name text immediately after each, because bash's
`${parameter//pattern/replacement}` construct treats an unescaped `&` in `replacement` as a
sed-style whole-match backreference, not a literal character — so 3 of the 4 entity
substitutions (`&lt;`, `&gt;`, `&quot;`) inserted the original matched character followed by
literal entity-name text instead of the intended entity (the 4th, `&`→`&amp;`, happened to
look correct only by coincidence). This had already corrupted the committed, publicly-served
`examples/index.html` (4 occurrences of the broken `"quot;Hello Triangle"quot;` pattern,
confirmed via direct execution of the unmodified function pre-fix) and would have baked
further corruption into 3 more examples' description text (`filter`, `minimize_wasm`,
`jewelry_site`) had it gone uncaught. The fix — escaping `&` as `\&` in `_html_escape`'s 4
replacement strings (`action/gallery:95-98`, with a `Fix(BUG-315)`/`Root cause`/`Pitfall`
source comment at `:85-93`) plus a new `action/tests/gallery_test.sh` reproducer that
extracts and exercises the real function — is already applied and was independently
reconfirmed live during this task's own filing (not merely cited from the bug file):
`bash action/tests/gallery_test.sh` exits 0 (`PASS: _html_escape produces correct HTML
entities`), and `examples/index.html` now contains the correctly-escaped
`&quot;Hello Triangle&quot;` (4 occurrences) with zero occurrences of any pre-fix
`<lt;`/`>gt;`/`"quot;` broken-entity pattern anywhere in the file. This task performs the
remaining lifecycle bookkeeping — `tsk.rulebook.md § Core Procedures : Procedure - Promote
Bug to Task` (PROC12) — to formally register that already-complete, already-verified fix as
a tracked task, closing BUG-315.
Testable: `bash action/tests/gallery_test.sh` → `PASS: _html_escape produces correct HTML
entities`, exit 0.

## In Scope

- `action/gallery` (`_html_escape`, `:83-100`) — the already-applied `&`→`\&` escaping fix
  in the 4 replacement strings (`:95-98`) and its `Fix(BUG-315)`/`Root cause`/`Pitfall`
  source comment (`:85-93`); verify both are present, no further edit expected
- `action/tests/gallery_test.sh` (new) — the already-created reproducer that extracts
  `_html_escape()` verbatim from `action/gallery` at run time and asserts correct entity
  output; verify it exists and passes
- `examples/index.html` — already-regenerated generated artifact; verify it carries the
  corrected `&quot;Hello Triangle&quot;` escaping with zero occurrences of any pre-fix
  `<lt;`/`>gt;`/`"quot;` corruption pattern anywhere in the file
- `action/readme.md` and `action/tests/readme.md` — already carry Responsibility Table rows
  for the new `tests/` directory and `gallery_test.sh` respectively; verify presence, no
  further edit expected
- Formal task registration and lifecycle walk (claim-verify, attempt `tsk .verify_pass`) for
  the already-complete fix
- Linking `task/bug/verified/315_gallery_html_escape_ampersand_backreference.md`'s header
  back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed)

## Out of Scope

- Any further code change to `action/gallery`'s `_html_escape` or any other `action/*`
  script — the fix is complete and independently reconfirmed live (see `## Goal`)
- Refactoring `_html_escape` or any other part of `action/gallery` beyond the already-applied
  4-line `&`→`\&` fix
- Changing `_html_escape`'s function signature or either of its 2 call sites
  (`action/gallery:187-188`)
- Re-running BUG-315's own MRE or its own VERIFY Gate — already run and recorded in the bug
  file's own History (Tier 2 Dual-Role Self-Check, 8/8 PASS, 2026-08-18); not re-litigated by
  this task's own Readiness Verification Gate, which checks task-file quality, not the
  underlying fix
- Sweeping for any other `${var/pattern/replacement}`-with-unescaped-`&` construct outside
  `action/gallery` — BUG-315's own Generalized Version already ran this sweep workspace-wide
  and found only `action/gallery`'s own 4 lines; not re-swept here
- Resolving `action/gallery verify::1`'s currently-reported drift — independently confirmed
  during this task's own filing that `verify::1` now exits 1 ("gallery is out of date"), but
  the diff is entirely unrelated to this fix: a new `examples/gpu_hal/triangle_vulkan_window`
  example (`api:vulkan` tag, added by concurrent work after BUG-315 was verified) plus 3
  renamed `Fix(BUG-XXX)`→real-bug-number placeholder comments in unrelated examples'
  descriptions (`filter`, `minimize_wasm`, `jewelry_site`) — confirmed via direct diff
  inspection to contain zero escaping-related hunks. Re-running `action/gallery`'s write mode
  now would pull in and conflate this unrelated concurrent drift with this task's own
  deliverable — out of scope for this registration task

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Failing-first evidence already on record: BUG-315's own MRE (direct execution of the
    exact, unmodified `_html_escape` body) reproduced
    `test <lt;!-- comment "quot;works"quot; -->gt; end` against the pre-fix code (bug file
    MRE section, 2026-08-18) — this task does not re-derive that evidence
-   Fix already applied and independently reconfirmed live: `action/gallery:95-98` escapes
    `&` as `\&` in all 4 replacement strings, with the 3-field `Fix(BUG-315)`/`Root
    cause`/`Pitfall` source comment in place at `:85-93`
-   Green state already confirmed, and reconfirmed live during this task's own filing:
    `action/tests/gallery_test.sh` exits 0 (`PASS: _html_escape produces correct HTML
    entities`); `examples/index.html` carries the corrected `&quot;Hello Triangle&quot;`
    escaping (4/4 occurrences) with zero broken-pattern matches anywhere in the file
-   No refactor needed — single-character escape fix, no structural churn
-   Fix documentation already complete at the bug level: BUG-315 carries the 5-section fix
    documentation (Root Cause, Why Not Caught, Fix Location, Prevention, Pitfall) — this task
    does not duplicate it, only cross-links via `closes: BUG-315`
-   Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
    .verify_pass` then attempted per standard lifecycle (expected to hit this sandbox's known
    same-actor guard, per project convention — document rather than force/spoof if so)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `bash action/tests/gallery_test.sh` | Fixed `_html_escape`, extracted live from `action/gallery` at test-run time | exit 0; prints `PASS: _html_escape produces correct HTML entities` |
| T02 | `grep -c 'Fix(BUG-315)' action/gallery` | Fixed `_html_escape` source comment | ≥1 (comment present) |
| T03 | `grep -c '&quot;Hello Triangle&quot;' examples/index.html` | Regenerated `examples/index.html` | 4 (all 4 occurrences correctly escaped) |
| T04 | `grep -cE '<lt;\|>gt;\|"quot;' examples/index.html` | Regenerated `examples/index.html` | 0 (zero occurrences of any pre-fix broken-entity pattern anywhere in the file) |
| T05 | BUG-315's own pre-fix MRE (already run, bug file MRE section, 2026-08-18) | Unfixed `_html_escape` body (historical; not re-applied to the live, already-fixed file) | Reproduces `test <lt;!-- comment "quot;works"quot; -->gt; end` — cited as historical record, not re-executed destructively against the live file |

## Acceptance Criteria

-   `action/gallery`'s `_html_escape` (`:95-98`) escapes `&` as `\&` in all 4 replacement
    strings, not the pre-fix bare `&` form
-   The same function carries a 3-field `Fix(BUG-315)`/`Root cause`/`Pitfall` source comment
    at `:85-93`
-   `action/tests/gallery_test.sh` exists, extracts `_html_escape()` verbatim from
    `action/gallery` at run time (not a hand-copied duplicate), and exits 0
-   `examples/index.html` contains the correctly-escaped `&quot;Hello Triangle&quot;`
    pattern exactly 4 times and zero occurrences of any pre-fix broken-entity pattern
    (`<lt;`, `>gt;`, `"quot;`) anywhere in the file
-   `task/bug/verified/315_gallery_html_escape_ampersand_backreference.md`'s header states
    `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `action/gallery`'s `_html_escape` function escape `&` as `\&` in all 4
      replacement strings (`\&amp;`, `\&lt;`, `\&gt;`, `\&quot;`), not the bare pre-fix form?
- [ ] C2 — Does the same function carry a source comment with all 3 fields `Fix(BUG-315)`,
      `Root cause`, and `Pitfall`?
- [ ] C3 — Does `action/tests/gallery_test.sh` exist and extract `_html_escape()` from
      `action/gallery` via `sed` (live extraction, not a hand-copied duplicate)?
- [ ] C4 — Does `bash action/tests/gallery_test.sh` exit 0?

**Generated-artifact correctness**
- [ ] C5 — Does `examples/index.html` contain `&quot;Hello Triangle&quot;` exactly 4 times?
- [ ] C6 — Is `examples/index.html` free of every pre-fix broken-entity pattern (`<lt;`,
      `>gt;`, `"quot;`)?

**Registration correctness**
- [ ] C7 — Does this task's `closes:` field name `BUG-315`?
- [ ] C8 — Does BUG-315's own header carry a `**Fix Task:**` line pointing back at this
      task's ID?

**Out of Scope confirmation**
- [ ] C9 — Is any `action/*` script other than `action/gallery` untouched by this task?
- [ ] C10 — Is `_html_escape`'s function signature and call-site usage unchanged from before
      this task (still called the same way at `action/gallery:187-188`)?
- [ ] C11 — Does this task's own change set leave `action/gallery verify::1`'s
      currently-unrelated drift (the `api:vulkan` example/tag, the 3 renamed `Fix(BUG-XXX)`
      placeholders) untouched — i.e. did this task not attempt to regenerate
      `examples/index.html`/`index.md` and thereby fold that unrelated drift into its own
      deliverable?

### Measurements

- [ ] M1 — `grep -c '&quot;Hello Triangle&quot;' examples/index.html` → 4 (was: 0
      correctly-escaped occurrences pre-fix — the pre-fix text read
      `"quot;Hello Triangle"quot;`, an unescaped `"` followed by literal `quot;` text, not
      `&quot;`)
- [ ] M2 — `grep -cE '<lt;|>gt;|"quot;' examples/index.html` → 0 (was: ≥5, per BUG-315's own
      Impact section documenting 5 corrupted lines pre-fix)

### Invariants

- [ ] I1 — reproducer test: `bash action/tests/gallery_test.sh` → exit 0
- [ ] I2 — script still valid bash: `bash -n action/gallery` → exit 0 (syntax-only check; no
      Rust compiler applies to this bash-only fix)

### Anti-faking checks

- [ ] AF1 — no mocking: `grep -c "Mock\|Fake\|Stub" action/tests/gallery_test.sh` → 0
- [ ] AF2 — test genuinely re-extracts the live function rather than asserting against a
      hardcoded/static string: `grep -c "sed -n" action/tests/gallery_test.sh` → ≥1
      (extraction mechanism present)
- [ ] AF3 — no disabled/skipped test: `grep -ci "skip\|ignore" action/tests/gallery_test.sh`
      → 0

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial: In Scope and Out of Scope checked for contradiction — none found; In Scope's "formal task registration and lifecycle walk" and Out of Scope's explicit exclusions (no further `action/gallery` edit, no sweep, no `verify::1` drift resolution) partition cleanly with no overlap | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial: does the Goal disclose the fix is already-applied rather than reading as forward-looking work? Confirmed yes — explicitly states "already applied and was independently reconfirmed live during this task's own filing (not merely cited from the bug file)", naming exact line ranges and the live verification commands run | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial: is there remaining value once the code fix already exists? Yes — BUG-315 sits at Verified (claimable/closeable) but unlinked to any task; this registration is the missing lifecycle step, matching the exact shape of precedent tasks 254/358/008. Value=6 (not TA060's 8-10 bug-fix default) checked against local precedent — 254 and 358 (both registration-only bug-promotion tasks) also use Value=6; consistent, not arbitrary | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial: T05 cites BUG-315's own pre-fix MRE as a non-reproducible-without-mutation historical record rather than a live command — checked whether this weakens "Testable" (D2) or leaves an unverifiable Acceptance Criterion; confirmed no AC/Checklist item depends on T05 being live-executable (T01's `gallery_test.sh` alone satisfies D2's Testable line), and T05 is explicitly labeled historical, not presented as live-passing — holds up as honest documentation, not a gap | — |
| D5 | Execution Scope | — | 🟢 | Adversarial: every deliverable path (`action/gallery`, `action/tests/gallery_test.sh`, `action/readme.md`, `action/tests/readme.md`, `examples/index.html`, `task/bug/verified/315_...md`) checked — all resolve inside this repository; zero external/cross-repo paths | — |
| D6 | Crate Scope Unity | — | 🟢 | Adversarial: `action/` has no `Cargo.toml` — is "crate" scope even well-defined here? Per task 008's own precedent (workspace-level task, root manifest treated as the single scope-unit), `action/` is treated as the single scope-unit — every authored deliverable lives inside it; `examples/index.html` is `action/gallery`'s own generated output cited only as a verification target, not a separately-authored deliverable, so it doesn't introduce a second scope-unit | — |
| D7 | Crate Locality | — | 🟢 | Adversarial: could the fix live anywhere other than `action/`? No — `_html_escape` is defined and consumed entirely within `action/gallery`; its test lives in the sibling `action/tests/`, the leaf location owning this functionality | — |
| D8 | Crate Single Responsibility | — | 🟢 | Adversarial: `action/readme.md`'s own stated responsibility — "Curated action scripts for the cgtools workspace" — checked for "and"-conjunction broadening; single sentence, no "and". This task's slice (fix one script's escaping bug) stays inside that existing responsibility, doesn't broaden it | — |
| **Total** | | — | 🟢 | 0 blocking findings; 3 adversarial challenges held up under scrutiny without requiring a fix (D3, D4, D6) | — |

**Aggregate verdict:** PASS — all 8 dimensions 🟢 on both passes (confirming + adversarial), first round, no fixes required.

## Journal

| Timestamp | Actor | Event | Note |
|-----------|-------|-------|------|
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | READINESS_GATE_PASS | Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS, 0 fixes needed |
| 2026-08-18 16:08:38 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 366` → blocked: "self-verification forbidden (actor matches filed_by)" — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 366` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |
| 2026-08-20 09:57:12 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-20 09:57:36 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-20 10:03:47 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 366` → exit 1, same-actor guard (unchanged). Round 7 fresh-reclaim re-confirmation: BUG-315 still 🎯 Verified, cited fix-file paths still resolve (mechanical drift check clean) |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` — Task filed via `bug_promote` skill (PROC12) to formally register
  BUG-315's already-applied, already-verified fix (`action/gallery`'s `_html_escape`, 4
  replacement strings `&`→`\&`, plus new `action/tests/gallery_test.sh` reproducer) as a
  tracked task, closing the bug. Goal/Scope/Delivery Requirements/Test Matrix/Acceptance
  Criteria derived from BUG-315's own Impact, Root Cause, Fix Location, and MRE sections.
  During filing, independently re-verified live (not merely cited from the bug file):
  `action/tests/gallery_test.sh` exits 0; `examples/index.html` contains
  `&quot;Hello Triangle&quot;` 4 times with zero broken-pattern matches. Also discovered
  during filing that `action/gallery verify::1` now exits 1 due to unrelated concurrent work
  (a new `api:vulkan` example plus 3 renamed `Fix(BUG-XXX)` placeholder comments in other
  examples) — confirmed via direct diff inspection to be entirely unrelated to this fix, and
  excluded from this task's Acceptance Criteria/Verification accordingly (see `## Out of
  Scope`).
- **[2026-08-18]** `RENUMBERED 357→366` — This task was originally filed as ID 357. An
  unbounded live disk scan performed immediately after filing (`find task -mindepth 1
  -iname "*.md"`, per this repo's own documented ID-collision precedent) found
  `task/draft/357_fix_quat_invert_wrong_for_non_unit_quaternions_bug298.md` already occupying
  357 — filed by a concurrently-running `bug_promote BUG-298` session actor (same sandbox,
  independent parallel activity; this repo's `task/readme.md` documents the same collision
  class for IDs 114-118/198/243). Neither side was wrong; genuine TOCTOU race between two
  live concurrent actors, caught before this task reached `unverified/`'s Tasks Index entry,
  its Readiness Verification Gate, or `tsk .claim_verify` — so the rename carries zero
  external cross-reference footprint to propagate (no Tasks Index row yet, no `tsk` CLI state
  yet, no inbound links from BUG-315 yet). Re-scanned disk immediately before renaming
  (highest ID in use: 365; 366 confirmed unclaimed) and moved the file to
  `task/unverified/366_gallery_html_escape_ampersand_backreference_fix_registration.md`.
- **[2026-08-18]** `READINESS_GATE_PASS` — Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS, 0
  fixes required. Adversarial pass challenged D3 (Value=6 vs TA060's 8-10 bug-fix default;
  held up against local precedent tasks 254/358), D4 (T05's non-live historical MRE citation;
  confirmed no AC/Checklist depends on it), and D6 (`action/` has no `Cargo.toml`; treated as
  the single scope-unit per task 008's own workspace-level precedent) — all three held up
  under scrutiny without requiring a fix. Re-ran all Verification-section commands live
  immediately before recording this gate (`gallery_test.sh` exit 0, `Fix(BUG-315)` grep=1,
  `&quot;Hello Triangle&quot;` grep=4, broken-pattern grep=0, `bash -n` exit 0, AF1=0, AF2=1,
  AF3=0 — all matching the values already documented in `## Verification`).
- **[2026-08-18]** `EXECUTED` — No new edit performed: the described fix (`action/gallery`'s
  `_html_escape`, `&`→`\&` in all 4 replacement strings, `Fix(BUG-315)`/`Root cause`/`Pitfall`
  source comment) and its reproducer (`action/tests/gallery_test.sh`) already existed on disk
  prior to this task's filing. This task's own contribution is the formal tracking
  registration and lifecycle walk, not the code change itself. `tsk .claim_verify 366`
  succeeded (state → 🔬 Verifying); `tsk .verify_pass 366` blocked by the same-actor guard
  (documented above in `## Journal`) — task left at 🔬 Verifying per standing sandbox
  limitation, not a quality defect.

## Related Documentation

- `task/bug/verified/315_gallery_html_escape_ampersand_backreference.md` — the source bug
  this task promotes; carries the full Root Cause/MRE/Prevention/History detail this task
  does not duplicate
- `action/readme.md` — `action/`'s own Responsibility Table, unchanged by this task
- `action/tests/readme.md` — `action/tests/`'s own Responsibility Table, unchanged by this
  task (already carries the `gallery_test.sh` row from the original fix)
