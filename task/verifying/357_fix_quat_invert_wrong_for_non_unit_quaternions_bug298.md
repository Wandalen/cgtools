# Fix `Quat::invert()` wrong for non-unit quaternions (BUG-298)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:54:17
- **expires_at:** 2026-08-20 00:54:17
- **round:** 1
- **state:** 🔬 (Verifying)
- **closes:** BUG-298
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/math/ndarray_cg
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **unverified_at:** 2026-08-19 22:37:54
- **unverified_by:** system
- **in_motion:** true
- **verifying_at:** 2026-08-19 22:54:17
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

BUG-298 (`task/bug/verified/298_quat_invert_wrong_for_non_unit_quaternions.md`, Medium
severity, 🎯 Verified) found `Quat<E>::invert()`
(`module/math/ndarray_cg/src/quaternion/arithmetics.rs:234-237`) returning the bare
conjugate unconditionally — correct only when the quaternion is unit-length
(`mag2() == 1`) — silently producing wrong results (scaled by the divisor's squared
magnitude) for every caller reached through it (`devide()`, `device_mut()`,
`Div`/`DivAssign` for `Quat`/`Quat`) whenever the right-hand operand is not
unit-length. The general multiplicative-inverse formula `q⁻¹ = conjugate(q) / |q|²`
reduces to the unit-length shortcut only when `|q|² = 1`; `invert()` implemented only
that shortcut, unconditionally, so any future caller dividing by a non-normalized
quaternion would silently get a wrong answer with no panic or error. The fix
(`self.conjugate()` → `self.conjugate() / self.mag2()`) is already applied and
independently confirmed via a round-trip reproducer test proving the defining
algebraic property of division, `(a / b) * b == a`, using deliberately non-unit
operands (bug file's own VERIFY Gate, 8/8 PASS, 2026-08-18). This task performs the
remaining lifecycle bookkeeping — `tsk.rulebook.md § Core Procedures : Procedure -
Promote Bug to Task` (PROC12) — to formally register that already-complete,
already-verified fix as a tracked task, closing BUG-298.
Testable: `cargo test -p ndarray_cg --all-features test_devide_non_unit_round_trip` →
`test result: ok. 1 passed`.

## In Scope

- `module/math/ndarray_cg/src/quaternion/arithmetics.rs` lines 221-237 — the
  already-applied `invert()` fix (`self.conjugate()` → `self.conjugate() /
  self.mag2()`), its updated doc comment, and its 3-field `Fix(BUG-298)`/`Root
  cause`/`Pitfall` source comment (verify all three are present; no further edit
  expected).
- `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` — the already-added
  `test_devide_non_unit_round_trip` reproducer test (`bug_reproducer(BUG-298)`
  marker) and its 5-section doc comment (Root Cause, Why Not Caught, Fix Applied,
  Prevention, Pitfall).
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/298_quat_invert_wrong_for_non_unit_quaternions.md`'s
  header back to this task via PROC12 Step 4 (performed as a follow-up edit once
  this file is filed).

## Out of Scope

- Any further code change to `module/math/ndarray_cg` — the fix is complete;
  `devide`/`device_mut`/`Div`/`DivAssign` (`src/quaternion/operator/div.rs`) needed
  no change, since all four already route through the now-corrected `invert()`
  (bug file's own Fix Location section; confirmed via `git diff --stat` showing
  `operator/div.rs` untouched).
- Refactoring unrelated code in `module/math/ndarray_cg` (e.g. other quaternion
  arithmetic methods not implicated by this defect).
- Changing the public API surface of `Quat<E>` — `invert()`'s signature (`pub fn
  invert( &self ) -> Self`) is unchanged; only the body's formula and doc comment
  changed.
- Fixing related-but-separate bugs — e.g. any other `invert`/`inverse`-named method
  elsewhere in the workspace carrying the same unchecked-precondition pattern named
  by BUG-298's own Prevention section (`grep -B2 "pub fn invert\|pub fn inverse"
  src/**/*.rs | grep -i "unit-length\|normalized\|assumes"`); no such instance was
  identified as in scope for this fix.
- Re-running BUG-298's MRE or its own VERIFY Gate — already run and recorded in the
  bug file's History (2026-08-18, 8/8 PASS); not re-litigated by this task's own
  Readiness Verification Gate, which checks task-file quality, not the underlying
  fix.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase
  .rulebooks`)
