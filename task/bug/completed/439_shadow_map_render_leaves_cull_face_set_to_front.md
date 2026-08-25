# BUG-439: `ShadowMap::render()` leaves global `CULL_FACE` mode set to `FRONT` after returning

- **Severity:** Medium (not a leak -- a global GL state leak that silently corrupts culling for
  any subsequent draw call that doesn't explicitly re-set `cull_face` itself, producing
  inside-out-looking geometry rather than a crash)
- **state:** Completed
- **Affects:** Any consumer that draws geometry immediately after a `ShadowMap::render()` call
  without going through `Renderer::render()`'s own per-material `material_face_properties_enable`
  (which always re-sets `cull_face` explicitly before every draw, and so was never actually
  affected by this bug in practice) -- i.e., any *direct* `ShadowMap::render` caller outside the
  full `Renderer` pipeline.
- **Component:** `module/helper/renderer` (`src/webgl/shadow.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Found in the same sweep as BUG-432 (also in `shadow.rs`); unrelated in
  mechanism -- BUG-432 is a leaked GPU object, this is a leaked global GL *state* value.

## Symptom

`ShadowMap::bind()` sets `cull_face(FRONT)` as a standard peter-panning mitigation for
depth-only passes. `ShadowMap::render()` already restored the framebuffer binding at its end,
but left `cull_face`'s mode untouched -- returning with `CULL_FACE` mode still `FRONT`, not
restored to the renderer-wide default (`BACK`).

## Impact

**Who is affected:** Any code drawing geometry immediately after a `ShadowMap::render()` call
without an intervening explicit `cull_face` reset. `Renderer::render()`'s own pipeline is not
affected in practice, since `material_face_properties_enable` always re-sets `cull_face`
explicitly before every draw regardless of incoming state -- but any other direct caller of
`ShadowMap::render` (tests, tooling, a future alternate render path) would silently inherit
`FRONT`-face culling.

**What breaks:** Geometry drawn with the wrong cull face renders inside-out -- back faces are
drawn and front faces are culled, the opposite of the intended silhouette.

**Magnitude:** One leaked GL state value (`CULL_FACE_MODE`) per `ShadowMap::render()` call, until
some later code explicitly resets it.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep as BUG-432 -- auditing
`shadow.rs`'s `bind`/`render` pair for symmetric enable/restore of every piece of GL state
`bind()` mutates. The framebuffer binding was correctly restored; `cull_face`'s mode was not.

## Minimum Reproducible Example

```rust
// module/helper/renderer/src/webgl/shadow.rs, mod tests (inline, wasm32-gated)
let gl = gl_init();
let mut shadow_map = ShadowMap::new( &gl, 512 ).unwrap();
let light = Light::from( spot_light_make() );
let scene = crate::webgl::Scene::new();
shadow_map.render( &gl, &light, &scene );
let mode = gl.get_parameter( gl::CULL_FACE_MODE ).unwrap().as_f64().unwrap() as u32;
// pre-fix: mode == gl::FRONT (left over from bind()); post-fix: mode == gl::BACK.
assert_eq!( mode, gl::BACK );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- shadow_map_render_restores_cull_face_to_back
```

## Root Cause

`bind()` sets `cull_face(FRONT)` (a standard peter-panning mitigation for depth-only passes);
`render()` already restored the framebuffer binding at its end but left this piece of state
untouched -- an asymmetric enable/restore pair.

## Why Not Caught

No test previously read back `CULL_FACE_MODE` after a `ShadowMap::render()` call -- existing
shadow tests (`fbo_pass_cycle_test.rs`) exercise the resulting shadow map's depth output, not
the GL state left behind afterward, and `Renderer::render()`'s own downstream re-set of
`cull_face` for every material draw masked the symptom in the one pipeline that was actually
exercised end-to-end.

## Fix Location

`module/helper/renderer/src/webgl/shadow.rs`, `ShadowMap::render`: added
`gl.cull_face(gl::BACK)` before returning, restoring `cull_face` to the renderer-wide default.
`CULL_FACE` enable/disable itself, and the viewport `bind()` sets, are deliberately left
unrestored -- see Pitfall.

## Prevention

New inline test `shadow_map_render_restores_cull_face_to_back` in `shadow.rs`'s
`#[cfg(all(test, target_arch = "wasm32"))] mod tests` block (alongside BUG-432's test; inline
because `render` is exercised through the same private-field-heavy construction path -- see
`rulebook.md § Test placement`). Reads back `gl::CULL_FACE_MODE` via `gl.get_parameter` after a
real `ShadowMap::render` call on an empty scene, asserting it is `gl::BACK` -- the general,
state-agnostic invariant the fix restores, not a pinned per-scene expectation. Construction
reuses the `spot_light_make()`/`Light::from`/`Scene::new()` pattern from `fbo_pass_cycle_test.rs`.

## Pitfall

`CULL_FACE` enable/disable is deliberately left as `bind()` set it (enabled) -- restoring face
*mode* to a sane default is enough to prevent silently-wrong culling; whether culling is enabled
at all is the next draw call's own responsibility, same as for every material-driven draw in
`Renderer::opaque_draw`. The viewport `bind()` sets (`resolution x resolution`) is deliberately
left unrestored too -- there is no single correct default to restore it to from this scope (the
real render target's size isn't known here); callers relying on a specific viewport must set it
themselves before their next draw, same as any other GL viewport consumer. GL state (as opposed
to GL objects/resources) has no "drop" mechanism at all -- a pass that mutates global context
state must explicitly restore whatever contract it promises callers, since nothing in the type
system enforces symmetric enable/restore the way `Drop` does for owned GPU objects.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Added `gl.cull_face(gl::BACK)` at the end of `ShadowMap::render`; added `Fix(BUG-439)`/`Root cause`/`Pitfall` source comment and inline reproducer test. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean; reuses `spot_light_make()`/`Light::from`/`Scene::new()` construction pattern already proven by `fbo_pass_cycle_test.rs`. Adversarial pass: confirmed by direct inspection that pre-fix `render()` had no `cull_face` reset of any kind after `bind()` set it to `FRONT` -- the `get_parameter(CULL_FACE_MODE)` readback would have returned `FRONT`, not `BACK`, against that code. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-439)`/`Root cause`/`Pitfall` 3-field source comment; 5-section test doc comment on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `shadow.rs`'s `ShadowMap::render` method plus its own inline test (co-located with BUG-432's test module). | — |

**Reproduced:** YES -- direct code inspection confirms pre-fix `ShadowMap::render` had no
`cull_face` reset after `bind()` set `FRONT`; the new test's `get_parameter(CULL_FACE_MODE)`
readback is the direct, deterministic check for exactly that state. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shadow.rs` | Added `gl.cull_face(gl::BACK)` to `ShadowMap::render` with `Fix(BUG-439)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/shadow.rs` | Added inline `mod tests::shadow_map_render_restores_cull_face_to_back` (wasm32-gated, co-located with BUG-432's reproducer). |
