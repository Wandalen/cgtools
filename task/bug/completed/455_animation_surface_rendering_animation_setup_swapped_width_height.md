# BUG-455: `animation_surface_rendering` called `animation_setup( gl, width, height )` with `( height, width )` transposed

- **Severity:** Low (visual only -- affects the aspect ratio used to set up the animation surface,
  not a crash or data loss)
- **state:** Completed
- **Affects:** `examples/minwebgl/animation_surface_rendering`
- **Component:** `examples/minwebgl/animation_surface_rendering/src/main.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None.

## Symptom

```rust
// pre-fix -- animation_surface_rendering/src/main.rs (was line 453/560 pre-earlier-edits in this
// session; call site, not the function definition)
fn animation_setup( gl : &GL, width : usize, height : usize ) -> animation::Animation { ... }
// ...
let animation = animation_setup( &gl, canvas.height() as usize, canvas.width() as usize );
```

`animation_setup`'s own signature takes `( width, height )` in that order, but its call site
passed `canvas.height()` as the `width` argument and `canvas.width()` as the `height` argument --
transposed.

## Impact

**Who is affected:** Any user of the `animation_surface_rendering` demo on a canvas whose width
and height differ (the common case -- most viewports aren't square).

**What breaks:** The animation surface is set up with the aspect ratio inverted relative to the
actual canvas -- whatever `animation_setup` internally derives from `width`/`height` (e.g.
viewport-relative sizing, aspect-correct layout of the rendered animation) uses the wrong ratio.

**Magnitude:** 1 call site, 2 arguments swapped.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of the minwebgl example crates, cross-checking every call to
`animation_setup` against its own `( width, height )` parameter declaration.

## Minimum Reproducible Example

```rust
fn animation_setup( gl : &GL, width : usize, height : usize ) -> animation::Animation { /* ... */ }

// pre-fix call site (transposed):
animation_setup( &gl, canvas.height() as usize, canvas.width() as usize );
// on any canvas where width != height, `width`/`height` inside animation_setup are swapped
// relative to the caller's actual canvas dimensions.
```

**Verify Command:** N/A -- no isolable pure-logic unit to test natively (`animation_setup` itself
takes a live `&GL` context); verified via direct source inspection confirming the call site now
matches the function's own parameter order, plus `cargo check --target wasm32-unknown-unknown`
(see Verification Record).

## Root Cause

Two same-typed (`usize`) positional parameters, `width` and `height`, compile fine in either
order -- the call site was authored with them transposed and nothing caught it, since there's no
type-level distinction between "a width" and "a height."

## Why Not Caught

Example crates carry no `tests/` requirement (`health.md`), and the visual effect of a swapped
aspect ratio is easy to miss without a side-by-side reference, especially if the demo's default
canvas happens to be close to square.

## Fix Location

`examples/minwebgl/animation_surface_rendering/src/main.rs`: swapped the call-site argument order
back to `animation_setup( &gl, canvas.width() as usize, canvas.height() as usize )`, matching the
function's own `( width, height )` declaration.

## Prevention

No native regression test is practical (the call site is a live-`&GL`-context integration point,
not isolable pure logic). The fix itself is the durable guard -- the call site now reads
`canvas.width()` for the `width` parameter and `canvas.height()` for `height`, matching the
function signature at the call site directly, verified via `cargo check --target
wasm32-unknown-unknown`.

## Pitfall

Two same-typed positional parameters (`width`/`height`, both `usize`) compile fine in either
order -- nothing at the type level catches a transposition. When a function takes multiple
same-typed dimension parameters, double-check every call site against the declaration order
directly rather than assuming argument order "looks right."

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of minwebgl example crates; discovery and fix landed together in one session. |
| 2026-08-20 | fixed | Swapped the call-site argument order back to `( width, height )`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Call site matches function signature order | — | 🟢 | Adversarial pass: re-read `animation_setup`'s declaration (`fn animation_setup( gl : &GL, width : usize, height : usize )`) side-by-side with the fixed call site (`animation_setup( &gl, canvas.width() as usize, canvas.height() as usize )`) -- confirmed `width`↔`width`, `height`↔`height`, no other call sites of `animation_setup` exist in this crate (`grep -n animation_setup`). | — |
| D2 | Compiles for wasm32 target | — | 🟢 | `cargo check --target wasm32-unknown-unknown -p animation_surface_rendering` (combined with the other 7 touched crates in one invocation) -- exit 0, zero errors, zero warnings. | — |

**Reproduced:** N/A (no native reproduction harness for a live-`&GL`-context call site) -- pre-fix
source inspected directly to confirm the transposition; post-fix source inspected directly to
confirm the call site now matches the function's own parameter order. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/animation_surface_rendering/src/main.rs` | Swapped `animation_setup( &gl, canvas.height() as usize, canvas.width() as usize )` to `animation_setup( &gl, canvas.width() as usize, canvas.height() as usize )`. |

## Refs: tests/

| File | Change |
|------|--------|
| — | No native test practical (live-`&GL`-context call site); verified via `cargo check --target wasm32-unknown-unknown` and direct source inspection. |