- Minimum applicable per `tsk.rulebook.md § Bug Fixes : Bug-Fixing Task Quality
  Requirements` (TA119): design, code_hyg_l1 (hygiene), test_organization, style

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase
  .rulebooks`)
- Failing-first evidence already on record: BUG-298's own MRE (pre-fix
  revert-and-rerun) reproduced `test_devide_non_unit_round_trip` failing with `left
  = Quat([135.0, 270.0, 405.0, 540.0])` vs `right = Quat([1.0, 2.0, 3.0, 4.0])` (bug
  file Symptom/MRE sections, 2026-08-18) — this task does not re-derive that
  evidence
- Fix already applied: `module/math/ndarray_cg/src/quaternion/arithmetics.rs:236`
  states `self.conjugate() / self.mag2()`, with an updated doc comment describing
  the general inverse formula and a 3-field `Fix(BUG-298)` source comment
- Green state already confirmed, and re-confirmed live by this task: `cargo test -p
  ndarray_cg --all-features test_devide_non_unit_round_trip` passes; full scoped
  suite (`verb/test_only pkg::ndarray_cg`, 282 tests) passes with 0 failures;
  `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings` clean
- No refactor needed — single-line formula change plus a doc-comment update, no
  structural churn
- Fix documentation already complete: source carries the 3-field
  `Fix(BUG-298)`/`Root cause`/`Pitfall` comment
  (`src/quaternion/arithmetics.rs:225-231`); the reproducer test carries the
  5-section `Root Cause`/`Why Not Caught`/`Fix Applied`/`Prevention`/`Pitfall` doc
  comment (`tests/inc/quat_test/arithmetic.rs:57-86`) — this task does not
  duplicate it, only cross-links via `closes: BUG-298`
- No mocking or faking anywhere in the reproducer test or fix — both use real
  `QuatF64` arithmetic
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected
  to hit this sandbox's known same-actor guard, per project convention — document
  rather than force/spoof if so)
- The Test Matrix below is populated (rows T01-T05, all already-executed and
  live-confirmed) before any test code is written or modified — no test code is
  added or changed by this task, only registered
- Independent verification (a verifier distinct from `filed_by`, per this task's own
  Verification `Execution` clause) must pass before this task's state is updated to
  ✅ Completed
- This task's state reaches ✅ Completed only upon that independent verification
  pass at 🔎 Accepting — never self-assigned by the filing/executing actor

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo test -p ndarray_cg --all-features test_devide_non_unit_round_trip` | Fixed `invert()` (`self.conjugate() / self.mag2()`) | `test result: ok. 1 passed` |
| T02 | `grep -n "self.conjugate() / self.mag2()" module/math/ndarray_cg/src/quaternion/arithmetics.rs` | Fixed formula present in source | 1 match, line 236 |
| T03 | `verb/test_only pkg::ndarray_cg` (full scoped suite) | All `ndarray_cg` tests against fixed code | 0 failures (282 passed, live-confirmed this task) |
| T04 | `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings` | Fixed crate, lint-clean | 0 warnings (exit 0, live-confirmed this task) |
| T05 | Pre-fix evidence (already on record, bug file History 2026-08-18: revert-and-rerun) | Unfixed `invert()` = `self.conjugate()` | `test_devide_non_unit_round_trip` FAILED — `left=Quat([135,270,405,540])` vs `right=Quat([1,2,3,4])`; not re-derived by this task |

## Acceptance Criteria

- `module/math/ndarray_cg/src/quaternion/arithmetics.rs:236` states
  `self.conjugate() / self.mag2()`, not the bare `self.conjugate()`
- The same function's doc comment (lines 221-222) describes the general inverse
  formula, not only the unit-length special case
