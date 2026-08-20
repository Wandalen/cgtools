# BUG-309: `pingpong_animation`'s hardcoded paddle X constants were never checked against the actual simulation's court bounds or the SVG adapter's lack of X-centering, rendering paddles off-canvas

- **Severity:** Medium (paddles rendered visibly wrong -- one entirely off the default canvas)
- **state:** Completed
- **Affects:** `examples/scene_script/pingpong_animation/src/render.rs`
- **Component:** examples/scene_script/pingpong_animation
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`PADDLE_LEFT_X`/`PADDLE_RIGHT_X` were hardcoded to `-380.0`/`380.0`. The actual simulation's
court, declared in `pingpong_animation.rhai` (`let court = f32x2(200.0, 100.0)`), bounces the
ball at `ball_pos.x <= 0.0 || ball_pos.x >= court.x` -- i.e. the real horizontal play area is
`[0.0, 200.0]`, not centered on 0. `frame.ball.x()` is rendered raw with no scaling/offset, and
`tilemap_renderer/src/adapters/svg.rs`'s `transform_to_svg_static` maps `position.x` directly onto
the SVG canvas with no centering (`pos_x = t.position[0]`, only Y gets a height-flip) -- so `x=0`
is the canvas' left edge. The old `-380.0`/`380.0` values were never checked against either fact:
the left paddle rendered entirely off the default 800px-wide canvas (negative x), and the right
paddle sat at `380.0`, ~180 units past the court's real `200.0` right boundary.

## Impact

**Who is affected:** anyone running the `adapter-svg`-featured build of this demo and viewing the
rendered output.

**What breaks:** the left paddle is invisible (rendered off-canvas), and the right paddle renders
far outside the actual play area the ball bounces within -- neither paddle visually corresponds
to where the simulation's bounce boundaries actually are.

**Entity Scope:** `None` -- rendering-only defect, simulation logic itself (bounce physics) was
already correct.

## How Discovered

Disclosed by a fork bug-hunting 7 `math`/`orrery`/`scene_script`/`renderer`/`tilemap_renderer`
crates (task #183). Independently verified by reading the actual `.rhai` script (confirming
`court = f32x2(200.0, 100.0)` and the bounce condition) and the actual `transform_to_svg_static`
function in `module/helper/tilemap_renderer/src/adapters/svg.rs` (confirming `pos_x =
t.position[0]` with no centering applied).

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "let court" examples/scene_script/pingpong_animation/src/pingpong_animation.rhai
grep -n "PADDLE_LEFT_X\|PADDLE_RIGHT_X" examples/scene_script/pingpong_animation/src/render.rs
```
**Expected** (fixed): `PADDLE_LEFT_X = 0.0`, `PADDLE_RIGHT_X = 200.0`, matching the `.rhai`
script's declared court bounds. **Actual** (pre-fix): `PADDLE_LEFT_X = -380.0`, `PADDLE_RIGHT_X =
380.0`, unrelated to the court's actual `[0.0, 200.0]` range.

## Root Cause

The paddle X constants were authored as plausible-looking hardcoded values, disconnected from
both ground truths that determine where they should actually sit: the `.rhai` script's declared
court width, and the SVG adapter's coordinate convention (no X-centering).

## Why Not Caught

Every existing test (T01-T05, AF2) exercised frame-to-command translation and rendering
correctness in general, but none of them ever inspected a paddle's specific x position against
the court's real boundaries.

## Fix Applied (2026-08-18)

Changed `PADDLE_LEFT_X`/`PADDLE_RIGHT_X` from `-380.0`/`380.0` to `0.0`/`200.0`, matching the
`.rhai` script's declared court bounds exactly.

Added `t06_paddle_x_positions_match_the_courts_bounce_boundaries` to `tests/render_test.rs`, with
a new `rhai_court_x()` helper that parses the `.rhai` script's declared court X value as ground
truth (rather than comparing against `simulate()`'s own observed ball range -- per the fork's
disclosed account, an initial wrong approach was caught and corrected: with only 40 ticks and the
ball starting at the court's horizontal center moving toward the right wall, the recorded frames
never actually reach back to the left wall within this short a run, so the court's *declared*
boundary, not a short run's incomplete observed traversal, is the correct ground truth).

## Verification

RED proof (per the fork's own account, manually confirmed by transiently restoring the pre-fix
`PADDLE_LEFT_X = -380.0`/`PADDLE_RIGHT_X = 380.0` values before writing the fix, then reverting):
with those values, the left paddle sits at a negative x (off the default 800px-wide canvas
entirely) and the right paddle sits at `380.0`, ~180 units from the court's real `200.0` right
boundary -- both existing tests (T01-T05, AF2) still passed throughout, since none of them ever
inspected a paddle's x position.

- **Post-fix (GREEN), independently re-run by the orchestrating session:** `cargo test -p
  pingpong_animation --features adapter-svg --tests` (combined `longrun`-detached sweep) →
  `render_test.rs` 6/6 (including `t06_paddle_x_positions_match_the_courts_bounce_boundaries`),
  `simulation_test.rs` 1/1, all passed. Also confirmed clean under default features (no
  `adapter-svg`): `render_test.rs` 0/0 (SVG-backend tests correctly gated out),
  `simulation_test.rs` 1/1. `cargo clippy -p pingpong_animation --all-targets --all-features -- -D
  warnings` → clean.

## Generalized Version

A hardcoded constant disconnected from ground truth is a distinct defect class from doc drift --
before hardcoding a positional/dimensional constant, trace it back to the actual source of truth
that determines its correct value (here: the simulation's own declared bounds and the rendering
adapter's coordinate convention), rather than picking a plausible-looking number. When writing the
regression test, prefer the ground-truth *declaration* (the `.rhai` script's own stated court
width) over a short run's *observed* behavior, which may not have traversed the full range yet.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting 7 `math`/`orrery`/`scene_script`/`renderer`/`tilemap_renderer` crates (task #183, one of 3 parallel forks covering 27 `examples/` crates); reported via both an `<agent-message from="fork">` cross-session channel and the standard task-notification for the same agent ID (corroborating, confirming genuineness); fixed and tested with a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file, including an honest disclosed account of an initial wrong test-design approach caught and corrected by the fork itself before finalizing. Independently verified by the orchestrating session (diff read, `.rhai` script and SVG adapter source both directly read to confirm the root cause, test independently re-run via a `longrun`-detached sweep after resolving 2 separate log-auto-discovery collisions) before this report and its real ID were assigned; placeholder replaced with BUG-309 after a fresh on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor. |
