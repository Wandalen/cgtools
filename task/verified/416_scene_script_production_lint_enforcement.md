# 416: scene_script — production-usable lint enforcement (not test-only)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/scene_script
- **verified_by:** self (doc_tsk Readiness Verification Gate, Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-20
- **blocked_by:** null

## MOST Goal

`docs/layer/006_l5_scene_script_and_runners.md`'s round-7 doc-sync fix (this session) corrected an
overstated claim and surfaced a real, previously-undocumented gap: `scene_script`'s two structural lints —
`check_top_level_is_declarative` (`top_level_lint.rs`, enforcing
[invariant/001](../../module/helper/scene_script/docs/invariant/001_top_level_bindings_convention.md)) and
`check_whole_ast_is_pure` (`purity_lint.rs`, enforcing
[invariant/004](../../module/helper/scene_script/docs/invariant/004_script_as_data_purity.md)) — are both
public, both real, both correctly implemented (confirmed working via `purity_lint_test.rs` and
`example_convention_test.rs`), but **neither is invoked by any production code path**. Every current
consumer (`examples/scene_script/pingpong_animation/src/lib.rs:44`,
`examples/scene_script/f32x2_vector_arithmetic/src/lib.rs:21`,
`examples/orrery/webgpu/src/scene.rs:246`) calls `scene_script::engine_build()` then compiles/evaluates the
script directly — the invariant docs' own words for the two lints ("the convention `scene_script` enforces")
overstate what actually happens at runtime: a violation is caught only if a maintainer remembers to write a
dedicated test for that specific script, never automatically at script-load time. This task closes that gap:
add a production-usable compile-and-lint entry point per script form (script-as-glue →
`check_top_level_is_declarative`; script-as-data → `check_whole_ast_is_pure`, per
[pattern/004](../../docs/pattern/004_script_as_data.md)/[pattern/005](../../docs/pattern/005_script_as_glue.md)'s
existing split) and switch all 3 existing consumers to it, so a lint violation in any of them is rejected at
load time, not merely catchable by a test someone remembers to write.
Testable: a deliberately impure/imperative-violating script fed through the new entry point returns `Err`
before any engine evaluation happens; the 3 existing consumers still evaluate their real scripts successfully
through the same entry point (`cargo nextest run -p scene_script -p pingpong_animation
-p f32x2_vector_arithmetic -p orrery_webgpu` — package names per each crate's own manifest).

## In Scope

- `module/helper/scene_script/src/` — new production-usable compile-and-lint function(s) (e.g. a new
  `script_load.rs` sibling module, or additions to `engine.rs`) that: (1) parses `source` against the
  provided `Engine`, (2) runs the lint appropriate to the declared script form, (3) returns a unified error
  type distinguishing parse failure from lint rejection. Exported via the crate's existing `mod_interface!`
  block in `lib.rs`.
- Updating the 3 existing consumer call sites (`pingpong_animation/src/lib.rs`,
  `f32x2_vector_arithmetic/src/lib.rs`, `orrery/webgpu/src/scene.rs`) to call the new entry point instead of
  raw `engine_build()` + direct compile/eval.
- A new test in `scene_script`'s own test suite proving a deliberately malformed script (violating each of
  the two lints in turn) is rejected via the new entry point specifically — distinct from the existing
  `purity_lint_test.rs`/`example_convention_test.rs`, which test the lint functions directly, not this new
  wiring layer.
- `docs/layer/006_l5_scene_script_and_runners.md`'s Sources row for `purity_lint.rs`/`top_level_lint.rs` —
  update once wired, since the current text ("neither lint is wired into a production loader") becomes stale
  the moment this task lands.

## Out of Scope

- Any change to the lints' own detection logic (`check_top_level_is_declarative`,
  `check_whole_ast_is_pure`) — both are already correct and independently tested; this task only adds a
  call site, never touches their internals.
- The tile-stack interactive-runner gap named in the same round-7 doc-sync fix
  (`docs/render_stack/002_tile.md`) — a separate, much larger, no-concrete-consumer-yet architectural gap,
  tracked instead as a draft watch-item (see Related Documentation).
- Any new script-form beyond the two already named by pattern/004/pattern/005 — no third form is in scope.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Zero change to the two lints' own internal detection logic
- Test Matrix populated, every row backed by a real passing test
- `verb/test`-equivalent scoped run passes with zero failures (scene_script + the 3 consumer example crates)
- No function exceeds 50 lines; public items have `///` doc comments
- `docs/layer/006_l5_scene_script_and_runners.md`'s Sources row updated to reflect the closed gap
- Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
- Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | A script-as-glue source violating `check_top_level_is_declarative` (imperative statement outside `main()`) | new glue-form entry point | Returns `Err` before engine evaluation; error identifies the lint violation, not a generic parse failure |
| T02 | A script-as-data source violating `check_whole_ast_is_pure` (contains a call expression) | new data-form entry point | Returns `Err` before engine evaluation; error identifies the impure call |
| T03 | `pingpong_animation.rhai`'s real, already-valid script | glue-form entry point, real consumer | Compiles and evaluates successfully — identical runtime behavior to today's direct `engine_build()` + compile path |
| T04 | orrery's real `scene.rhai` | data-form entry point, real consumer | Compiles and evaluates successfully — identical output to today's direct path |
| T05 | `f32x2_vector_arithmetic.rhai`'s real script | glue-form entry point, real consumer | Compiles and evaluates successfully — identical output to today's direct path |

