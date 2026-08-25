# BUG-456: `curve_surface_rendering`'s 3-entry `colors` palette had 2 dead entries, only `colors[0]` was ever read

- **Severity:** Low (no functional defect -- every point already rendered with the intended
  color -- this is dead/misleading code, not a behavior bug)
- **state:** Completed
- **Affects:** `examples/minwebgl/curve_surface_rendering`
- **Component:** `examples/minwebgl/curve_surface_rendering/src/main.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None.

## Symptom

```rust
// pre-fix -- curve_surface_rendering/src/main.rs:199-216 (approx.)
let colors = vec!
[
  F32x4::from_array( [ 1.0, 0.0, 0.0, 1.0 ] ), // red
  F32x4::from_array( [ 1.0, 1.0, 1.0, 1.0 ] ), // white
  F32x4::from_array( [ 0.0, 1.0, 0.0, 1.0 ] ), // green
];
// ...
p.color = colors[ 0 ];
```

A 3-entry color palette (red/white/green) was constructed, evidently intended for per-glyph or
per-font color cycling, but only `colors[ 0 ]` (red) was ever indexed anywhere in the assignment
loop -- `colors[1]`/`colors[2]` were unreachable dead data.

## Impact

**Who is affected:** No user-visible impact -- every point already rendered red, matching
`colors[0]`, both before and after this fix. This is a code-clarity/dead-code issue, not a
behavior change.

**What breaks:** Nothing functionally; the dead entries misled a reader into thinking per-font/
per-glyph color cycling was implemented when it wasn't.

**Magnitude:** 1 array (3 entries, 2 dead) + 1 indexed read simplified to a direct binding.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of the minwebgl example crates, auditing every indexed
array/`Vec` for whether all of its entries are actually read anywhere downstream.

## Root Cause

Investigated whether per-glyph/per-font color cycling should be implemented (the array's apparent
original intent) rather than deleted outright, per this repo's YAGNI-avoidance principle
("preserve freedom for uncertain aspects" cuts both ways -- don't speculatively build the missing
half of a feature either). Concrete evidence against implementing it:
- `font_names` (in the same function) has exactly **one** entry -- there is no second font to
  cycle a second color to.
- The whole `Vec< F32x4 >` this function returns is discarded at its only call site:
  `let ( canvas_gltf, _ ) = canvas_scene_setup( &gl ).await;` in `app_run` -- the second tuple
  element (the colors) is thrown away by `_`, so even a correctly-cycling palette would never
  reach the screen.

Both facts together mean per-font cycling was never exercisable even if fully implemented -- the
2 unused entries were dead from the moment they were written, not a regression.

## Why Not Caught

Example crates carry no `tests/` requirement (`health.md`), and an unused `Vec` entry produces no
compiler warning (the `Vec` itself is constructed and returned, so nothing is flagged as dead code
at the definition site -- only tracing every read of `colors[..]` reveals indices 1/2 are never
touched).

## Fix Location

`examples/minwebgl/curve_surface_rendering/src/main.rs`: collapsed the 3-entry `colors` array to a
single `let color = F32x4::from_array( [ 1.0, 0.0, 0.0, 1.0 ] );`, and changed
`p.color = colors[ 0 ];` to `p.color = color;`. Behavior is unchanged (still red) -- this is a
dead-code deletion, not a functional fix.

## Prevention

No test added -- this is a dead-code removal with no behavior to regress-test (the rendered output
is identical before and after). Verified via `cargo check --target wasm32-unknown-unknown`
(confirms no leftover reference to the deleted `colors` binding or its now-nonexistent indices).

## Pitfall

An indexed collection where only index 0 is ever read is a sign the surrounding
cycling/selection logic was never finished (or never wired to a real second case) -- grep every
read of the collection before assuming the other entries are load-bearing, and check whether the
collection's own return value is even consumed by its caller before investing effort in
"completing" it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of minwebgl example crates; discovery and fix landed together in one session. |
| 2026-08-20 | fixed | Deleted the 2 dead palette entries (chose deletion over implementing per-font cycling -- see Root Cause for the concrete evidence behind that judgment call: only 1 font configured, and the whole return value is discarded by the only caller). |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Deletion judgment call is evidence-backed, not speculative | — | 🟢 | Adversarial pass: re-checked `font_names`'s entry count (1) and re-traced the function's return value to its only call site (`let ( canvas_gltf, _ ) = canvas_scene_setup( &gl ).await;`) -- both facts independently confirmed via direct source read, not assumed. | — |
| D2 | Compiles for wasm32 target, no dangling references | — | 🟢 | `cargo check --target wasm32-unknown-unknown -p curve_surface_rendering` (combined with the other 7 touched crates in one invocation) -- exit 0, zero errors, zero warnings; confirms no remaining reference to the deleted `colors` `Vec` or its indices. | — |

**Reproduced:** N/A -- dead-code removal, no behavior to reproduce; verified via direct source
inspection (pre-fix: 2 unreachable entries; post-fix: single `color` binding, identical rendered
output) and `cargo check`. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/curve_surface_rendering/src/main.rs` | Collapsed the 3-entry `colors : Vec< F32x4 >` (red/white/green, only index 0 read) to a single `color : F32x4` binding; updated the one assignment site accordingly. |

## Refs: tests/

| File | Change |
|------|--------|
| — | No test added -- dead-code removal with no behavior change to regress-test. |
