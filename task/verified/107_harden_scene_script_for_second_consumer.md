# Add a whole-AST purity check to `scene_script`, enforcing the script-as-data invariant

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **unit_type:** module
- **unit:** module/helper/scene_script
- **repo_identity:** self
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **verification_date:** 2026-08-13
- **blocked_by:** null

## Goal

`docs/pattern/004_script_as_data.md` names a real invariant — "the script cannot
call the engine" — that today has zero code-level enforcement: the only known
use (`examples/orrery/webgpu/src/scene.rs::SceneConfig::load()`) satisfies it
purely "by inspection" (pattern/004's own words), and `top_level_lint.rs`'s
existing checker is explicitly documented as structural-only, checking top-level
*shape*, never whether a `let` initializer or nested expression anywhere in the
AST calls into engine-registered vocabulary
(`pitfall/002_checker_is_structural_not_semantic.md`). This gap is real and
present today regardless of any other consumer: `scene_script` documents a
purity contract it cannot currently check. This task closes it: a new whole-AST
purity-check function walks every statement and expression in a compiled
script, top-level and nested alike, and rejects the first `FnCall`/`MethodCall`
node found anywhere — with no exception for operator calls, since an operator
like `+`/`*` desugars to a registered `Engine::register_fn` call same as any
named function. Success is observable as: a new public function in
`scene_script` that returns `Err` naming the offending call for an impure
script and `Ok(())` for a pure one, covered by tests that fail without the
change and pass with it — run via `cargo nextest run -p scene_script`.

## In Scope

- A new whole-AST purity-check function in `module/helper/scene_script/src/`
  (new file, e.g. `purity_lint.rs`, sibling to `top_level_lint.rs`) that
  recursively walks every expression position reachable from a `rhai::AST` —
  including but not limited to `let` initializers, array elements,
  object-map values, and the bodies of any nested block/if/loop/try-catch
  construct — and returns `Err` naming the first `FnCall`/`MethodCall` found
  anywhere, with no exception for operator calls (see `top_level_lint.rs`'s
  own `Role::PlainExpression` doc comment on why operator calls are real
  engine calls too) — a script-as-data document uses only Rhai's native
  `#{...}`/`[...]`/literal syntax, per `docs/pattern/004`'s own description
  of `orrery_webgpu`'s `scene.rhai` containing "zero function or operator
  calls". A valid pure document has no control-flow constructs at all, but
  the check must still descend into one if present, to name the call
  hiding inside it rather than stopping at the construct's own shape.
- `docs/invariant/004_script_as_data_purity.md` (new instance in
  `module/helper/scene_script/docs/invariant/`): states the purity property
  the new check enforces, its enforcement mechanism (the new function), and
  violation consequences (caller receives `Err` naming the offending call).
- `docs/definition/readme.md` count update for the new invariant instance
  (3 → 4) plus a new row in its Overview Table.
- `docs/feature/001_rhai_scene_scripting.md`'s Invariants table gains a row
  for the new instance (it is the crate's navigational hub; leaving it stale
  would break Step 3's cross-reference check on the next `doc_tsk` pass over
  this crate).

## Out of Scope

- Extracting a reusable load-and-deserialize helper generalizing
  `SceneConfig::load()`'s inline `engine_build()` → `eval()` →
  `from_dynamic()` sequence. No second concrete caller exists yet: the one
  currently-planned second script-as-data consumer
  (`codename_space_sandbox` task 007) uses that same inline sequence
  directly rather than a shared helper (see that task's own Out of Scope),
  so extracting an abstraction now would be premature generalization
  validated by only one shape — orrery's own, which stays unmodified per
  the next bullet. Revisit if and when a second real caller wants to share
  the sequence.
- Refactoring `examples/orrery/webgpu/src/scene.rs::SceneConfig::load()` to
  adopt the new purity check — `examples/orrery` is a separate crate outside
  `module/helper/scene_script`'s own roof; adopting it there is a follow-on
  task if ever pursued, not part of hardening `scene_script` itself.
- Adding a script-facing color type or `Squad` easing-curve support
  (`pitfall/006`'s open gap) — unrelated scope, no dependency either
  direction.
- Building or wiring any actual script-as-data consumer — that is
  `codename_space_sandbox` task 007 (a different repository), which does
  not require this task's deliverable to exist first (no `blocked_by`
  relationship either direction).
- Extending the purity check to catch non-call side channels (e.g. a
  `const` referencing engine-registered constants) — no such channel is
  known to exist in the current binding surface; adding speculative
  handling for one would be undocumented, unverifiable scope.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code (code tasks)
-   Every Test Matrix case is backed by a test that failed before its implementing change landed (code tasks)
-   Minimum code to satisfy Test Matrix — no features beyond requirements (code tasks)
-   `verb/test` passes with zero failures and zero warnings (code tasks)
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments (code tasks)
-   `docs/invariant/004_script_as_data_purity.md` written with a real, non-empty Enforcement Mechanism section pointing at the actual shipped function (never TBD — the code must exist first)
-   `docs/definition/readme.md` and `docs/feature/001_rhai_scene_scripting.md` updated to register the new instance
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution` (all non-admin tasks)
-   Task state updated to ✅ on verification pass; file moved to `task/completed/` (final)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Script is a pure object-map/array literal document (structure mirrors `orrery_webgpu`'s own `scene.rhai`) | Whole-AST purity check | Returns `Ok(())` |
| T02 | Script's top level is declarative, but one object-map value nested inside it is an arithmetic expression (`#{ x: 1 + 2 }`) | Whole-AST purity check | Returns `Err` naming the offending operator call, not just top-level shape |
| T03 | Script contains a named function call nested inside an array element (not at top level) | Whole-AST purity check | Returns `Err` naming the offending call |
| T04 | Script contains a method call (`.foo()`) nested inside an object-map value | Whole-AST purity check | Returns `Err` naming the offending call |
| T05 | Script's top level is just a `main()` call (passing `top_level_lint`'s own shape check), but `main`'s body contains a call two blocks deep (e.g. inside an `if` branch inside a `for` loop) | Whole-AST purity check | Returns `Err` naming the offending call — proves recursion actually descends into control-flow bodies, not just literal containers |
| T06 | Existing `top_level_lint::check_top_level_is_declarative` test suite | Regression | All existing tests remain green — new purity check is additive, not a replacement |

## Acceptance Criteria

-   Whole-AST purity-check function exists, is public, and is exercised by T01-T05
-   `top_level_lint.rs`'s existing behavior and tests are unmodified and green (T06)
-   `docs/invariant/004_script_as_data_purity.md` exists with non-empty, non-TBD content in every required section
-   `docs/definition/readme.md` instance count matches actual file count in `invariant/`
-   Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Purity check (src/)**
- [ ] C1 — Does the new purity-check function recurse into `let` initializers, array elements, and object-map values, not just top-level statements?
- [ ] C2 — Does it reject operator calls (not only named calls) as impure?
- [ ] C3 — Does `top_level_lint.rs`'s existing test suite remain green and unmodified, confirming this task's change is additive only (T06)?

**Documentation**
- [ ] C4 — Does `docs/invariant/004_script_as_data_purity.md` name a real enforcement mechanism (the shipped function, not a TBD)?
- [ ] C5 — Does `docs/definition/readme.md`'s registered `invariant/` instance count accurately reflect the new file (3 → 4)?

**Out of Scope confirmation**
- [ ] C6 — Is `examples/orrery/webgpu/src/scene.rs` unmodified?
- [ ] C7 — Does `scene_script`'s public surface gain no new load-and-deserialize helper (only the purity-check function)?
- [ ] C8 — Is there no new script-facing color type or Squad easing-curve support added?
- [ ] C9 — Does this task's diff avoid touching or wiring any consumer in `codename_space_sandbox` (a separate repository — task 007 there is unaffected either direction)?
- [ ] C10 — Does the purity check limit itself to `FnCall`/`MethodCall` detection only, with no added non-call side-channel detection (e.g., a `const` referencing an engine-registered constant)?

### Measurements

- [ ] M1 — Test count: `cargo nextest run -p scene_script 2>&1 | tail -5` → T01-T06 all present and passing (was: 0 purity tests before this task)
- [ ] M2 — Doc instance count: `docs/definition/readme.md`'s invariant row count matches `ls docs/invariant/` count exactly

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p scene_script` → 0 warnings

### Anti-faking checks

- [ ] AF1 — T02-T05 actually exercise nested (non-top-level) impurity, not a restatement of `top_level_lint`'s existing top-level-only cases: `grep -n "fn.*nested\|fn.*t0[2345]" tests/*.rs` → each test's script fixture has the call embedded inside a `let`/array/object-map/control-flow body, confirmed by reading the fixture, not just the test name
- [ ] AF2 — No load-and-deserialize helper was added under a different name as a disguised re-inclusion of the cut Out-of-Scope item: `git diff --stat` reviewed for any new function calling `rhai::serde::from_dynamic`

## History

- **[2026-08-13]** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/. Goal: give `scene_script` a whole-AST purity check enforcing the script-as-data invariant `docs/pattern/004` documents but cannot currently check.
- **[2026-08-13]** `NOTE` — Filing location corrected: originally left at `task/107_....md` (task-dir root) after VERIFY passed; `tsk.rulebook.md`'s state-directory table specifies `task/verified/` for 🎯 Verified, not root. Moved to `task/verified/107_harden_scene_script_for_second_consumer.md`, no content change.
- **[2026-08-13]** `NOTE` — Step 7 (Task Quality Gate, TA122) compliance fix: added missing `repo_identity: self` to Execution State; added Checklist items C3 (top_level_lint regression), C5 (doc-count accuracy), C8-C10 (3 previously-unconfirmed Out of Scope bullets — color/easing, cross-repo non-wiring, non-call side channels). No change to Goal, Scope, or the Verification Record's gate verdicts.

## Related Documentation

- `module/helper/scene_script/docs/pattern/004_script_as_data.md` (workspace-root path: `docs/pattern/004_script_as_data.md`) — names the purity invariant this task enforces in code for the first time
- `module/helper/scene_script/docs/pattern/005_script_as_glue.md` (workspace-root path: `docs/pattern/005_script_as_glue.md`) — contrasting form, confirms purity is specific to script-as-data
- `module/helper/scene_script/docs/pitfall/002_checker_is_structural_not_semantic.md` — documents the exact gap this task closes (top-level-only, not whole-AST)
- `module/helper/scene_script/docs/feature/001_rhai_scene_scripting.md` — navigational hub, updated by this task
- `module/helper/scene_script/docs/invariant/readme.md` — collection this task adds instance 004 to
- `module/helper/scene_script/docs/definition/readme.md` — Module Index updated by this task
- `/home/user1/pro/lib/yrd_gamedev/cgtools/rulebook.md` — updated 2026-08-13 (this session, prior to this task's filing) to register the `animation` crate's layer placement; unrelated to this task's scope but touched in the same `doc_tsk` pass

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | 🟡 | 🟢 | Loader-helper deliverable had no concrete second caller — task 007 uses the inline pattern directly, not a shared helper — premature generalization | Cut loader helper from In/Out of Scope, Delivery Requirements, Test Matrix, Acceptance Criteria, Checklist entirely; kept only the purity check, which has standalone justification |
| D4 | Implementation Readiness | 🟡 | 🟢 | Recursion description named `let`/array/object-map containers but not control-flow bodies — a call hiding inside one would be missed | Broadened recursion wording to "every expression position reachable," explicitly including nested block/if/loop/try-catch bodies; added T05 |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 2 issues, both fixed | 2/2 |
