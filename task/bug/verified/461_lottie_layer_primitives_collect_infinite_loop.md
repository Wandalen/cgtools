# BUG-461: `lottie_surface_rendering`'s `layer_primitives_collect` hangs forever on any non-`Shape` layer content

- **Severity:** High (hangs the render/load path entirely -- not a visual glitch, the browser tab
  never finishes loading for any Lottie animation containing a `None`/`Instance` content layer)
- **state:** Verified
- **Affects:** `examples/minwebgl/lottie_surface_rendering`'s animation loader, for any `.json`
  Lottie asset containing a layer whose `content` is `velato::model::Content::None` (e.g. a null
  or precomp-reference layer) or `Content::Instance` (an unexpanded asset reference) -- both are
  legitimate, spec-valid Lottie content that `layer_to_primitives` intentionally does not expand
  into primitives.
- **Component:** `examples/minwebgl/lottie_surface_rendering` (`src/animation.rs`,
  `layer_primitives_collect`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **Fix Task:** [505](../../verifying/505_register_lottie_surface_rendering_layer_primitives_collect_infinite_loop_fix_closes_bug461.md)
- **verification_date:** 2026-08-20

## Symptom

```rust
// pre-fix -- src/animation.rs, layer_primitives_collect
let mut i = 0;
while i < layers.len()
{
  let Some( layer_primitives ) = layer_to_primitives( i, layers, &mut repeaters )
  else
  {
    continue; // <- `i` never advances
  };
  // ... (only reached when layer_to_primitives returns Some)
}
```

`layer_to_primitives` returns `None` for `Content::None` and `Content::Instance` without ever
pushing anything onto `layers`. When that happens, the `else` branch above `continue`s straight
back to the `while` condition -- but nothing changed `i` or `layers.len()` in that branch, so the
loop condition (`i < layers.len()`) evaluates identically forever. Any Lottie asset with such a
layer hangs this function permanently.

## Impact

**Who is affected:** Any consumer loading a Lottie `.json` composition containing a `None`- or
`Instance`-content layer through `layer_primitives_collect` (reached via `animation_load`).

**What breaks:** The async load future never resolves -- the entire demo hangs on load with no
error, no timeout, no console output. Indistinguishable from a slow network fetch until the tab is
manually reloaded.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `examples/minwebgl/lottie_surface_rendering`, reading
`layer_primitives_collect`/`layer_to_primitives` together and noticing the `else` branch's
`continue` has no preceding index advance, while every other branch of the surrounding logic
reaches the loop's own trailing `i += 1`.

## Minimum Reproducible Example

```rust
// examples/minwebgl/lottie_surface_rendering/src/animation.rs, #[cfg(test)] mod tests
let mut layers = vec!
[
  velato::model::Layer::default(), // Content::None (its own Default)
  velato::model::Layer { content : Content::Instance { name : "asset_0".into(), time_remap : None }, ..Default::default() },
  velato::model::Layer { content : Content::Shape( vec![] ), ..Default::default() },
];
let ( primitives, _repeaters ) = layer_primitives_collect( &mut layers ); // pre-fix: never returns
```

**Verify Command:**
```bash
cd examples/minwebgl/lottie_surface_rendering && cargo nextest run layer_primitives_collect_skips_non_shape_content_without_hanging
```

## Root Cause

The `else` branch's `continue` skips the `while` loop body's own trailing `i += 1;` -- a manually
indexed `while` loop (unlike a `for` loop over an iterator) does not advance its index "for free"
on `continue`; every early-exit branch needs its own explicit index advance, and this one had none.

## Why Not Caught

No existing test exercised `layer_primitives_collect` with anything other than `Content::Shape`
layers (the only variant this crate's own sample assets apparently contain), so the infinite loop
had no code path exercising it. Manual testing in-browser would also not obviously reveal the
cause -- a hang looks identical to a slow network load until a debugger/profiler is attached.

## Fix Location

`examples/minwebgl/lottie_surface_rendering/src/animation.rs`, `layer_primitives_collect`: added
`i += 1;` immediately before the `else` branch's `continue;`.

## Prevention

Added `layer_primitives_collect_skips_non_shape_content_without_hanging` to a new
`#[ cfg( test ) ] mod tests` block in `animation.rs` (this crate is bin-only, no `lib` target, so
`tests/` integration tests cannot see this private function -- matches this workspace's own
`rulebook.md` test-placement rule for private-helper coverage). The test constructs a 3-layer
fixture (`None`, `Instance`, `Shape`) and calls `layer_primitives_collect` on a background thread,
bounded by `mpsc::Receiver::recv_timeout( 5s )` -- a regression back to the infinite loop fails the
test loudly within 5 seconds instead of hanging the whole test binary forever. The fixture and
result are plain `usize` counts sent across the channel (never the `!Send`
`PrimitiveData`/`Layer` values themselves, which hold an `Rc<RefCell<_>>` internally via
`primitive_generation::PrimitiveData`).

## Pitfall

A `let-else` `continue` inside a manually-indexed `while` loop skips the loop body's own trailing
index advance -- `continue` is not "skip to the next iteration" for free the way it is in a `for`
loop over an iterator; every early-exit branch in a manually-indexed loop needs to advance the
index itself before `continue`.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX sweep of `examples/minwebgl/lottie_surface_rendering`. |
| 2026-08-20 | fixed | Added the missing `i += 1;`; documented with `Fix(BUG-461)`/`Root cause`/`Pitfall`; added a genuine, timeout-bounded regression test. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted the `i += 1;` fix, ran the new test, confirmed it fails via the 5s `recv_timeout` panic path (not vacuously passing) -- genuinely catches the defect; restored the fix and confirmed the test passes. `cargo test -p lottie_surface_rendering` (native) -- clean; `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` -- clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-461)`/`Root cause`/`Pitfall` 3-field format applied at the fix site; test carries its own doc comment explaining root cause and the `!Send`/timeout design rationale. | — |

**Reproduced:** YES -- adversarial revert of the `i += 1;` fix caused the new test to fail with the
5-second timeout panic ("did not return within 5s -- regressed to the BUG-461 infinite loop");
restoring the fix passes. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/lottie_surface_rendering/src/animation.rs` | `layer_primitives_collect`: added the missing `i += 1;` before `continue`, with `Fix(BUG-461)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/minwebgl/lottie_surface_rendering/src/animation.rs` | Added `#[ cfg( test ) ] mod tests` with `layer_primitives_collect_skips_non_shape_content_without_hanging` (thread + channel + 5s `recv_timeout` bound). |
