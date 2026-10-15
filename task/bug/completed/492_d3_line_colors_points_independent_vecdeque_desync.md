# BUG-492: `line_tools::d3::Line`'s `colors` `VecDeque` is fully independent from `points`/`distances`, silently desyncing on a mismatched add/remove call

- **Severity:** Medium (no crash -- `mesh_update` uploads whatever length `colors` happens to be;
  a desync silently shifts every subsequent point's rendered color by an index offset, or leaves
  stale trailing entries, with no error anywhere)
- **state:** Completed
- **Affects:** Any consumer of `d3::Line` that calls a `point_*`/`points_*` add/remove method
  without calling the matching `color_*`/`colors_*` method for the exact same count, while
  `vertex_color_use( true )` (the default) is in effect. Confirmed via workspace-wide audit: the
  one real caller that performs a *bulk* removal, `examples/minwebgl/3d_line/src/main.rs:151-152`
  (`points_remove_front_no_distance_update( n )` immediately followed by
  `colors_remove_front( n )`), already calls both with the same count -- so no *currently shipped*
  caller triggers a desync today. It remains a latent defect: nothing in the API prevents a future
  caller (or a future edit to this exact call site) from calling one side without the other, and
  `falling_frontier`'s own `d3::Line` usage sidesteps the whole question by disabling vertex colors
  (`line.vertex_color_use( false )`) rather than by the API enforcing consistency.
