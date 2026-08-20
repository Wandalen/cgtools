# Fix renderer panic that violates its own Result-returning signature

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Fix a site in `renderer` where a function whose own signature returns `Result<_, _>` panics instead of
returning `Err` on a failure condition its signature already advertises as handleable (P1 — soundness
bucket, Fix-in-place). **Carried forward from the audit triage plan — exact file/line is not re-verified
in this filing pass; re-confirm against current `module/helper/renderer/src/` before touching.** Distinct
from task 020 (renderer's Composer/raw.rs dead-code and Quick Start doc drift, a hygiene concern in the
same crate) — keep separate even though both live in `renderer`.

## In Scope

- `module/helper/renderer/src/webgl/geometry.rs`: `Geometry::add_attribute` (the confirmed panic site,
  previously at line 112) — convert the duplicate-attribute-name panic branch to return `Err`; update
  its doc comment to match
- `module/helper/renderer/tests/geometry_tests.rs` (new file): a `wasm_bindgen_test(async)` test
  exercising the duplicate-name failure path, matching the crate's established GL-dependent test
  pattern (`tests/skeleton_tests.rs` precedent)
- `module/helper/renderer/tests/readme.md`: register the new test file's responsibility row

## Out of Scope

- Task 020 (`task/draft/020_renderer_dead_code_and_quickstart_doc.md` — renderer's Composer/raw.rs
  dead-code and Quick Start doc drift) — a hygiene concern in the same crate, tracked separately
- `Node::set_world_matrix` and `Node::local_bounding_box_hierarchical` (`src/webgl/node.rs`) — both
  call `.inverse().unwrap()` on a matrix that can be singular, a real but structurally different
  concern: their own signatures return `()` / `BoundingBox`, not `Result`, so they don't match this
  task's "function whose own signature returns `Result`" bug pattern
- `WideOutlinePass::new`'s `create_framebuffer(...).unwrap()` calls
  (`src/webgl/post_processing/outline/wide_outline.rs`) — a genuine, structurally similar concern in a
  different function; this task's Goal names a single site
- The systemic `locations.get(name).unwrap()` idiom, confirmed recurring inside `Result`-returning
  functions in at least `material/pbr.rs`, `post_processing/color_grading.rs`,
  `post_processing/gbuffer.rs`, `post_processing/shadow_to_color.rs`,
  `post_processing/unreal_bloom.rs`, `post_processing/outline/{wide,narrow,normal_depth}_outline.rs`
  (list not necessarily exhaustive for this specific idiom) — a widespread, deliberate codebase
  convention (uniform-location lookup assumed present after successful shader linking), not the
  specific bug pattern fixed here
- `renderer.rs`'s IBL `.expect(...)` calls — deliberate, documented invariant assertions referencing
  project docs, not an oversight
- wasm32-target *runtime* execution of the new test — blocked by this sandbox's pre-existing absence of
  `wasm-bindgen-test-runner` tooling; see `## History` for the full environmental-constraint account

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   No mocking — real `WebGl2RenderingContext` via `wasm_bindgen_test`, matching established crate
    precedent (`tests/skeleton_tests.rs`); no gl-free candidate matching this task's bug pattern exists
    anywhere in `src/` (confirmed by exhaustive search — see `## History`)
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `cargo nextest run -p renderer --all-features` (native) passes with zero failures, zero new warnings
-   `cargo clippy -p renderer --all-targets --all-features -- -D warnings` (native) clean
-   No duplication introduced; public items keep `///` doc comments accurate to new behavior
-   All Rust code uses 2-space indentation, no `cargo fmt`

## Work Procedure

1. Search `module/helper/renderer/src/` for a function whose own signature returns
   `Result< _, gl::WebglError >` (or similar) but whose body panics via `panic!`/`.unwrap()`/
   `.expect()` on a condition distinct from genuine `?`-propagated errors.
2. Confirm `Geometry::add_attribute` (`src/webgl/geometry.rs:95-116`) as the match: the
   duplicate-attribute-name branch called `panic!(...)` despite the function's own
   `-> Result< (), gl::WebglError >` signature already using `?` for legitimate error propagation two
   lines earlier (`info.upload( gl )?`).
