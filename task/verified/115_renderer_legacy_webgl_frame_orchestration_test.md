# Legacy webgl-path frame-orchestration attachment-selection test coverage

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Give the legacy `renderer::webgl::Renderer::render()`'s drawbuffers
attachment-selection logic (the `has_transparent`/`has_emissive` 4-way
branch selecting which of `[0]` / `[0,1]` / `[0,2,3]` / `[0,1,2,3]` color
attachments to enable per frame) natively-testable, zero-GL-context unit
coverage — the one piece of this method's frame-shape logic that is a pure
function of two booleans, extractable without touching any
`WebGl2RenderingContext` call, unlike the canonical webgpu path which
already gets end-to-end coverage via `opaque_path_renders_lit_quad`. Matters
now because `docs/layer/003_l2_frame_orchestration.md`'s Embedded Instances
Today section documents this exact ordering/attachment logic as fact with
zero test citation, unlike its sibling canonical-path bullet which cites a
real passing test. Bounded to one pure-function extraction plus one new
native test file in this one crate. Testable: `cargo test -p renderer --test
webgl_frame_orchestration_test` exits 0 with all 4 branch cases passing.

## In Scope

- `module/helper/renderer/src/webgl/renderer.rs`: extract the
  attachment-selection branch — currently inline in `Renderer::render()`,
  the `if has_transparent && has_emissive {...} else if has_transparent
  {...} else if has_emissive {...} else {...}` block that feeds
  `gl::drawbuffers::drawbuffers` — into a pure function, e.g. `fn
  frame_attachments( has_transparent : bool, has_emissive : bool ) -> &'static
  [ u32 ]`, called from `render()` in place of the inline branch, immediately
  followed by the same `gl::drawbuffers::drawbuffers( gl, ... )` call
  `render()` already makes.
- New `module/helper/renderer/tests/webgl_frame_orchestration_test.rs`
  (native, no GL context, no feature gate beyond the crate's default), 4
  cases — one per boolean combination — pinning the exact attachment array
  each combination returns.

## Out of Scope

- Pixel-level / live-`WebGl2RenderingContext` rendering verification for the
  legacy path — this workspace has no native/offscreen WebGL2 provider
  (confirmed absent: no swiftshader/osmesa/surfman/glutin dependency
  anywhere in `minwebgl`/`mingl`/`renderer`'s `Cargo.toml`), so unlike the
  canonical webgpu path's native-wgpu-plus-lavapipe route, there is no way
  to construct a real `WebGl2RenderingContext` outside an actual browser.
  Closing that is a workspace-wide browser-test-infrastructure decision
  (already flagged as an accepted gap elsewhere — see
  `tilemap_renderer/tests/webgpu_backend_test.rs`'s own doc comment), not
  achievable inside this leaf-crate task.
- Any other part of `render()`'s GL-calling sequence (`nodes_collect`,
  `opaque_draw`, `transparent_draw`, `composite`) — these are not pure
  functions (every step calls `gl` directly) and extracting them is a much
  larger refactor than this task's bounded scope.
- The canonical `src/webgpu/` path — already covered by
  `native_render_test.rs`.
- Migrating the legacy path onto `gpu_hal` (tracked separately per
  `docs/layer/004_l3_stack_engine.md`'s strangulation note).

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Extraction lands with zero behavior change — `render()`'s call sequence
    still invokes `gl::drawbuffers::drawbuffers` with the same arrays it did
    before, just via the new function
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its
    implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `verb/test` passes with zero failures and zero warnings
-   No function exceeds 50 lines; no duplication; public items have `///`
    doc comments
-   Independent verification passes per `§ Acceptance Verification :
    Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to
    `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | has_transparent=false, has_emissive=false | `frame_attachments` | returns `[0]` |
| T02 | has_transparent=false, has_emissive=true | `frame_attachments` | returns `[0,1]` |
| T03 | has_transparent=true, has_emissive=false | `frame_attachments` | returns `[0,2,3]` |
| T04 | has_transparent=true, has_emissive=true | `frame_attachments` | returns `[0,1,2,3]` |

## Acceptance Criteria

-   A pure attachment-selection function exists in `src/webgl/renderer.rs`
    (no `&self`, no `gl` parameter)
-   `render()` calls it in place of the previous inline branch, passing the
    same two booleans it already computes
-   `tests/webgl_frame_orchestration_test.rs` exists with all 4 Test Matrix
    cases passing
-   No pre-existing test in `renderer`'s suite regresses
-   Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Extraction**
- [ ] C1 — Does a pure attachment-selection function exist, taking exactly `( bool, bool )` and returning the attachment index list, with no `&self`/`gl` parameter?
- [ ] C2 — Does `render()`'s body call it instead of the previous inline 4-arm branch, immediately followed by the same `gl::drawbuffers::drawbuffers( gl, ... )` call it already had?

**Tests**
- [ ] C3 — Does `tests/webgl_frame_orchestration_test.rs` exist with 4 tests, one per Test Matrix row?

**Out of Scope confirmation**
- [ ] C4 — Is the new test file free of any `WebGl2RenderingContext`/`gl::` construction call?
- [ ] C5 — Do `nodes_collect`/`opaque_draw`/`transparent_draw`/`composite` remain unmodified (`git diff` shows no edits to those functions' bodies beyond the one branch touched)?
- [ ] C6 — Does `src/webgpu/` remain unmodified (`git diff` shows no edits under `src/webgpu/`)?
- [ ] C7 — Does the legacy `src/webgl/` path remain on its existing direct-GL call surface, with no new `gpu_hal` dependency introduced?

### Measurements

- [ ] M1 — new test count: `cargo test -p renderer --test webgl_frame_orchestration_test 2>&1 | grep -c "test result: ok"` → 1 (was: file did not exist)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p renderer --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — the 4 test cases assert 4 genuinely different return values, not the same array asserted 4 times: `grep -c "assert_eq" tests/webgl_frame_orchestration_test.rs` → ≥4, and the 4 expected arrays are pairwise distinct (checked by reading the file, not merely counted)

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

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

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-15]** `FILED` — Task filed via `/doc_tsk` Phase 2 (docs/layer gap audit): add legacy webgl-path frame-orchestration attachment-selection test coverage to `renderer`.

## Related Documentation

- `docs/layer/003_l2_frame_orchestration.md` — Embedded Instances Today section's legacy-path bullet this task backs with tests
- `docs/layer/004_l3_stack_engine.md` — `renderer`'s L3 engine table entry (legacy vs canonical downward seam)
- `module/helper/renderer/tests/native_render_test.rs` — the canonical-path precedent this task partially mirrors, at the orchestration level rather than the pixel level
