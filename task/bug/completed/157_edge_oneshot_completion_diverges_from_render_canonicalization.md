# BUG-157: `Scene::tick_into`'s `OneShot` completion phase uses the raw, non-canonical edge hex instead of the render path's canonicalized one

- **Severity:** Medium (silent event-timing defect, not a crash -- affected instances still fire
  their `AnimationCompleted` event, just at the wrong tick, and only when both a
  position-dependent phase offset and a non-canonically-declared `Edge` placement are combined)
- **state:** Completed
- **Affects:** `Scene::tick_into`'s `OneShot` completion-crossing detection for any instance
  placed via `Placement::Edge{hex,dir}` whose owning `Animation` uses
  `PhaseOffset::HashCoord` or `PhaseOffset::Linear` (both depend on `pos`) and which was declared
  on the edge's non-canonical side
- **Component:** `module/helper/tilemap_scene` (`src/scene.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None (independent of BUG-156, filed in the same review batch of task #92's
  `tilemap_scene` pass but a different code path -- this is scene-tick event timing, BUG-156 is
  compile-time draw order).

## Symptom

```rust
// Two instances on the SAME physical hex edge, declared from opposite sides,
// with an animation phase that depends on position.
let a = scene.spawn( actor, Placement::Edge { hex: (0, 0), dir: EdgeDirection::N } );   // non-canonical side
let b = scene.spawn( actor, Placement::Edge { hex: (0, -1), dir: EdgeDirection::S } );  // canonical side -- same edge as `a`
// anim.phase_offset = PhaseOffset::Linear { per_q: 0.0, per_r: -0.2 }; duration = 0.5s

let evs = scene.tick( 0.35 ); // cumulative clock crosses the shared 0.3s crossing point
// Wrong (pre-fix):   evs.len() == 1  (only `b`, the already-canonical instance, fires here;
//                     `a` doesn't cross until clock reaches 0.5, 0.2s later)
// Correct (post-fix): evs.len() == 2  (both fire together -- they're the same edge)
```

## Impact

**Who is affected:** Any scene with `Edge`-placed `OneShot` instances whose animation uses a
position-dependent `PhaseOffset` (`HashCoord` or `Linear`) and where at least one instance is
declared on the non-canonical side of its edge -- plausible whenever edge instances are authored
by hand or generated from asymmetric source data (only one adjacent tile "owns" the edge
declaration in the source format), since `Placement::Edge`'s own doc explicitly defers the
canonical-side decision to "the renderer," implying callers are not expected to pre-canonicalize.

**What breaks:** `declared_phase_seconds` (`compile/animation.rs:167-173`) documents itself as
agreeing "byte-for-byte with what `animation_frame_resolve` would show on screen." The render
path (`edge_pass_scene_compile` → `edge_sprite_source_resolve`, `compile/frame.rs`) canonicalizes
the edge's hex via `canonical_edge` before resolving phase. Pre-fix, `Scene::tick_into` computed
phase from `inst.placement.hex_coord()`, which returns `Placement::Edge`'s raw `hex` field
verbatim -- no canonicalization. For an instance declared on the non-canonical side, the
`AnimationCompleted` event fires at a different tick than what the rendered frame would actually
show as the animation's true completion point, breaking the documented agreement between the two
paths.

**Magnitude:** Silent -- no panic, no error. The event still fires, just off-schedule relative to
the visual. Only observable by comparing event timing against rendered output, or (as this bug's
regression test does) by placing two instances on opposite sides of one edge and observing they
don't fire together despite representing the same physical location.

**Entity Scope:** None -- a code-level defect, not an operational-entity concern.

## How Discovered

Flagged during a background review pass over `tilemap_scene` (task #92). Independently
re-derived the full causal chain via direct reads of `instance.rs` (`Placement::Edge`'s doc,
`hex_coord()`'s raw pass-through), `scene.rs` (`tick_into`'s phase computation),
`compile/edges.rs` (`canonical_edge`'s actual canonicalization rule), and `compile/animation.rs`
(`declared_phase_seconds`'s own "byte-for-byte" doc promise and its `Linear`/`HashCoord` position
dependence) before filing -- confirming both code paths operate on the identical `Instance` data
but diverge specifically in whether they canonicalize first.

## Minimum Reproducible Example

```bash
cd module/helper/tilemap_scene && cargo test --test scene_events_test edge_instances_on_opposite_sides_of_same_edge_complete_together 2>&1 | tail -12
```

**Expected** (post-fix):
```
test edge_instances_on_opposite_sides_of_same_edge_complete_together ... ok
```

**Actual** (pre-fix -- confirmed via in-place revert-test-restore against the real unfixed code):
```
thread 'edge_instances_on_opposite_sides_of_same_edge_complete_together' panicked at module/helper/tilemap_scene/tests/scene_events_test.rs:468:3:
assertion `left == right` failed: instances on opposite sides of the same edge must share one canonical phase and complete in the same tick; saw [AnimationCompleted { instance: InstanceHandle(2v1), state: StateHandle { object: ObjectHandle(0), state_index: 1 }, layer_index: 0, animation: AnimationRef("spawn_fx") }] (a build that reads the raw, non-canonical hex would fire only the already-canonical instance here, with the other arriving 0.2s later)
  left: 1
 right: 2
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 16 filtered out
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_scene && cargo test --test scene_events_test edge_instances_on_opposite_sides_of_same_edge_complete_together
# ok = fixed; assertion `left == right` (1 vs 2 events) = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `tick_into` computes `OneShot` completion phase from the raw, uncanonicalized `Placement::Edge.hex`, diverging from the render path's canonicalized phase whenever an edge is declared non-canonically and the phase offset is position-dependent. | ✅ Root Cause | Direct read of `tick_into`'s `pos` computation (pre-fix: bare `inst.placement.hex_coord()`) versus `edge_pass_scene_compile`'s canonicalize-then-resolve sequence confirmed the divergence; the MRE's captured failure (only the already-canonical instance fires) matches exactly. | E1, E2, E3 |
| H2 | The divergence only matters for `PhaseOffset::None`/`Fixed`, which don't depend on `pos` at all. | ❌ Rejected | `declared_phase_seconds`'s `match` (`compile/animation.rs:182-199`) shows `None`/`Fixed` ignore `pos` entirely -- confirming the bug is real specifically for, and only for, `HashCoord`/`Linear`, both of which read `pos.0`/`pos.1` directly. | E4 |
| H3 | `Scene` has no accessible `TilingStrategy` inside `tick_into`, so canonicalization can't be performed there without a signature change. | ❌ Falsified | `Scene.spec: Arc<RenderSpec>` is already a field; `self.spec.pipeline.hex.tiling` resolves to the same `TilingStrategy` the render path uses (`RenderSpec.pipeline: RenderPipeline`, `RenderPipeline.hex: HexConfig`, `HexConfig.tiling`) -- confirmed via direct struct-field reads, no new field or signature change needed. | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/instance.rs:90` | `Placement::Edge{hex,dir}` doc: "Owning hex (the canonical-side decision is made by the renderer)" -- explicitly defers canonicalization to callers/renderer, confirming raw `hex` is expected to sometimes be non-canonical. | H1 ✅ |
| E2 | `src/compile/animation.rs:171-174` (unedited) | `declared_phase_seconds`'s own doc: "Mirrors the renderer's frame-resolution path so completion-event detection in `Scene::tick` agrees byte-for-byte with what `animation_frame_resolve` would show on screen." | H1 ✅ |
| E3 | `-0003_longrun.log`-equivalent (in-place revert-test-restore run against the real unfixed code) | Captured exact pre-fix failure: only 1 of 2 same-edge instances fires in the shared-crossing tick, precisely the predicted divergence. | H1 ✅ |
| E4 | `src/compile/animation.rs:182-199` (unedited) | `PhaseOffset::None => 0.0`, `Fixed(s) => s` (both `pos`-independent); `HashCoord`/`Linear` both read `pos.0`/`pos.1`. | H2 ❌ |
| E5 | `src/scene.rs` (`Scene` struct fields) / `src/pipeline.rs:14-17,85+` | `Scene.spec: Arc<RenderSpec>`; `RenderSpec.pipeline: RenderPipeline`; `RenderPipeline.hex: HexConfig`; `HexConfig.tiling: TilingStrategy` -- full path already reachable as `self.spec.pipeline.hex.tiling`, matching `compile/frame.rs`'s own usage. | H3 ❌ |

## Root Cause

```
Scene::tick_into (pre-fix)
  let pos = inst.placement.hex_coord().unwrap_or( ( 0, 0 ) );
  // Placement::Edge{hex,..} -> Some(hex) -- the RAW, possibly non-canonical hex.

edge_pass_scene_compile -> edge_sprite_source_resolve (render path, unchanged)
  let canon = canonical_edge( EdgePosition{hex,dir}, tiling )...;
  // uses canon.0 -- the CANONICALIZED hex.
```

Two readers of the same `Instance.placement` field diverge on whether they canonicalize before
using it as a phase-offset input.

## Why Not Caught

No existing test placed two `Edge` instances on opposite (non-canonical vs. canonical) sides of
the *same* physical edge with a position-dependent `PhaseOffset`. The existing `HashCoord`
divergence test (`hash_coord_phase_can_separate_completions`) exercises `Placement::Hex` at two
*different* hexes -- a case where divergence is the intended, correct behavior, not a bug -- so
it couldn't have caught this. `Placement::Edge` doesn't appear anywhere else in
`scene_events_test.rs`.

## Fix Location

`module/helper/tilemap_scene/src/scene.rs`, `Scene::tick_into`:

```rust
// before
let pos = inst.placement.hex_coord().unwrap_or( ( 0, 0 ) );

// after
let pos = match inst.placement
{
  Placement::Edge { hex, dir } => canonical_edge( EdgePosition { hex, dir }, self.spec.pipeline.hex.tiling )
    .map_or( hex, | canon | canon.0 ),
  _ => inst.placement.hex_coord().unwrap_or( ( 0, 0 ) ),
};
```

Falls back to the raw `hex` only if `canonical_edge` itself returns `None` (an invalid
dir/tiling combination) -- preserving prior behavior exactly for that edge case rather than
introducing a new panic or a silent `(0,0)` default. Non-`Edge` placements are unaffected
(`hex_coord()`'s existing behavior, including the `(0,0)` `FreePos`/`Viewport` fallback, is
unchanged).

## Prevention

Added `edge_instances_on_opposite_sides_of_same_edge_complete_together`
(`bug_reproducer(BUG-157)`) to `tests/scene_events_test.rs`: two instances on the same physical
edge (`hex=(0,0),dir=N` and `hex=(0,-1),dir=S` under `HexFlatTop`, confirmed via
`neighbor_offset_by_dir`/`opposite_dir` to canonicalize to the same `((0,-1), S)`), a
`PhaseOffset::Linear{per_r:-0.2}` animation (so raw vs. canonical `pos` predict different
phases), asserts both complete in the same `tick()` call and neither fires early or repeats.

## Pitfall

Two code paths reading the same `Instance` field can silently diverge when only one of them
applies a normalization step the field's own doc comment defers to "the renderer" -- reading
`hex_coord()`'s doc alone gives no signal that its raw pass-through is unsafe for `Edge`
placements specifically. Grep every reader of a field whose doc says "canonicalized elsewhere"
or similar, not just the obviously-rendering-related one; event-detection and rendering are
easy to treat as separate concerns even when a doc comment (`declared_phase_seconds`'s
"byte-for-byte" promise) explicitly says they must agree.

## Generalized Version

**Broken assumption:** "canonicalization is a rendering concern, so only rendering code needs to
apply it." False here -- `declared_phase_seconds` is shared by both the render path and the
event-detection path specifically because they're documented to agree; canonicalizing in only
one of the two shared callers breaks the shared contract.

**Confirmed general rule:** when a struct field's doc comment defers a normalization step to
"the renderer" (or any single named consumer), grep for every OTHER consumer of that field before
trusting the deferred step doesn't matter elsewhere -- a normalization named after one consumer
is a strong signal other consumers may be silently skipping it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Flagged by a background review pass over `tilemap_scene` (task #92); independently re-derived the full 6-file causal chain (`instance.rs`, `scene.rs`, `compile/edges.rs`, `compile/animation.rs`, `compile/frame.rs`, `pipeline.rs`) before filing. |
| 2026-08-16 | fixed | `tick_into` now canonicalizes `Placement::Edge{hex,dir}` via `canonical_edge`/`self.spec.pipeline.hex.tiling` before computing phase, matching the render path; falls back to the raw hex only if canonicalization itself fails. |
| 2026-08-16 | verified | Added `edge_instances_on_opposite_sides_of_same_edge_complete_together` via in-place revert-test-restore: reverted `tick_into`'s `pos` computation back to bare `hex_coord()`, captured the real pre-fix failure (1 of 2 same-edge instances fires instead of both), restored the fix, confirmed passing. Full crate suite (171 tests) + `cargo clippy -p tilemap_scene --all-targets --all-features -- -D warnings` clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass hand-derived the exact edge/direction pair (`(0,0),N` vs `(0,-1),S`) that canonicalizes to the same edge, using `neighbor_offset_by_dir`/`opposite_dir`'s real tables rather than assuming; adversarial pass performed a real in-place revert-test-restore (not a hypothesized transcript) and captured the actual panic output before restoring the fix. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-156 (same review batch, different code path: scene-tick event timing vs. compile-time draw order) -- no cross-dependency. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by reading both diverging call sites directly plus `declared_phase_seconds`'s own explicit "byte-for-byte" doc promise establishing the two paths are meant to agree. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Traced the full `TilingStrategy` field-access chain (`Scene.spec` -> `RenderSpec.pipeline` -> `RenderPipeline.hex` -> `HexConfig.tiling`) directly against struct definitions rather than assuming a path from memory (H3). | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `tilemap_scene` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is a `match` replacing one `let` binding, 2 new imports; no signature/field/public API change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing documented byte-for-byte agreement between render and event-detection paths now actually holds for `Edge` placements. | — |

**Reproduced:** YES -- `edge_instances_on_opposite_sides_of_same_edge_complete_together` was run
against the real unfixed code via in-place revert-test-restore, producing the exact predicted
failure (only 1 of 2 same-edge instances fires); restoring the fix returns the test to passing.
Full crate suite (171 tests) + `cargo clippy -p tilemap_scene --all-targets --all-features -- -D
warnings` clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_scene/src/scene.rs` | `tick_into`'s `pos` computation: added a `match` canonicalizing `Placement::Edge{hex,dir}` via `canonical_edge`/`self.spec.pipeline.hex.tiling` before use, falling back to the raw hex only if canonicalization fails. Added `crate::compile::edges::canonical_edge` and `crate::snapshot::EdgePosition` imports. `Fix(BUG-157)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_scene/tests/scene_events_test.rs` | Added `edge_instances_on_opposite_sides_of_same_edge_complete_together` (`bug_reproducer(BUG-157)`, full doc comment). Added `EdgeDirection` to the crate-root import list. |
