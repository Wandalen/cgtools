# Register PEC reader's stitch_block_len underflow fix (closes BUG-314)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:45:32
- **expires_at:** 2026-08-20 00:45:32
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** BUG-314
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/embroidery_tools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **in_motion:** true
- **verifying_at:** 2026-08-19 22:45:32
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **unverified_at:** 2026-08-19 22:37:55
- **unverified_by:** system

## Goal

BUG-314 (`task/bug/verified/314_pec_stitch_block_len_underflow.md`, High severity,
🎯 Verified) found `content_read` (PEC reader) computing
`stitch_block_len - 5 + reader.stream_position()?` via a raw `-` operator on
`stitch_block_len` -- a 24-bit value read directly from untrusted file bytes
(`reader.read_u24::<LE>()?`) -- which underflows for any value under 5: panics in a
debug build ("attempt to subtract with overflow") and silently wraps to a value near
`u64::MAX` in a release build, corrupting a subsequent
`reader.seek(SeekFrom::Start(stitch_block_end))` call. The bug's own Evidence Table
(E3) confirms this same `content_read` function is also reached indirectly via 2
`pes::*` call sites, so both `.pec` and `.pes` untrusted-file parsing are affected.
The fix -- `stitch_block_len.checked_sub(5).ok_or_else(|| EmbroideryError::DecodingError(...))?`,
with a 3-field `Fix(BUG-314)`/`Root cause`/`Pitfall` source comment -- is already
applied and committed (`module/helper/embroidery_tools/src/format/pec/reader.rs:121-131`,
confirmed live via `git show 46a4bd2f`), and independently confirmed via a reproducer
test (`content_read_rejects_stitch_block_len_below_5_instead_of_underflowing`,
`tests/pec_test.rs:259-272`) that returns `DecodingError` instead of panicking (bug
file's own VERIFY Gate, 8/8 PASS, 2026-08-18). This task performs the remaining
lifecycle bookkeeping -- `tsk.rulebook.md § Core Procedures : Procedure - Promote Bug
to Task` (PROC12) -- to formally register that already-complete, already-verified
fix as a tracked task, closing BUG-314.
Testable: `cd module/helper/embroidery_tools && cargo nextest run -E
'test(content_read_rejects_stitch_block_len_below_5_instead_of_underflowing)'
--all-features` → 1 passed, 0 failed.

## In Scope

- `module/helper/embroidery_tools/src/format/pec/reader.rs` lines 121-131 -- the
  already-applied `checked_sub( 5 )` guard fix and its `Fix(BUG-314)`/`Root cause`/
  `Pitfall` source comment (verify both are present; no further edit expected).
- `module/helper/embroidery_tools/tests/pec_test.rs` lines 208-272 -- the
  already-added `build_pec_content_with_stitch_block_len` helper and
  `content_read_rejects_stitch_block_len_below_5_instead_of_underflowing` reproducer
  test (verify present and passing; no further edit expected).
- Formal task registration and lifecycle walk (claim-for-verify, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/314_pec_stitch_block_len_underflow.md`'s header back to
  this task via PROC12 Step 4 (performed as a follow-up edit once this file is
  filed).

## Out of Scope

- Any further code change to `module/helper/embroidery_tools` -- the fix is
  complete and independently verified; no additional editing expected in this
  crate.
- Any other `read_u*`-then-raw-arithmetic call site in `pec/reader.rs`/
  `pes/reader.rs` -- BUG-314's own Prevention section names a starting-point
  detection grep for human review, not a completed audit; auditing/fixing further
  sites is a separate, not-yet-filed concern.
- BUG-234 (the writer-path `128_usize.wrapping_sub( count )` defect) -- already
  separately fixed prior to this bug, a different (though related-pattern) defect
  in a different code path (writer, not reader); confirmed untouched by this task
  (`git diff --stat -- module/helper/embroidery_tools/src/format/pec/writer.rs`
  empty).
- Re-running BUG-314's own MRE or its own VERIFY Gate -- already run and recorded
  in the bug file's History (2026-08-18, Tier 2 Dual-Role Self-Check, 8/8 PASS);
  not re-litigated by this task's own Readiness Verification Gate, which checks
  task-file quality, not the underlying fix's correctness a second time.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Failing-first evidence already on record: BUG-314's own Root Cause section
    demonstrates the pre-fix `stitch_block_len - 5` expression underflows for
    `stitch_block_len < 5` (Rust's documented debug-mode overflow-check default);
    the reproducer test cannot execute the panicking pre-fix expression directly (a
    panic would abort the test process), so the divergence is demonstrated via the
    fix itself, per the bug file's own MRE section -- this task does not re-derive
    that evidence
-   Fix already applied and committed: `module/helper/embroidery_tools/src/format/pec/reader.rs:121-131`
    states the `checked_sub( 5 ).ok_or_else( .. )?` guard, with the 3-field
    `Fix(BUG-314)`/`Root cause`/`Pitfall` source comment in place (confirmed live via
    `git show 46a4bd2f -- module/helper/embroidery_tools/src/format/pec/reader.rs`)
-   Green state confirmed live in this task's own filing session (not merely cited
    from the bug file): `content_read_rejects_stitch_block_len_below_5_instead_of_underflowing`
    passes in isolation (`cargo nextest run -E '...' --all-features` → 1 passed, 0
    failed) and the full crate suite passes with no regressions
    (`cargo nextest run --all-features` → 17 passed, 0 failed);
    `RUSTFLAGS="-D warnings" cargo check -p embroidery_tools --all-features` → 0
    warnings
-   No refactor needed -- the fix is a minimal `checked_sub`+`ok_or_else`
    substitution, no structural churn
-   Fix documentation already complete at the bug level: BUG-314 carries the
    5-section fix documentation (Root Cause, Why Not Caught, Fix Location,
    Prevention, Generalized Version) plus the 3-field source comment -- this task
    does not duplicate it, only cross-links via `closes: BUG-314`
-   Task state reaches 🎯 on this task file's own Readiness Verification Gate;
    `tsk .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle
    (expected to hit this sandbox's known same-actor guard, per project convention
    -- document rather than force/spoof if so)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo nextest run -E 'test(content_read_rejects_stitch_block_len_below_5_instead_of_underflowing)' --all-features` (run from `module/helper/embroidery_tools`) | Reproducer test with a constructed `stitch_block_len = 0` buffer | 1 test run: 1 passed, 0 failed -- `content_read` returns `Err(EmbroideryError::DecodingError(_))` |
| T02 | `grep -n "checked_sub( 5 )" module/helper/embroidery_tools/src/format/pec/reader.rs` | Fixed guard present in source | 1 match, non-empty |
| T03 | `RUSTFLAGS="-D warnings" cargo check -p embroidery_tools --all-features` | Crate compiles with the fix in place | 0 errors, 0 warnings |
| T04 | `cargo nextest run --all-features` (run from `module/helper/embroidery_tools`) | Full crate suite, no regressions | 17 tests run: 17 passed, 0 failed |

## Acceptance Criteria

-   `module/helper/embroidery_tools/src/format/pec/reader.rs` replaces the raw
    `stitch_block_len - 5` with `stitch_block_len.checked_sub( 5 ).ok_or_else( .. )?`,
    returning `EmbroideryError::DecodingError` for any length under 5
-   The fix's source comment at `reader.rs:122-128` carries all 3 required fields:
    `Fix(BUG-314)`, `Root cause`, `Pitfall`
-   `content_read_rejects_stitch_block_len_below_5_instead_of_underflowing` exists
    in `module/helper/embroidery_tools/tests/pec_test.rs`, tagged
    `bug_reproducer(BUG-314)`, and passes
-   `task/bug/verified/314_pec_stitch_block_len_underflow.md`'s header states
    `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT
self-verify -- an independent verifier performs the walk after the task reaches
🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `module/helper/embroidery_tools/src/format/pec/reader.rs` contain
      `stitch_block_len.checked_sub( 5 )` (not a raw `stitch_block_len - 5`)?
- [ ] C2 — Does the same fix's source comment carry `Fix(BUG-314)`, `Root cause`,
      and `Pitfall` fields?
- [ ] C3 — Does `content_read_rejects_stitch_block_len_below_5_instead_of_underflowing`
      exist in `module/helper/embroidery_tools/tests/pec_test.rs`, tagged
      `bug_reproducer(BUG-314)`?
- [ ] C4 — Does `RUSTFLAGS="-D warnings" cargo check -p embroidery_tools --all-features`
      succeed with 0 errors and 0 warnings?

**Registration correctness**
- [ ] C5 — Does this task's `closes:` field name `BUG-314`?
- [ ] C6 — Does BUG-314's own header carry a `**Fix Task:**` line pointing back at
      this task's ID?

**Out of Scope confirmation**
- [ ] C7 — Is the raw pre-fix expression `stitch_block_len - 5` absent from
      `module/helper/embroidery_tools/src/format/pec/reader.rs`?
- [ ] C8 — Is BUG-234's writer-path fix location
      (`module/helper/embroidery_tools/src/format/pec/writer.rs`) untouched by this
      task (`git diff --stat` empty for that path)?

### Measurements

- [ ] M1 — raw underflow expression absent: `grep -c "stitch_block_len - 5" module/helper/embroidery_tools/src/format/pec/reader.rs` → 0 (was: 1, pre-fix)
- [ ] M2 — checked_sub guard present: `grep -c "checked_sub( 5 )" module/helper/embroidery_tools/src/format/pec/reader.rs` → ≥1 (was: 0, pre-fix)

### Invariants

- [ ] I1 — crate test suite: `cargo nextest run -p embroidery_tools --all-features` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p embroidery_tools --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — the fix returns a real decode error, not a swallowed/ignored result: `grep -n "checked_sub( 5 )" -A1 module/helper/embroidery_tools/src/format/pec/reader.rs | grep -c "ok_or_else"` → ≥1 (guard is chained into a real `?`-propagated `Err`, not silently defaulted)
- [ ] AF2 — reproducer test is not disabled or trivial: `grep -c '#\[ ignore \]\|assert!( true )' module/helper/embroidery_tools/tests/pec_test.rs` → 0

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Confirming: In Scope (4 bullets) and Out of Scope (3 bullets) both non-empty; observable outcome is BUG-314 gaining a `Fix Task:` link and a tracked closure record, not merely "write a file". Adversarial: tried to disprove In Scope bullet 3 ("lifecycle walk") as self-referential meta-busywork rather than real scope — checked against precedent task 254, which uses the identical claim/verify/link-back scope item for the same registration-task shape; not a defect, an established pattern. Tried Out-of-Scope's BUG-234 exclusion as padding — confirmed relevant (same file, superficially similar wrapping-subtraction pattern, heads off a real scope-creep question). Scope Sizing Gate (same 3 checks) passes. | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass: Motivated clause explains the "why now" (fix already exists and is verified, only bookkeeping remains) but never states registration's own benefit as a standalone sentence (traceability/auditability); Testable clause's runnable command covers only the code-level claim (T01), not the registration-level claims (BUG-314 link, task state), which are instead covered by Checklist C5/C6 rather than the Goal's own Testable line. Matches precedent task 254's identical registration-task shape exactly — non-blocking, not a defect unique to this task. | — |
| D3 | Value / YAGNI | — | 🟢 | Confirming: Null Hypothesis answered (skipping leaves BUG-314 without a Fix Task link and `task/readme.md` without a closure record — both already-existing, concrete gaps, confirmed via Read prior to filing). Adversarial: tried to disprove this task itself as ceremonial YAGNI busywork — checked whether the gap is real vs speculative; confirmed real (BUG-314's header had no Fix Task line as of this session's own pre-edit read). Value scored modestly in Metrics (registration of an already-verified fix, not new engineering), consistent with precedent's own judgment call. | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial: tried to disprove Delivery Requirements as vague — found them retrospective/confirmatory ("already applied", "confirmed live") rather than imperative, but that is the correct shape for a registration-only task (no remaining code work to prescribe) and matches precedent 254 exactly. Cross-checked Test Matrix T01-T04 against Acceptance Criteria for padding — all 4 rows traceable to a specific AC bullet, none found decorative. | — |
| D5 | Execution Scope | — | 🟢 | Adversarial: re-scanned every path named in `## Goal`/`## In Scope`/`## Acceptance Criteria` specifically (`reader.rs`, `pec_test.rs`, `task/bug/verified/314_...md`) — all resolve under `/home/user1/pro/lib/yrd_gamedev/cgtools`; none cross into another repository. `pes/reader.rs` (Related Documentation only) correctly excluded from this check's scan set. | — |
| D6 | Crate Scope Unity | — | 🟢 | BUG-314 link-back touches `task/bug/verified/314_pec_stitch_block_len_underflow.md`, a tracking file outside `unit_type: module`'s crate boundary (`module/helper/embroidery_tools`) — same disposition as every other bug-promotion cross-link in this repo (tracking-file edits are not crate-scope violations; confirmed against precedent task 254's identical D6 note). Adversarial: tried treating the bug-file edit as a genuine second crate — D7's own enumeration (code/test/doc/example/fixture/benchmark/changelog) categorically excludes task-registry markdown, so it doesn't count. | — |
| D7 | Crate Locality | — | 🟢 | Adversarial: tried to disprove `embroidery_tools` as a leaf crate (i.e., a possible aggregator re-exporting format-specific sub-crates) — confirmed `src/format/pec/reader.rs` and `src/format/pes/reader.rs` live directly inside `embroidery_tools`'s own tree, not in separate sub-crates; it directly implements PEC/PES parsing, not composition-only. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Adversarial: tried to disprove via "0 code changes make this vacuous" — confirmed this is the expected, correct outcome for a registration-only task (matches precedent 254, whose underlying fix was likewise pre-existing); crate responsibility ("parses and writes embroidery machine file formats") is unchanged and still statable without "and". | — |
| **Total** | | — | 🟢 | 2 non-blocking (D2, D6) | — |

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 16:03:40 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 16:10:44 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 365` → blocked: "self-verification forbidden (actor matches filed_by)" — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 365` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` — Task filed via `bug_promote` skill (PROC12) to formally register BUG-314's already-applied, already-verified fix (`module/helper/embroidery_tools/src/format/pec/reader.rs:121-131`, raw `stitch_block_len - 5` → `checked_sub( 5 ).ok_or_else( .. )?`) as a tracked task, closing the bug. Pre-fix content (`stitch_block_len - 5`, count 1) and post-fix content (`checked_sub( 5 )`, count 1) both confirmed live via `git show 46a4bd2f^`/current source; reproducer test and full crate suite (17 tests) both confirmed passing live via `longrun`-detached `cargo nextest run`; `cargo check -p embroidery_tools --all-features` under `RUSTFLAGS="-D warnings"` confirmed clean live.
- **[2026-08-18]** `RENUMBERED` — 356 → 360 → 365, two hops within minutes of first filing: a concurrent session actor (same sandbox identity, independent activity, actively promoting a batch of BUG-3xx-series bugs — BUG-298/300/311/312/313/315 confirmed live in `draft/`/`unverified/`/`verifying/` during this task's own filing window) claimed 356 (`draft/356_character_control_yaw_halving_fix_registration.md`) and then 360 (`draft/360_character_control_yaw_halving_fix_registration.md`) within the same short window this file independently computed and claimed each of those same IDs from its own on-disk scan — a genuine TOCTOU race between two live actors, not a defect in either side's allocation logic (confirmed via `tsk .check task` surfacing the live collision and a live `find` re-scan immediately before each rename). 365 confirmed free via immediate `find`-based re-check before and after the move. No internal self-reference to the old ID existed in this file's own body (verified via `grep -n "356"` prior to the first move), so only the filename changed each hop.
- **[2026-08-18]** `READINESS_GATE_PASS` — Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS. Two non-blocking adversarial-pass notes recorded (D2: Testable clause covers only the code-level claim, registration-level claims covered by Checklist C5/C6 instead; D6: BUG-314 link-back touches a tracking file outside the crate boundary, same disposition as every other bug-promotion cross-link in this repo) — neither is a Blocking Finding. No fixes required this round.
- **[2026-08-18]** `EXECUTED` — No new edit performed: the described fix (`module/helper/embroidery_tools/src/format/pec/reader.rs:121-131` `stitch_block_len - 5` → `checked_sub( 5 ).ok_or_else( .. )?`, `Fix(BUG-314)`/`Root cause`/`Pitfall` comment) already existed on disk and was already committed (`git show 46a4bd2f`) prior to this task's filing. This task's own contribution is the formal tracking registration and lifecycle walk, not the code change itself. `tsk .claim_verify 365` succeeded; `tsk .verify_pass 365` blocked by the same-actor guard (documented above) — task left at 🔬 Verifying per standing sandbox limitation, not a quality defect.

## Related Documentation

- `task/bug/verified/314_pec_stitch_block_len_underflow.md` — the source bug this
  task promotes; carries the full Root Cause/MRE/Prevention/History detail this
  task does not duplicate
- `module/helper/embroidery_tools/src/format/pes/reader.rs` — 2 call sites (`:60`,
  `:85`) reaching the same fixed `content_read` function via `.pes` parsing
  (confirmed, not modified by this task)