## Acceptance Criteria

- New production entry point function(s) exist in `scene_script`, exported via `mod_interface!`
- All 3 existing example consumers call the new entry point, not raw `engine_build()` + direct compile
- A deliberately-invalid script is rejected at load time via the new entry point (T01/T02)
- All 3 existing consumers' real scripts still evaluate correctly through the new entry point (T03-T05) —
  zero behavioral regression
- `docs/layer/006_l5_scene_script_and_runners.md` no longer states the lints are unwired
- No function in the changed/new code exceeds 50 lines

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-19]** `CREATED` — Filed via `/doc_tsk` after round-7's docs/layer gap audit revealed
  `scene_script`'s two structural lints are real and correctly implemented but invoked only by tests, never
  by any production code path — closing the gap between the invariant docs' claimed enforcement and actual
  runtime behavior.

## Related Documentation

- `docs/layer/006_l5_scene_script_and_runners.md` — the layer doc whose round-7 fix surfaced this gap
  (Sources row for `purity_lint.rs`)
- `module/helper/scene_script/docs/invariant/001_top_level_bindings_convention.md` — the invariant
  `check_top_level_is_declarative` enforces
- `module/helper/scene_script/docs/invariant/004_script_as_data_purity.md` — the invariant
  `check_whole_ast_is_pure` enforces
- `docs/pattern/004_script_as_data.md` / `docs/pattern/005_script_as_glue.md` — the two script-form
  patterns this task's two entry-point variants correspond to
- `task/completed/107_harden_scene_script_for_second_consumer.md` — the original task that added
  `purity_lint.rs`; this task is the natural follow-up closing its production-wiring gap
- `task/verifying/386_register_scene_script_top_level_lint_dot_chain_lhs_call_detection_fix_closes_bug351.md` —
  a related but distinct task (fixes the lint's own detection logic, not its production wiring)

## Verification Record

- **Gate:** doc_tsk Readiness Verification Gate
- **Tier:** 2 (Dual-Role Self-Check — self-administered, no subagent dispatch, per
  `tsk.rulebook.md § Task File : Readiness Verification Gate`'s Verification Delegation prohibition)
- **Date:** 2026-08-20
- **Verifier:** self (authoring agent, dual-role: Pass 1 Confirming + Pass 2 Adversarial)

**Pass 1 (Confirming) summary:** Scope Coherence — In Scope's 4 items (entry-point function(s), 3
consumer call-site swaps, new load-time-rejection test, docs/layer/006 Sources row) serve exactly one
goal; Out of Scope fences off 3 concrete creep vectors. MOST Goal Quality — cites real evidence (exact
function names, exact consumer file paths/lines, the exact doc claim being corrected) and ends with a
concrete, named test command. Value/YAGNI — closes a real, present mismatch between the invariant docs'
claimed enforcement and actual runtime behavior across 3 already-shipped consumers, not a hypothetical
future need. Implementation Readiness — every prerequisite fact (existing function signatures, existing
call sites, existing test patterns) was confirmed present via direct grep this session. Execution Scope —
sized as one indivisible unit (entry point + all 3 dependent call-site swaps + proof test); splitting
would create artificial inter-task ordering dependencies with no independently shippable intermediate
state. Crate Scope Unity — every touched crate (scene_script + its 3 existing consumers) already sits in
scene_script's own consumer graph. Crate Locality — new logic lives entirely in scene_script; the 3
consumer edits are drop-in call swaps only (T03-T05 assert identical output). Crate Single Responsibility
— the new entry point is a narrow extension of scene_script's existing engine-assembly/lint charter, not
a new unrelated concern.

**Pass 2 (Adversarial) summary:** Attempted to find scope incoherence in the docs/layer/006 Sources-row
delivery item — ruled out: the row currently describes the *actual current unwired state* correctly (not
stale), and only becomes stale once this task's own code change lands, so listing its follow-up edit as a
delivery item is intrinsic to this task's own unit of work, not deferred pre-existing doc debt. Attempted
to falsify MOST Goal Quality by executing the stated Testable command against the actual In-Scope
consumer list — found the illustrative `cargo nextest` example omitted the third consumer's package
(`orrery_webgpu`, confirmed via `Cargo.toml`); classified Non-Blocking since the authoritative Delivery
Requirements text ("scene_script + the 3 consumer example crates") was already complete and the Test
Matrix (T03-T05) already covered all 3 — fixed the illustrative command in place rather than carrying the
inconsistency forward. Attempted to falsify Value/YAGNI by arguing the 3 known-valid scripts don't
currently violate their lints so wiring is "belt-and-suspenders" — rejected: the gap is that a *future*
edit to any of the 3 scripts, or a 4th consumer, can silently ship a violation today with zero enforcement
until a human remembers to write a bespoke test, which is a real present process gap, not speculative
hardening. Attempted to falsify Crate Scope Unity by treating the 3 consumer-crate touches as scope creep
beyond the single `unit:` crate — rejected: wiring into every real consumer is the literal content of the
MOST Goal (a version touching only scene_script would not close the claimed gap), and the touches
themselves are mechanical swaps, not new functionality. No Blocking Finding survived either pass.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | Illustrative test command omitted 3rd consumer's package (`orrery_webgpu`) — non-blocking; authoritative Delivery Requirements text was already complete | Added `-p orrery_webgpu` to the example command |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 non-blocking | 1 fix |

All 8 dimensions PASS in the same check → task promoted to 🎯 (Verified), moved to `task/verified/`.