- Lines 223-231 carry the 3-field source comment: `Fix(BUG-298)`, `Root cause`,
  `Pitfall`
- `test_devide_non_unit_round_trip` exists in
  `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` with a `// test_kind:
  bug_reproducer(BUG-298)` marker
- The reproducer test carries a 5-section doc comment: Root Cause, Why Not Caught,
  Fix Applied, Prevention, Pitfall
- `cargo test -p ndarray_cg --all-features test_devide_non_unit_round_trip` passes
- No mocking or faking in the reproducer test or fix (real `QuatF64` arithmetic
  only)
- `module/math/ndarray_cg/src/quaternion/operator/div.rs` is unmodified — no change
  was needed there, since `devide`/`Div`/`DivAssign` already route through the
  now-fixed `invert()`
- `task/bug/verified/298_quat_invert_wrong_for_non_unit_quaternions.md`'s header
  states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- This task's `closes:` field names `BUG-298`
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `module/math/ndarray_cg/src/quaternion/arithmetics.rs:236` state
  `self.conjugate() / self.mag2()`?
- [ ] C2 — Does the same function's doc comment describe the general inverse
  formula rather than only the unit-length special case?
- [ ] C3 — Does the same function carry the 3-field source comment
  (`Fix(BUG-298)`, `Root cause`, `Pitfall`)?
- [ ] C4 — Does `cargo test -p ndarray_cg --all-features
  test_devide_non_unit_round_trip` succeed?

**Test**
- [ ] C5 — Does `bug_reproducer(BUG-298)` exist in
  `module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs`?
- [ ] C6 — Does the reproducer test carry the 5-section doc comment (Root Cause,
  Why Not Caught, Fix Applied, Prevention, Pitfall)?

**Registration correctness**
- [ ] C7 — Does this task's `closes:` field name `BUG-298`?
- [ ] C8 — Does BUG-298's own header carry a `**Fix Task:**` line pointing back at
  this task's ID?

**No-mocking confirmation**
- [ ] C10 — Does `grep -rn "Mock\|Fake\|Stub"
  module/math/ndarray_cg/src/quaternion/arithmetics.rs
  module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` return 0 matches (no
  mocking or faking in either the fix or the reproducer test)?

**Out of Scope confirmation**
- [ ] C9 — Is `module/math/ndarray_cg/src/quaternion/operator/div.rs` untouched by
  this task (`git diff --stat` empty for that path)?
- [ ] C11 — Is `git diff --stat -- module/math/ndarray_cg/src/quaternion/` free of
  any hunk outside `arithmetics.rs` (confirming no refactor of unrelated quaternion
  arithmetic methods)?
- [ ] C12 — Does `invert()`'s public signature (`pub fn invert( &self ) -> Self`)
  remain textually unchanged (`grep -n "pub fn invert" arithmetics.rs` still matches
  the pre-fix signature)?
- [ ] C13 — Does `grep -rn "pub fn invert\|pub fn inverse"` across
  `module/math/ndarray_cg/src` show no method other than `Quat::invert()` modified by
  this task (confirming no related-but-separate `invert`/`inverse` bug was fixed)?
- [ ] C14 — Does `task/bug/verified/298_quat_invert_wrong_for_non_unit_quaternions.md`'s
  own Verification Record/History still show only its original 2026-08-18 VERIFY Gate
  entries, with no new MRE or VERIFY Gate re-run added by this task?

**Test Matrix confirmation**
- [ ] C15 — Do all five Test Matrix rows (T01-T05) show the outcome stated in their
  own Expected Behavior column (T01/T04 live-rerun this task; T02/T03 cross-checked
  via M2/I2; T05 pre-existing evidence, not re-derived)?

### Measurements