- **Component:** `module/helper/line_tools` (`src/d3/line.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None known.

## Symptom

```rust
let mut line = d3::Line::default();
line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
line.point_add_back( &[ 2.0, 0.0, 0.0 ] );
line.color_add_back( [ 1.0, 0.0, 0.0 ] );
line.color_add_back( [ 0.0, 1.0, 0.0 ] );
line.color_add_back( [ 0.0, 0.0, 1.0 ] );

line.point_remove_front();
// points_get().len() == 2, colors_get().len() == 3 -- desynced, no error
```

## Impact

**Who is affected:** Any `d3::Line` consumer that removes (or adds) points/distances via one of
`impl_basic_line!`'s methods without separately, correctly, mirroring the exact same operation on
`colors` -- `colors` is a fully independent `VecDeque` with its own parallel add/remove family
that `impl_basic_line!` never touches. Once desynced, `mesh_update`'s colors-upload block
(guarded by `colors_changed && vertex_color_use`) uploads `self.geometry.colors` at whatever
length it happens to be, with no check against `self.geometry.points.len()` -- every point after
the shorter side's length silently gets no color update (stale/wrong color), or, if `colors` is
longer, the extra trailing entries are simply never read by anything, wasting memory.

**What breaks:** Visual only (wrong or stale per-vertex color) when the desync happens while
`vertex_color_use( true )`; no effect when colors are disabled (`falling_frontier`'s own usage).
No panic, no crash, no effect on `points`/`distances` themselves (independent storage).

**Consumer audit:** the one real bulk-removal call site
(`examples/minwebgl/3d_line/src/main.rs:151-152`) already pairs
`points_remove_front_no_distance_update( n )` with `colors_remove_front( n )` using the same
count -- confirming developers *can* get this right by discipline, but the API gives them nothing
that catches it if they don't. `examples/minwebgl/falling_frontier/src/trajectories.rs` only ever
adds points (`point_add_back`), never removes, and explicitly disables vertex colors
(`line.vertex_color_use( false )`), sidestepping the issue entirely rather than relying on any
built-in consistency guarantee.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Assigned as part of a repo-wide bug/UX sweep of `line_tools`, `canvas_renderer`, and
`browser_input`, naming this exact structural gap: `points`/`distances` are kept in lockstep by
every `impl_basic_line!` add/remove method, but `colors` (a separate, hand-added field on
`d3::Line`'s `LineGeometry`) has its own independent add/remove family that the macro never
touches, and `mesh_update` performs no length-consistency check before uploading colors.

## Minimum Reproducible Example

```rust
// module/helper/line_tools/tests/webgl/colors_desync.rs
let mut line = Line::default();
line.point_add_back( &[ 0.0_f32, 0.0, 0.0 ] );
line.point_add_back( &[ 1.0_f32, 0.0, 0.0 ] );
line.point_add_back( &[ 2.0_f32, 0.0, 0.0 ] );
line.color_add_back( [ 1.0_f32, 0.0, 0.0 ] );
line.color_add_back( [ 0.0_f32, 1.0, 0.0 ] );
line.color_add_back( [ 0.0_f32, 0.0, 1.0 ] );
line.point_remove_front();
// points_get().len() == 2, colors_get().len() == 3
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/line_tools && cargo nextest run -E 'test(bug_492)'
```

## Root Cause

`LineGeometry` (`d3/line.rs`) stores `points : VecDeque<F32x3>`, `distances : VecDeque<f32>`
(feature-gated), and `colors : VecDeque<F32x3>` as 3 separately-owned collections. `points`/
`distances` are kept in lockstep by construction: every add/remove method generated by the
`impl_basic_line!` macro (`lib.rs`) updates both together in one call. `colors`, however, is a
hand-written addition to `d3::Line` specifically (not part of the macro, and not present on
`d2::Line` at all), with its own parallel `color_add_back`/`color_remove_front`/etc. family that
`impl_basic_line!`'s methods have no knowledge of and never call. Nothing links the two families:
calling a `point_*` method alone is a fully valid, panic-free way to desync `colors` from
`points`, and `mesh_update` never checked their lengths against each other before this fix.

## Why Not Caught

No existing test constructed a `Line`, added matching points and colors, and then removed only
one side -- `points`/`distances`' own lockstep invariant (enforced by construction, within the
macro) is implicitly well covered by any test that exercises point add/remove at all, but nothing
exercised `colors` against that same invariant, since `colors` sits outside the macro entirely.

## Fix Location

`module/helper/line_tools/src/d3/line.rs`: added `colors_length_consistency_check( colors_len,
points_len ) -> Result< (), gl::WebglError >`, a pure function (no GL context needed) that returns
`Err` (after logging the actual mismatched lengths via `gl::warn!`) when the two lengths differ.
Called as the first line inside `mesh_update`'s existing `if self.change_state.colors_changed &&
self.render_state.vertex_color_use { .. }` block, before the colors buffer upload -- so a desync
is only surfaced when it would actually matter (colors were touched and are actually being
rendered), not on every `mesh_update` call regardless of whether colors are in play.

**Judgment call (fold vs. assert):** the task allowed either folding `colors` into the same
lockstep structure as `points`/`distances`, or adding an explicit consistency check that fails
loudly on mismatch. Chose the **assertion/fail-loud** approach: folding `colors` into
`impl_basic_line!`'s structure would mean adding a 4th, always-present field to a macro shared
with `d2::Line` (which has no `colors` field at all), forcing an invasive macro change and a
new-field-on-a-type-that-doesn't-want-it problem for `d2::Line`. The extracted pure-function
check mirrors this crate's own established precedent for testable GL-adjacent logic
(`canvas_renderer::renderer::mesh_colors_resolve`, extracted from `render()` for the identical
reason: testable without a live `WebGl2RenderingContext`) -- a narrower, less invasive fix that
still makes the desync impossible to silently ship into a GL upload.

## Prevention

New file `module/helper/line_tools/tests/webgl/colors_desync.rs`, two tests:
`point_remove_front_without_matching_color_remove_desyncs_colors_and_points_bug_492` (confirms the
desync is genuinely reachable through the crate's real public API -- builds a `Line`, adds 3
matching points/colors, calls only `point_remove_front()`, asserts the resulting length mismatch)
and `colors_length_consistency_check_rejects_mismatched_lengths_and_accepts_matched_ones` (direct
unit coverage of the new guard function itself: matched lengths incl. `(0, 0)` pass, mismatched
lengths in both directions fail). `mesh_update`'s own call site cannot be exercised natively (no
live `WebGl2RenderingContext` in this crate's test environment), so the guard's logic is pinned
directly instead.

## Pitfall

A doc comment asserting index-correspondence between two independently-mutable collections is not
an enforced invariant -- only a check at the point of consumption (or, more robustly, folding both
into one structure) actually prevents the desync from being observable. When one collection
(`colors`) is added to a type built around a shared macro (`impl_basic_line!`) but implemented by
hand outside it, every invariant the macro enforces for its own fields silently stops applying to
the hand-added one -- the burden shifts entirely onto every future caller remembering to keep both
in step, unenforced by the type system.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Assigned as part of a repo-wide bug/UX sweep, naming the exact structural gap between `impl_basic_line!`'s lockstep `points`/`distances` and the independently-implemented `colors`. |
| 2026-08-20 | fixed | Added `colors_length_consistency_check` (pure function) and wired it into `mesh_update`'s colors-upload guard; judgment call (assert vs. fold) documented above. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Both new tests confirmed passing against the fixed code (`cargo nextest run -E 'test(bug_492) or test(colors_length_consistency_check)'` -- 2/2 pass). The public-API reproducer test (`point_remove_front_without_matching_...`) needed no fix-dependent revert to prove itself real: it asserts the *desync itself* (an `impl_basic_line!`-level fact, unaffected by this fix, which only adds a downstream guard) is reachable, which is independently true before and after this fix -- the guard-logic unit test directly exercises `colors_length_consistency_check`'s pass/fail branches on both sides. Full scoped suite: `cargo nextest run -p line_tools -p canvas_renderer -p browser_input --all-features` -- 139/139 pass; `cargo clippy` clean; `cargo test --doc` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-492)`/`Root cause`/`Pitfall` 3-field format on both the new function's doc comment and its call site in `src/d3/line.rs`. | — |
| D3 | Scope containment | — | 🟢 | Confirmed via `git diff` that only `src/d3/line.rs` (fix) and `tests/webgl/colors_desync.rs`+`tests/webgl/mod.rs`+`tests/webgl/readme.md` (tests/registration) were touched for this bug. | — |

**Reproduced:** YES -- `point_remove_front_without_matching_color_remove_desyncs_colors_and_points_bug_492`
demonstrates the desync is real and public-API-reachable (`points_get().len() == 2` vs.
`colors_get().len() == 3` after a single `point_remove_front()` call with no matching
`color_remove_front()`); `colors_length_consistency_check_rejects_mismatched_lengths_and_accepts_matched_ones`
confirms the new guard correctly rejects exactly that shape of mismatch. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/line_tools/src/d3/line.rs` | Added `colors_length_consistency_check` (pure function, exported via `mod_interface!`'s `own use`); called as the first statement inside `mesh_update`'s `colors_changed && vertex_color_use` block, before the colors buffer upload. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/line_tools/tests/webgl/colors_desync.rs` | New file: 2 tests -- public-API desync reproducer, and direct unit coverage of the new guard function. |
| `module/helper/line_tools/tests/webgl/mod.rs` | Added `mod colors_desync;`. |
| `module/helper/line_tools/tests/webgl/readme.md` | Registered `colors_desync.rs` in the Responsibility Table. |
