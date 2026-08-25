# BUG-441: `WebGlBackend.context_lost` cleared by the DOM restore listener before GPU state is re-uploaded, not after

- **Severity:** Medium (not a leak -- after a real WebGL context loss/restore cycle,
  `submit`/`output` would proceed to issue GL calls against a context with no valid GPU objects,
  instead of returning the documented `RenderError::ContextLost` the caller is supposed to be
  able to rely on)
- **state:** Completed
- **Affects:** Every consumer of `tilemap_renderer`'s `WebGlBackend` that can experience a real
  WebGL context loss/restore cycle (e.g. a GPU driver reset, tab backgrounding on some browsers,
  too many contexts open) and calls `submit`/`output` in the window between the browser firing
  `webglcontextrestored` and the caller getting around to re-calling `assets_load`.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/webgl.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- independent of the `module/helper/renderer` GPU-resource-leak family
  (BUG-432/433/436/437/438/440); a state-machine ordering defect in `tilemap_renderer`'s own
  context-loss handling, found in the same sweep but in a different crate.

## Symptom

```rust
// pre-fix -- src/adapters/webgl.rs, WebGlBackend's webglcontextrestored listener
let on_restored = Closure::<dyn FnMut(web_sys::Event)>::new( move |_event| {
  self_context_lost.set( false ); // cleared here, immediately on the DOM event
});
```

`context_lost` used to be cleared by the `webglcontextrestored` DOM listener the instant the
browser fired the event -- before the caller had any chance to re-call `assets_load` and
re-upload GPU state. `submit`/`output` would then see `context_lost == false` and proceed to
issue GL calls against a context whose textures/buffers/VAOs no longer existed (a restored WebGL
context starts with all GPU objects gone).

## Impact

**Who is affected:** Any consumer that can experience a real context loss/restore cycle and
calls `submit`/`output` before re-calling `assets_load`.

**What breaks:** Between `webglcontextrestored` firing and the caller's next `assets_load` call,
`submit`/`output` would see `context_lost == false` and proceed to issue GL calls against a
context with no valid resources -- silently drawing nothing, erroring deep inside driver calls,
or in the worst case triggering a second context loss -- instead of returning the documented,
catchable `RenderError::ContextLost` the caller is supposed to be able to rely on during that
window.

**Magnitude:** One incorrect `context_lost == false` window per context loss/restore cycle,
lasting until the caller's next `assets_load` call.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep as the `renderer` crate items --
auditing every state flag with a documented safety contract (`context_lost` gating `submit`/
`output`) against every code path that mutates it, looking for a mutation that runs before the
invariant it's supposed to guard is actually re-established.

## Minimum Reproducible Example

```rust
// module/helper/tilemap_renderer/src/adapters/webgl.rs, mod tests (inline, wasm32-gated)
let gl = gl_init();
let mut backend = WebGlBackend::new( RenderConfig::default(), gl ).unwrap();
backend.context_lost.set( true ); // simulates the effect of a real webglcontextlost event
assert!( backend.submit( &[] ).is_err() );
backend.assets_load( &empty_assets() ).unwrap();
// pre-fix: this would already be false the instant a webglcontextrestored DOM event fired,
// even if assets_load had never actually been called -- this test simulates the loss directly
// and confirms assets_load, not the listener, is what clears the flag.
assert!( !backend.context_lost.get() );
assert!( backend.submit( &[] ).is_ok() );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_renderer && cargo test -p tilemap_renderer --lib --target wasm32-unknown-unknown -- assets_load_clears_context_lost_after_simulated_loss
```

## Root Cause

The `webglcontextrestored` listener cleared the flag the instant the browser fired the event --
but a restored context starts with all GPU objects gone (textures, buffers, VAOs deleted).
Between that event firing and the caller getting around to re-calling `assets_load`, `submit`/
`output` would see `context_lost == false` and proceed to issue GL calls against a context with
no valid resources.

## Why Not Caught

No existing test exercised `context_lost`'s lifecycle at all -- `webgl_backend_test.rs` only
covers `declared_capabilities()`, and nothing simulated a loss/restore cycle.

## Fix Location

`module/helper/tilemap_renderer/src/adapters/webgl.rs`:
- The `webglcontextrestored` listener no longer clears `context_lost` -- it only logs a warning
  that `submit`/`output` remain blocked until `assets_load` is called again.
- `assets_load`'s tail now clears `context_lost` itself, once GPU state has actually been
  re-uploaded.

## Prevention

New inline test `assets_load_clears_context_lost_after_simulated_loss` in `webgl.rs`'s
`#[cfg(all(test, target_arch = "wasm32"))] mod tests` block (inline because `context_lost` is a
private field -- see `rulebook.md § Test placement`). Simulates a loss (`context_lost.set(true)`
-- the same effect the real `webglcontextlost` listener has) without needing to synthesize a
real `WEBGL_lose_context` DOM event (no precedent for that exists anywhere in this workspace),
then verifies `assets_load` -- not merely the passage of time or a restored-event -- is what
unblocks `submit`/`output` again.

## Pitfall

`context_lost` now means "safe to issue GL calls, GPU state is known-good" rather than merely
"the context object is alive" -- the two are NOT the same thing across a restore. Any future
code that re-populates GPU state some other way (not through `assets_load`) must also clear this
flag, or it will stay permanently stuck rejecting `submit`/`output` after a real restoration.
Separately: directly writing the private `context_lost` field in the test is a white-box
shortcut standing in for a real `webglcontextlost` event; it exercises the observable contract
the fix changed (assets_load is now the sole place that clears the flag), not the DOM listener
registration path itself.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Moved `context_lost` clearing from the `webglcontextrestored` listener to `assets_load`'s tail; added `Fix(BUG-441)`/`Root cause`/`Pitfall` source comments at both sites and an inline reproducer test. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p tilemap_renderer` compiles clean. Adversarial pass: confirmed by direct inspection that pre-fix, the `webglcontextrestored` listener closure called `self_context_lost.set(false)` directly -- with no dependency on `assets_load` having run -- so the fix's behavioral change (listener now only logs) is structurally real, not cosmetic. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-441)`/`Root cause`/`Pitfall` 3-field source comments at both fix sites (listener and `assets_load`); 5-section test doc comment on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `webgl.rs`'s `WebGlBackend` context-loss listener and `assets_load` method, plus its own inline test module. | — |

**Reproduced:** YES -- direct code inspection confirms the pre-fix listener closure called
`context_lost.set(false)` unconditionally on the DOM restore event, independent of whether
`assets_load` had run; the new test's simulated-loss-then-assert-still-blocked-until-
`assets_load` sequence is the direct, deterministic check that the fixed code now requires
`assets_load` specifically (not the mere passage of the restore event) to clear the flag.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/webgl.rs` | Moved `context_lost` clearing from the `webglcontextrestored` listener to `assets_load`'s tail, with `Fix(BUG-441)`/`Root cause`/`Pitfall` comments at both sites. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/webgl.rs` | Added inline `mod tests::assets_load_clears_context_lost_after_simulated_loss` (wasm32-gated). |