- [ ] M1 — `grep -c "self.conjugate()$" module/math/ndarray_cg/src/quaternion/arithmetics.rs` → 0 (was: 1, pre-fix — bare-conjugate-only form now absent from `invert()`'s body)
- [ ] M2 — `grep -c "self.conjugate() / self.mag2()" module/math/ndarray_cg/src/quaternion/arithmetics.rs` → 1 (was: 0, pre-fix)

### Invariants

- [ ] I1 — `module/math/ndarray_cg/src/quaternion/operator/div.rs` unaffected:
  `git diff --stat -- module/math/ndarray_cg/src/quaternion/operator/div.rs` →
  empty
- [ ] I2 — full scoped suite green: `verb/test_only pkg::ndarray_cg` → 0 failures
  (282 passed, live-confirmed this task's own filing, 2026-08-18)
- [ ] I3 — compiler/lint clean: `cargo clippy -p ndarray_cg --all-targets
  --all-features -- -D warnings` → 0 warnings

### Anti-faking checks

- [ ] AF1 — the fix changes only `invert()`'s formula and doc comment, not its
  signature or any caller (`devide`/`Div`/`DivAssign`) — checked by reading the
  literal diff at `arithmetics.rs:221-237` and confirming `operator/div.rs` is
  untouched, not just the presence of the new formula string
- [ ] AF2 — no mocking: `grep -rn "Mock\|Fake\|Stub"
  module/math/ndarray_cg/tests/inc/quat_test/arithmetic.rs` → 0 matches

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 15:54:05 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 16:08:07 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 16:08:17 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 16:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 357` → blocked: "self-verification forbidden (actor matches filed_by)" — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:41:16 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:44:40 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:54 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:54:17 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:54:17 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 357` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` — Task filed via `bug_promote` skill (PROC12) to formally
  register BUG-298's already-applied, already-verified fix
  (`module/math/ndarray_cg/src/quaternion/arithmetics.rs:224-237`,
  `self.conjugate()` → `self.conjugate() / self.mag2()`) as a tracked task, closing
  the bug.
- **[2026-08-18]** `VERIFIED` (self-check) — Readiness Verification Gate (TA095,
  MAAV Tier 2 Dual-Role Self-Check) run directly in-context, never delegated: 8/8
  dimensions PASS, both confirming and adversarial passes agree, no Blocking
  Findings. Full record in `## Verification Record` below. `tsk .verify_pass 357`
  then attempted per standard lifecycle and blocked by this sandbox's known
  same-actor guard (`self-verification forbidden (actor matches filed_by)` — actor
  and `filed_by` are both `user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/`,
  precedented across tasks 254/197/203/224/219/251/358 in this same repo). Not
  forced, spoofed, or bypassed per standing project convention — task left at 🔬
  Verifying, readiness content complete and passing, pending an independent-actor
  `tsk .verify_pass` run.

## Related Documentation

- `task/bug/verified/298_quat_invert_wrong_for_non_unit_quaternions.md` — the
  source bug this task promotes; carries the full Root Cause/MRE/Prevention/History
  detail this task does not duplicate
- `module/math/ndarray_cg/src/quaternion/operator/div.rs` — `devide`/`Div`/
  `DivAssign` (confirmed unaffected, not modified by this task; they route through
  the now-fixed `invert()`)

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by
user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

**Pass 1 (Confirming) evidence, by dimension:**
- **D1 Scope Coherence:** In Scope (4 bullets) and Out of Scope (5 bullets) both
  non-empty; observable outcome is BUG-298's fix formally registered and closed as a
  tracked task; bounded and verifiable without a follow-up task.
- **D2 MOST Goal Quality:** Motivated (explains the wrong general-formula shortcut
  and its downstream blast radius through `devide`/`Div`/`DivAssign`); Observable
  (`arithmetics.rs:234-237`, exact formula change); Scoped (registers one
  already-complete fix only); Testable (`cargo test -p ndarray_cg --all-features
  test_devide_non_unit_round_trip` → `test result: ok. 1 passed`).
- **D3 Value / YAGNI:** Null Hypothesis — skipping this task leaves BUG-298
  untracked as closed despite a live fix, breaking bug↔task closure provenance;
  concrete committed need already exists (`bug_promote` skill + PROC12 exist
  specifically for this workflow; 3+ precedent tasks in this repo, e.g. 254/358/359,
  follow the identical pattern).
- **D4 Implementation Readiness:** Delivery Requirements are concrete (exact
  files/lines/commands, no "implement the feature" vagueness); Test Matrix populated
  (T01-T05, all three columns); Acceptance Criteria all name an exact file, line, or
  command.
- **D5 Execution Scope:** Every path in Goal/In Scope/Acceptance Criteria
  (`module/math/ndarray_cg/...`, `task/bug/verified/298_...md`) resolves inside this
  repo; `repo_identity: self` set accordingly.
- **D6 Crate Scope Unity:** All code/test deliverable paths resolve inside exactly
  one crate, `module/math/ndarray_cg`; the bug-header link-back
  (`task/bug/verified/298_...md`) is task-system bookkeeping, not a crate deliverable
  — same treatment as precedent tasks 254/358/359 (all `unit_type: module`, all
  PASSed this same dimension with an identical bug-header-link bullet).
- **D7 Crate Locality:** `ndarray_cg` is the leaf crate that owns `Quat` and its
  arithmetic directly (`self.conjugate() / self.mag2()` is real algorithmic logic
  in-crate, not orchestration/re-export glue) — not an aggregator.
- **D8 Crate Single Responsibility:** `ndarray_cg`'s responsibility (math
  primitives: vectors/matrices/quaternions) is unchanged and statable without "and";
  this task only touches existing quaternion-arithmetic surface plus its own tests,
  no second concern grafted on.

**Pass 2 (Adversarial) attempts, by dimension:**
- **D1:** Attempted to argue this is a "paperwork-only" task with no real doing
  since the code fix predates it — rejected: registering an already-verified fix as
  a tracked, closed task is this repo's own established bug-closure convention
  (`bug_promote` skill exists for exactly this), not filler. No overlap/contradiction
  found between In Scope and Out of Scope bullets.
- **D2:** Attempted to find a vague or unrunnable Testable clause — the stated
  command is exact and directly runnable; attempted to find an Observable claim not
  tied to a real line number — none found, all cite `arithmetics.rs:221-237`
  specifically.
- **D3:** Attempted to disprove the committed need by treating this as bureaucratic
  overhead — rejected: the bug's own lifecycle (`bugs/file.rulebook.md`) requires
  formal closure via a linked task, and this is a pre-existing procedural
  requirement independent of this session, not invented busywork.
- **D4:** Scanned every Delivery Requirements bullet for vagueness — the one
  boilerplate rulebook-adherence bullet is a standard required constraint (TA119),
  not a deliverable description; every other bullet ties to a real artifact.
  Attempted to find a Test Matrix row missing a column — none found.
- **D5:** Re-scanned every path cited across Goal/In Scope/Acceptance Criteria for
  any absolute path or sibling-repo reference — none found; all paths are
  repo-relative and resolve under this repo's own tree.
- **D6:** Attempted to argue the bug-header edit (`task/bug/verified/298_...md`)
  breaks single-crate unity since it's outside `module/math/ndarray_cg` — rejected
  on direct precedent: tasks 254, 358, and 359 all perform the identical bug-header
  link-back edit, all declare `unit_type: module`, and all record PASS 8/8 on this
  same dimension: this repo's established interpretation treats task-system
  bookkeeping paths (`task/bug/...`, the task file itself) as exempt from crate-scope
  unity.
- **D7:** Attempted to find orchestration-only/re-export code masquerading as a real
  fix — the changed line is a genuine formula computation inside the owning crate,
  not a thin pass-through; no aggregator crate is involved anywhere in this task.
- **D8:** Attempted to argue "fix code" + "add test" + "register task" is three
  concerns, not one — rejected: test coverage is inherent to a crate's own single
  math-correctness responsibility (every crate here carries its own tests/), and
  task-registration bookkeeping touches the task system, not the crate's charter, so
  it does not graft a second responsibility onto `ndarray_cg` itself.

No dimension surfaced a Blocking Finding on either pass. Aggregate verdict: PASS
8/8.
