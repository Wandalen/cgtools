# BUG-522: `raycaster`'s unclamped per-frame delta time lets the player tunnel through walls on a stalled frame

- **Severity:** Medium (silently breaks the core "walls are solid" gameplay invariant of this one
  example under realistic conditions -- a backgrounded tab, a GC pause, or a slow first frame --
  with no panic, no error, and no signal to the player; confined to this one example, not a
  shared-crate chokepoint)
- **state:** Completed
- **Affects:** `raycaster` example only (`src/main.rs`'s render loop)
- **Component:** `examples/minwebgl/raycaster` (`src/main.rs`, `src/sim.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Related Bugs:** Not a duplicate of any filed bug. Same defect *class* -- unbounded
  `requestAnimationFrame` delta time reaching a position update with no ceiling -- as the one
  `examples/minwgpu/flecs_bouncing_circles` already independently guards against via its own
  pre-existing `MAX_DT` constant; that guard was read as precedent while investigating this crate,
  not itself a bug.

## Symptom

```rust
// sim.rs -- player_pos = [ 3.5, 3.5 ], angle = 0.0 ( facing the wall at row 3 col 5, MAP[ 29 ] )
let move_dir = move_dir_resolve( pos, angle, 1.0 ); // 1.0 -- 1.5 units of clearance, movement permitted
let next = player_step( pos, angle, 1.3, 1.6, move_dir ); // pre-fix: raw stalled-frame dt = 1.6s
// next == [ 5.58, 3.5 ] -- inside the wall cell ( col 5 spans [ 5.0, 6.0 ) ), despite
// move_dir_resolve having just certified the wall was 1.5 units away, far past WALL_CLEARANCE ( 0.1 )
```

## Impact

**Who is affected:** Anyone playing the `raycaster` example whose browser tab stalls for a frame
long enough to produce a large `requestAnimationFrame` delta -- backgrounding the tab, a GC pause,
or the browser's own slow first frame after page load.

**What breaks:** The player's position update (`player_pos = player_step( ... )`) is not bounded by
the clearance `move_dir_resolve` checks before allowing movement -- that check only proves a wall
is farther than `WALL_CLEARANCE` (`0.1`) units away *at the instant the check runs*, not that the
upcoming step is small enough to respect that margin. A single oversized `delta_time` lets the
player's screen-space position, and every ray cast from it, end up inside or past a wall cell with
no error, no clamp, no visual warning -- silent state corruption of the one invariant (`walls block
movement`) the whole demo exists to showcase.

**Magnitude:** Confined to this one example -- `player_step`/`move_dir_resolve` are private to
`raycaster`'s own `src/sim.rs`, not shared with any other crate.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-defect sweep of 8 previously-unswept single-example crates,
`raycaster` among them. `src/main.rs`'s `delta_time = time - last_time` computation had no visible
ceiling; tracing `time`'s origin to `mingl::web::exec_loop::run` (`module/min/mingl/src/
web/exec_loop.rs`) confirmed it passes `requestAnimationFrame`'s raw timestamp through with zero
smoothing of its own, so nothing anywhere in the call chain bounded `delta_time` before this fix.
`examples/minwgpu/flecs_bouncing_circles`'s own pre-existing `MAX_DT` constant -- guarding the
identical failure mode in a different example -- was the precedent that made the missing guard here
stand out.

## Minimum Reproducible Example

```rust
// examples/minwebgl/raycaster/tests/wall_tunnel_test.rs
let pos = [ 3.5_f32, 3.5_f32 ];
let angle = 0.0_f32;
let move_dir = move_dir_resolve( pos, angle, 1.0 ); // permitted: 1.5 units of clearance
let unclamped_pos = player_step( pos, angle, 1.3, 1.6, move_dir ); // raw stalled-frame dt
// unclamped_pos == [ 5.58, 3.5 ] -- inside the wall at MAP[ 3 * 8 + 5 ]
```

**Verify Command** (<=3 lines, standalone):
```bash
cd examples/minwebgl/raycaster && cargo test --test wall_tunnel_test
```

## Root Cause

`main.rs`'s render loop computed `delta_time` directly from `requestAnimationFrame`'s raw
timestamp with no ceiling. `move_dir_resolve` casts one clearance ray and permits movement whenever
a wall is farther than `WALL_CLEARANCE` away -- a fixed distance check entirely independent of how
far the upcoming step will actually move the player. `player_step` then multiplies the unclamped
`delta_time` straight into the position update, so a large enough `delta_time` overshoots clean
through a wall the clearance check had just certified as "far enough away."

## Why Not Caught

Every manual playtest runs at a steady ~60fps, where `delta_time` never exceeds a few milliseconds
and the failure mode never triggers -- nothing in the render loop or its dependencies asserted an
upper bound on `delta_time` before this fix, and no test exercised a stalled/backgrounded-tab frame.

## Fix Location

`examples/minwebgl/raycaster/src/main.rs` (render loop): `delta_time` now goes through
`sim::frame_dt_clamp` before it reaches rotation or `player_step`. `examples/minwebgl/raycaster/
src/sim.rs` (new file): pure map/raycasting/movement logic extracted verbatim from `main.rs` --
`MAP`/`MAP_SIDE`, `ray_cast`, `direction`, `angle_wrap`, `move_dir_resolve`, `player_step` -- plus
the new `WALL_CLEARANCE`-documented `MAX_DT` constant and `frame_dt_clamp( raw_dt ) -> raw_dt.min(
MAX_DT )`. The extraction exists specifically so this logic can be exercised natively from
`tests/`, since `raycaster` is a binary-only crate (no `[lib]` target) -- same pattern
`examples/minwebgl/hexagonal_grid/tests/basic.rs` already established for a sibling binary-only
example. `MAX_DT` (`0.05`) is chosen so `MAX_DT * move_velocity < WALL_CLEARANCE` holds for
`main.rs`'s own `move_velocity` (`1.3`): `0.05 * 1.3 == 0.065 < 0.1`.

## Prevention

New test file `tests/wall_tunnel_test.rs`, two tests: `bug_reproducer_bug_522_stalled_frame_
tunnels_through_wall` drives the exact numeric scenario above -- confirms the raw, unclamped step
tunnels (documents why the fix is needed), then confirms the same raw `dt` routed through
`frame_dt_clamp` neither lands inside a wall nor exceeds `WALL_CLEARANCE`.
`max_dt_times_move_velocity_stays_under_wall_clearance` directly guards the constant relationship
(`MAX_DT * move_velocity < WALL_CLEARANCE`) the fix's soundness depends on, independent of any one
scenario.

## Pitfall

`move_dir_resolve`'s clearance check only proves a wall is farther than `WALL_CLEARANCE` away
*before* the step -- it says nothing about the step's own size. Any future movement code that
consumes raw, unclamped frame time the same way will reintroduce this exact failure mode under the
same conditions (stalled frame + any nonzero clearance check), regardless of how small
`WALL_CLEARANCE` is set. Raising `move_velocity` or lowering `WALL_CLEARANCE`/`MAX_DT` later without
re-checking `MAX_DT * move_velocity < WALL_CLEARANCE` would silently reopen this -- the invariant is
enforced only by the constants' values, not by the type system.

## Generalized Version

**Broken assumption:** "a clearance/proximity check evaluated once at the start of a frame is
enough to make an unbounded per-frame step safe."

**Confirmed general rule:** A distance-based clearance check and a step-size ceiling are two
independent guarantees -- the first says nothing about the second, no matter how conservative the
distance threshold is. Any per-frame movement driven by raw wall-clock/`requestAnimationFrame`
delta time needs its own explicit ceiling before that delta reaches a position update; a proximity
check alone is not a substitute, since it can always be defeated by a single sufficiently large
frame delta.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed | Found during a repo-wide bug/UX-defect sweep of 8 previously-unswept single-example crates; traced `main.rs`'s unclamped `delta_time` to `mingl::web::exec_loop::run`'s raw, unsmoothed `requestAnimationFrame` timestamp. |
| 2026-08-21 | fixed | Extracted pure logic to new `src/sim.rs`; added `MAX_DT`-documented `frame_dt_clamp`, applied to `main.rs`'s `delta_time` before use, plus the 3-field `Fix(BUG-522)`/`Root cause`/`Pitfall` source comment. |
| 2026-08-21 | verified | See Verification Record below. |
| 2026-08-21 | renumbered | Originally filed as BUG-512, then BUG-514, then BUG-516, then BUG-520; two other sweeps running live in this same shared repo (`gpu_picking`, `tiles_tools`) independently claimed and re-numbered their own files through the same 512-520 window at the same time this fix's own filing was polling it, colliding four times in a row. Settled on BUG-522 (re-derived live immediately before each successive write) before this file's first commit; all in-file/in-source references updated together in the same pass, each time -- see the sibling sweep's own filings in this same range for the other side of each collision. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: wrote `tests/wall_tunnel_test.rs`, ran it GREEN (2/2). Adversarial pass: temporarily edited `frame_dt_clamp`'s body to a no-op (`raw_dt` instead of `raw_dt.min( MAX_DT )`) -- `cargo test -p raycaster --test wall_tunnel_test` failed exactly as expected (`bug_reproducer_bug_522_...` FAILED, the unrelated constant-invariant test still passed, 1/2), confirming the test genuinely exercises the fix rather than passing vacuously; restored the real clamp, re-ran, GREEN (2/2). Full scoped suite `cargo test -p raycaster`: 2/2 pass. `cargo check -p raycaster --target wasm32-unknown-unknown`: clean. `cargo clippy -p raycaster --all-targets --all-features -- -D warnings`: clean -- adversarial pass on the test file itself caught 2 real issues on first attempt (a `dead_code` lint on `RayCollision::pos`, unread by the test crate's own narrower compilation graph despite being genuinely used by `main.rs`'s; two `clippy::float_cmp` lints on exact-literal-passthrough assertions), fixed by reading `pos` directly via a `ray_cast` sanity check and by adding a `#[ allow( clippy::float_cmp, reason = "..." ) ]` matching this codebase's own established precedent (`examples/minwebgl/morph_targets/tests/gui_weight_override_test.rs`), then re-verified clean. | Extracted `frame_dt_clamp` to `sim.rs`; used `RayCollision::pos` directly in the test; added scoped `float_cmp` allow with reason. |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-522)`/`Root cause`/`Pitfall` 3-field comment at the fix site in `main.rs`; 5-section (Root Cause/Why Not Caught/Fix Applied/Prevention/Pitfall) doc comment on `bug_reproducer_bug_522_...` in the test file. | — |
| D3 | Scope containment | — | 🟢 | Fix touches only `raycaster`: `main.rs`'s delta-time line, new `sim.rs` (logic moved verbatim, not rewritten), new test file, `readme.md`'s Responsibility Table (now required -- `src/` crossed the 3-file threshold). No other crate modified. | — |

**Reproduced:** YES -- with `frame_dt_clamp` temporarily reduced to a no-op,
`bug_reproducer_bug_522_stalled_frame_tunnels_through_wall` fails (clamped step lands inside the
wall); restoring the real clamp passes. Full scoped suite (2/2), wasm32 check, and clippy
(`-D warnings`) all clean, 2026-08-21.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/raycaster/src/main.rs` | Render loop: `delta_time` now routed through `sim::frame_dt_clamp` before use (full `Fix(BUG-522)` comment block); `mod sim;` added; `MAP`/`MAP_SIDE`/`ray_cast`/`direction`/`angle_wrap`/`RayCollision` moved to `sim.rs` and imported; inline movement match block replaced by calls to `sim::move_dir_resolve`/`sim::player_step`. |
| `examples/minwebgl/raycaster/src/sim.rs` | New file: pure map/raycasting/movement logic (`MAP`, `MAP_SIDE`, `WALL_CLEARANCE`, `MAX_DT`, `frame_dt_clamp`, `RayCollision`, `ray_cast`, `direction`, `angle_wrap`, `move_dir_resolve`, `player_step`), extracted from `main.rs` so it can run without a browser/GPU context. |
| `examples/minwebgl/raycaster/readme.md` | Added Responsibility Table (`src/` now has 3 files). |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/minwebgl/raycaster/tests/wall_tunnel_test.rs` | New file: `bug_reproducer_bug_522_stalled_frame_tunnels_through_wall` and `max_dt_times_move_velocity_stays_under_wall_clearance`, included via `#[ path = "../src/sim.rs" ] mod sim;` (binary-only crate, no `[lib]` target). |