3. Write `tests/geometry_tests.rs` — a `wasm_bindgen_test(async)` test (matching the
   `tests/skeleton_tests.rs` precedent, since `Geometry`/`AttributeInfo` require a live
   `WebGl2RenderingContext` and cannot be constructed in a plain native `#[test]`) that adds an
   attribute under a name, then adds a second attribute under the same name, asserting the second call
   returns `Err` rather than panicking.
4. Change the duplicate-name branch in `add_attribute` to
   `return Err( gl::WebglError::Other( "An attribute with this name already exists" ) )`; update the
   function's doc comment from "It panics if..." to "Returns `Err` if...".
5. Register `tests/geometry_tests.rs` in `tests/readme.md`'s Responsibility Table.
6. Run `cargo nextest run -p renderer --all-features` (native) to confirm no regression in the crate's
   natively-runnable suite.
7. Run `cargo clippy -p renderer --all-targets --all-features -- -D warnings` (native) to confirm clean.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|---|
| T01 | `add_attribute(gl, "positions", info)` called once on a fresh `Geometry` | `Geometry::add_attribute`, first call | Returns `Ok(())`; attribute stored |
| T02 | `add_attribute(gl, "positions", info)` called twice with the same name | `Geometry::add_attribute`, duplicate name | Returns `Err(_)` — does NOT panic |

## Acceptance Criteria

- `Geometry::add_attribute` returns `Err` instead of panicking when called with a duplicate attribute
  name
- `add_attribute`'s doc comment accurately describes the `Err` behavior (no longer claims "It panics")
- Every Test Matrix row has a corresponding test in `tests/geometry_tests.rs`
- `cargo nextest run -p renderer --all-features` passes with zero failures, zero new warnings (native
  suite — unaffected by the new wasm32-gated test, same as all 8 pre-existing `wasm_bindgen_test` files
  in this crate)
- `cargo clippy -p renderer --all-targets --all-features -- -D warnings` clean
- `tests/readme.md` Responsibility Table includes a row for `geometry_tests.rs`

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

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
| B1 | Rulebook Compliance | — | 🟢 | — | — |
| B2 | Test-First Requirement | — | 🟢 | — | — |
| B3 | Evidence of Failure | — | 🟢 | Pre-fix panic verified by direct diff read, not execution — wasm-bindgen-test-runner absent from sandbox, a structural gap affecting all 8 pre-existing `wasm_bindgen_test` files in this crate identically, not specific to this fix | — |
| B4 | Proper Fix Only | — | 🟢 | — | — |
| B5 | Fix Verification | — | 🟢 | — | Independently re-ran `will .test l::3` from `module/helper/renderer/` (package-scoped, via `longrun`) → exit 0, "4/4 commands passed, 0 failed"; corroborates the implementing subagent's own report with fresh, self-executed evidence |
| B6 | Knowledge Preservation | 🟡 | 🟢 | `tests/geometry_tests.rs`'s doc comment was one flowing paragraph, not the mandated 5-section format | Rewrote as `## Root Cause` / `Why Not Caught` / `Fix Applied` / `Prevention` / `Pitfall`, matching the format already established by this workspace's own prior bug-fix tests (e.g. `mdmath_core`'s `tuple2_test.rs`) |
| B7 | Code Cleanliness | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 | 1/1 |

**Aggregate verdict:** PASS — one Blocking Finding (B6) surfaced by the adversarial pass, fixed in place via a self-contained Fix-and-Recheck Loop, and re-verified by direct re-read; all other 14 dimensions clean on both the confirming and adversarial pass. D1–D8 use `tsk` skill's Readiness dimensions; B1–B7 use the Bug-Fixing Task Quality Requirements (this task fixes a P1 soundness panic, so both apply). Verification independently re-executed rather than solely trusted from the implementing subagent's report, per this session's Stale Evidence Trust discipline.

## Verification

### Checklist

- [x] C1 — Does `Geometry::add_attribute`'s duplicate-attribute-name branch return `Err` instead of `panic!`ing? `src/webgl/geometry.rs:124` → `return Err( gl::WebglError::Other( "An attribute with this name already exists" ) );`, preceded by a `Fix(task 013)` / `Root cause` / `Pitfall` comment (lines 111-118). No `panic!` remains inside `add_attribute`'s body.
- [x] C2 — Does the function's doc comment now say "Returns `Err`" instead of "It panics"? Confirmed by direct read: "It binds the VAO, uploads the attribute, and stores the `AttributeInfo`. Returns `Err` if an attribute with the same name already exists." — no "It panics" text remains anywhere in the doc comment.
- [x] C3 — Does `tests/geometry_tests.rs` exist with a `wasm_bindgen_test(async)` test exercising the duplicate-name failure path, using the workspace's 5-section bug-fix doc-comment format? Confirmed present: `add_attribute_duplicate_name_returns_err_not_panic`, `#[ wasm_bindgen_test( async ) ]`, with `## Root Cause`/`## Why Not Caught`/`## Fix Applied`/`## Prevention`/`## Pitfall` doc comment (lines 37-57) — matches the B6 Fix-and-Recheck Loop finding recorded above in `## Verification Record`.
- [x] C4 — Is `tests/geometry_tests.rs` registered in `tests/readme.md`'s Responsibility Table? `grep -n geometry_tests tests/readme.md` → `10:| geometry_tests.rs | Tests \`Geometry\` attribute API (add_attribute duplicate handling) |`.

### Measurements

- [x] M1 — `panic!` calls in `Geometry::add_attribute`'s duplicate-name branch: `0` (was: `1` — `panic!( "An attribute {} already exists", name );`, cite `git show 8c912a5e:module/helper/renderer/src/webgl/geometry.rs` line 114, the commit that introduced this code before the fix).

### Invariants

- [x] I1 — Native test suite (package-scoped, `longrun`-detached): `cargo nextest run -p renderer --all-features` → exit 0, `79 tests run: 79 passed, 0 skipped` (the new wasm32-gated geometry test is invisible here by design, per this task's own documented environmental constraint).
- [x] I2 — Compiler/lints: `cargo clippy -p renderer --all-targets --all-features -- -D warnings` → exit 101, **fails**, but not on this task's code: root-caused to `module/helper/browser_log/src/panic.rs:82`'s `#[ allow( clippy::exhaustive_structs ) ]` missing a `reason = "..."` clause, which violates the workspace's `allow_attributes_without_reason = "warn"` lint (root `Cargo.toml:117`) once escalated by `-D warnings`. Introduced by commit `5f33be66` ("feat: consolidate test infrastructure and refactor module architecture", 2026-08-11) — after this task's 2026-08-10 completion, and unrelated to `renderer` or this fix. Isolated with `cargo clippy -p renderer --all-targets --all-features --no-deps -- -D warnings` → exit 0, clean — confirming `renderer`'s own code, including this task's `geometry.rs` change, remains fully clippy-clean; `browser_log` is only swept in because `cargo clippy` (without `--no-deps`) lints every local path-dependency workspace member, not because `add_attribute` itself regressed.

### Anti-faking checks

- [x] AF1 — Guards against the panic branch silently reappearing: re-running `grep -n "panic!" src/webgl/geometry.rs` inside `add_attribute` must stay at `0` — any hit signals a reversion to the pre-fix behavior this task fixed.
- [x] AF2 — Guards against `tests/geometry_tests.rs`'s assertion being silently weakened: since this test is wasm32-gated and cannot be executed in this sandbox (see I1), a weakened assertion (e.g. `result.is_err()` → `is_ok()`, or dropping the duplicate `add_attribute` call) would be invisible to every native verification command in this repo, including I1's `79/79` count. The only re-check is a direct source read confirming the test still calls `add_attribute` twice with the same name and asserts `result.is_err()` on the second call.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P1 (soundness bugs)
  tier, Fix-in-place bucket.
- **[2026-08-10]** `INVESTIGATED_AND_FIXED` — Exhaustively searched `module/helper/renderer/src/` for a
  function whose own `-> Result< _, gl::WebglError >` signature panics instead of returning `Err` on a
  handleable condition: every `Result`-returning function in the crate was cross-referenced against
  every `panic!`/`.unwrap()`/`.expect()` occurrence. Confirmed `Geometry::add_attribute`
  (`src/webgl/geometry.rs:95-116`, panic previously at line 112) as the exact match:
  `panic!( "An attribute {} already exists", name )` fired on a duplicate attribute name — an ordinary,
  externally-reachable condition (e.g. malformed glTF re-declaring the same accessor semantic twice) —
  despite the function's own body already using `?` for legitimate error propagation two lines earlier
  (`info.upload( gl )?`). Two structurally similar but out-of-scope panics were found and deliberately
  left untouched because their own signatures return `()` / `BoundingBox`, not `Result` (see
  `## Out of Scope`): `Node::set_world_matrix` and `Node::local_bounding_box_hierarchical`
  (`src/webgl/node.rs`, both `.inverse().unwrap()` on a possibly-singular matrix).
  **Fix:** the duplicate-name branch now returns
  `Err( gl::WebglError::Other( "An attribute with this name already exists" ) )`; doc comment updated
  from "It panics if..." to "Returns `Err` if...".
  **Test:** added `tests/geometry_tests.rs`
  (`add_attribute_duplicate_name_returns_err_not_panic`, `wasm_bindgen_test(async)`) — the crate's
  established GL-dependent test style, matching `tests/skeleton_tests.rs` precedent, since
  `Geometry::new`/`AttributeInfo` require a real `WebGl2RenderingContext` unavailable to a plain native
  `#[test]` (confirmed by the exhaustive search above: zero gl-free candidates matching this task's bug
  pattern exist anywhere in `src/`). RED→GREEN was verified by code inspection rather than execution:
  before the fix the duplicate branch panicked unconditionally (deterministic abort within the
  `wasm_bindgen_test` harness); after the fix it deterministically returns `Err`, both directly readable
  from the diff.
  **Environmental constraint (full transparency):** this test's runtime pass/fail could not be executed
  in this sandbox. `wasm-bindgen-test-runner` is not installed and no
  `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER` is configured. Independently,
  `cargo check --target wasm32-unknown-unknown -p renderer --tests` fails on a pre-existing `getrandom`
  wasm32-backend/dev-dependency gap, reproducing identically with zero optional features enabled — not
  caused by this change; adding `--all-features` additionally hits a pre-existing, unrelated type error
  in `gpu_hal` (`webgpu` feature chain, `builder.entry(raw_entry)` missing a `?`). Both gaps affect every
  one of the crate's 8 pre-existing `wasm_bindgen_test` files identically, not just the new one — a
  structural, workspace-level sandbox limitation, not specific to this fix.
  **What was genuinely executed and verified in this sandbox:**
  `cargo nextest run -p renderer --all-features` (native, via `longrun`) — `70 tests run: 70 passed, 0
  skipped`, no regression (the new test is invisible to this native command, same as all other
  `wasm_bindgen_test` files, since it is `#[cfg(target_arch = "wasm32")]`-gated).
  `cargo clippy -p renderer --all-targets --all-features -- -D warnings` (native, via `longrun`) — clean,
  0 warnings. `tests/readme.md` updated with a `geometry_tests.rs` row. State intentionally left at 📝
  Draft — Readiness Verification Gate not self-administered as part of this pass, per this task's own
  dispatch instructions.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`): directly re-read the `geometry.rs` diff and the new test file rather than
  relying solely on the dispatch pass's prose report; independently re-ran verification via
  `will .test l::3` from `module/helper/renderer/` (package-scoped, `longrun`-detached) — exit 0, "4/4
  commands passed, 0 failed". Adversarial pass caught one Blocking Finding: `geometry_tests.rs`'s doc
  comment was a single flowing paragraph instead of this workspace's mandated 5-section bug-fix test
  format; fixed in place (rewrote as `## Root Cause`/`Why Not Caught`/`Fix Applied`/`Prevention`/
  `Pitfall`) and re-verified by direct re-read — a self-contained Fix-and-Recheck Loop, not a second
  round. All 15 dimensions (8 Readiness + 7 Bug-Fixing Quality) PASS. State moved directly to ✅
  Completed (fix was already implemented and now independently verified — no separate 🎯 Verified
  holding state needed). Note: an unrelated, actively-running concurrent process independently worked
  the same P1 backlog this session (see sibling tasks 009/010/011 — filed and completed as
  `BUG-050`/`BUG-051`/`BUG-052` via the formal bug pipeline) but never touched this task's files; no
  collision occurred.
