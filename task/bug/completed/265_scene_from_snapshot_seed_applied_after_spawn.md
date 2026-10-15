# BUG-265: `Scene::from_snapshot` applies the snapshot's `seed` after spawning
every instance, so `PhaseOffset::Instance` salts use the default seed instead
of the declared one

- **Severity:** Medium (silently wrong output, not a panic -- every snapshot-loaded scene with a
  non-default `seed` and any `PhaseOffset::Instance` layer gets the wrong stagger distribution)
- **state:** Completed
- **Affects:** `tilemap_scene::Scene::from_snapshot`
- **Component:** `module/helper/tilemap_scene` (`src/scene.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`Scene::from_snapshot` spawns every tile / edge / multihex / free / viewport / entity instance
from the snapshot in a sequence of loops, and only afterward applies `snap.seed` via
`scene.seed_set(seed)`. `Scene::spawn`, however, reads `self.seed` synchronously -- during each of
those earlier spawn calls -- to derive the instance's `instance_phase_seed` (`spawn`'s own doc
comment: "Mixed with the scene seed so re-seeded scenes get a different distribution"). Because
`seed_set` runs after every spawn has already completed, every instance in a snapshot-loaded scene
is salted with `Scene::new`'s default seed (`0`), never the snapshot's declared `seed`, regardless
of what `snap.seed` actually holds.

## Impact

**Who is affected:** Any caller loading a scene via `Scene::from_snapshot` (the crate's own
documented snapshot-materialization entry point) with a `SceneSnapshot.seed` set to a non-default
value and at least one object using `PhaseOffset::Instance` on any layer.

**What breaks:** `PhaseOffset::Instance` is documented (`src/resource.rs`) as "a per-instance seed
stamped at `Scene::spawn`... mixed with the scene seed so re-seeded scenes get a different
distribution" -- its entire purpose is to let a game re-roll the animation-phase stagger of
freely-positioned / viewport-anchored instances (which have no grid coordinate for
`HashCoord`/`Linear` to key off) by changing the scene's seed. Under this bug, every scene loaded
from a snapshot with an explicit `seed` silently gets the *same* stagger distribution as an
unseeded (seed `0`) scene -- the re-seeding has zero effect on `instance_phase_seed`, even though
`VariantSelection::Random` (which reads `scene.seed()` live at render/compile time, not stamped at
spawn) *does* correctly respond to the same `seed` field. This asymmetry means a snapshot author
setting `seed` to get two visibly different results only gets partial reseeding: `Random` variant
selection changes, but `PhaseOffset::Instance` staggering silently does not.

**Entity Scope:** `None` -- source-level logic defect, not entity directory instances.

## How Discovered

During this session's review of `module/helper/tilemap_scene/src/{load,pipeline,renderer,
resource,scene,snapshot,source,spec,validate}.rs` for functional bugs (`compile/*.rs` and the 10
core-data files were out of scope, owned by concurrent review groups). `scene.rs`'s
`Scene::spawn` doc comment explicitly promises `instance_phase_seed` is "mixed with the scene
seed so re-seeded scenes get a different distribution" -- cross-checking that promise against
`Scene::from_snapshot`'s actual call order (all spawn loops, *then* `seed_set`) showed the promise
cannot hold for any snapshot-loaded scene. Confirmed via `grep -rn "\.seed()\|instance_phase_seed"
src/compile/*.rs` that `instance_phase_seed` is read only at spawn time (`compile/frame.rs:905`,
`:1257`) while `scene.seed()` is additionally read live at render time (`compile/frame.rs:641`),
confirming the two are genuinely different mechanisms and this is not a false alarm from
`Random`'s already-correct live-read behavior. Confirmed via `grep -rn "seed"
tests/scene_model_test.rs tests/scene_model_compile_test.rs` that the only existing seed test
(`variant_random_deterministic_across_frames`) exercises `VariantSelection::Random` only, leaving
this exact path (`from_snapshot` + `PhaseOffset::Instance` + custom seed) unpinned by any test.
Also checked `docs/invariant/004_deterministic_compilation.md` and `docs/pitfall/*.md` for any
existing acknowledgement of this specific ordering gap -- found none, confirming this is a fresh
finding, unlike a separate `source.rs` doc-comment/`validate.rs` gap noted during the same review
(composite-source nesting validation), which the project's own `docs/invariant/001` and
`roadmap.md` "Polish items" already explicitly track as deliberately-deferred future work and was
therefore left unactioned by this review.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tilemap_scene --all-features from_snapshot_applies_seed_before_spawn
```
**Expected** (fixed): the test passes -- an instance spawned via `Scene::from_snapshot` with
`snap.seed = Some(seed)` gets the identical `instance_phase_seed` as the same object spawned
directly after calling `seed_set(seed)` up front. **Actual** (pre-fix, confirmed via temporary
direct-source-edit revert-and-rerun of `from_snapshot`'s seed-application order back to "after all
spawn loops"): the test failed --
`assertion `left == right` failed: from_snapshot must apply snap.seed before spawning so
instance_phase_seed matches a scene seeded up front` (`left: 1721727358, right: 4226464507`).

## Root Cause

`Scene::from_snapshot` (pre-fix, verbatim order):
```rust
pub fn from_snapshot( snap : &SceneSnapshot, spec : Arc< RenderSpec > ) -> Result< Self, SnapshotLoadError >
{
  let mut scene = Self::new( spec );          // scene.seed == 0 here

  for tile in tiles_iter { /* ... */ scene.spawn( obj, Placement::Hex { q, r } ); }
  for inst in &snap.edges { /* ... */ scene.spawn( obj, Placement::Edge { .. } ); }
  for inst in &snap.multihex_instances { /* ... */ scene.spawn( obj, Placement::Multihex { .. } ); }
  for inst in &snap.free_instances { /* ... */ scene.spawn( obj, Placement::FreePos { .. } ); }
  for inst in &snap.viewport_instances { /* ... */ scene.spawn( obj, Placement::Viewport ); }
  for ent in &snap.entities { /* ... */ scene.spawn( obj, Placement::Hex { .. } ); }
  // every instance above stamped its instance_phase_seed using scene.seed == 0

  if let Some( tint_id ) = snap.initial_global_tint.as_ref() { scene.global_tint_set( .. ); }
  if let Some( seed ) = snap.seed { scene.seed_set( seed ); }   // too late -- nothing left to salt

  Ok( scene )
}
```
And `Scene::spawn` (unchanged by this fix):
```rust
pub fn spawn( &mut self, object : ObjectHandle, placement : Placement ) -> InstanceHandle
{
  let raw_seed = self.next_phase_seed;
  self.next_phase_seed = self.next_phase_seed.wrapping_add( 1 );
  let scene_salt = ( self.seed as u32 ) ^ ( ( self.seed >> 32 ) as u32 );   // reads self.seed NOW
  let instance_phase_seed = crate::hash::coord_hash( raw_seed as i32, 0, scene_salt ^ 0x9E37_79B9 );
  // ...
}
```
The setter calls at the bottom of `from_snapshot` were grouped together, in snapshot-field order
(`initial_global_tint` then `seed`), rather than in the data-dependency order `spawn` actually
requires (`seed` must precede any `spawn` call; `global_tint_set` has no such dependency since
`global_tint_override` is read only at render time, never at spawn).

## Why Not Caught

The only existing seed-focused test, `variant_random_deterministic_across_frames`
(`tests/scene_model_compile_test.rs`), exercises `VariantSelection::Random`, which reads
`Scene.seed` *live* at compile time via `scene.seed()` -- it therefore passes regardless of when
`seed_set` runs relative to `spawn`, and never observes a stamp-once-at-spawn ordering bug.
`instance_phase_seed` is the only seed-derived quantity that is computed once, inside `spawn`,
and no prior test compared it against a reference scene seeded before spawning.

## Fix Applied (2026-08-17)

**`src/scene.rs`:** Moved the `if let Some(seed) = snap.seed { scene.seed_set(seed); }` block in
`Scene::from_snapshot` from after all six spawn loops to immediately after `Self::new`, before any
spawn call. `global_tint_set`'s call site was left in its original position (after the spawn
loops) since `global_tint_override` is read only at render time, not at spawn -- its ordering
relative to `spawn` has no observable effect, so moving it would not be a functional fix.

**`tests/scene_model_compile_test.rs`** (new test in the existing file, alongside
`variant_random_deterministic_across_frames`): `from_snapshot_applies_seed_before_spawn` spawns
the same object twice -- once through `Scene::from_snapshot` with `seed` set on the snapshot, once
by calling `seed_set` manually before a direct `spawn` on a freshly-built `Scene` -- and asserts
the two instances receive an identical `instance_phase_seed`.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p tilemap_scene --all-features from_snapshot_applies_seed_before_spawn` -- pre-fix
  (temporary direct-source-edit revert of `from_snapshot`'s seed-application order, restored
  immediately after): test failed (`left: 1721727358, right: 4226464507`). Post-fix (restored):
  test passed.
- `cargo test -p tilemap_scene --all-features` (full scoped suite): all tests pass.
- `cargo clippy -p tilemap_scene --all-targets --all-features -- -D warnings`: clean.

## Generalized Version

**Broken assumption:** grouping a constructor's optional-field setter calls together at the end of
a function, in the same order the fields appear on the source struct, is safe because "they're
just setters." It isn't, when one of those setters seeds mutable state that an *earlier* step in
the same function already consumed to compute a value it stamps once and never revisits. The
compiler enforces nothing here -- `self.seed` is valid (defaulted to `0`) at every point in the
function, so no borrow-checker or type error signals the gap; the only symptom is silently
plausible-looking output computed from the wrong input. Same generalized lesson as BUG-259
(a doc comment is state that can go stale with zero compiler signal) applied to intra-function
ordering instead of documentation: when a value is stamped once from currently-live state, every
setter that state depends on must run before the stamp, not merely "before the function returns."

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's review of `module/helper/tilemap_scene/src/{load,pipeline,renderer,resource,scene,snapshot,source,spec,validate}.rs` (9 files; `compile/*.rs` and 10 core-data files out of scope, owned by concurrent review groups) -- all 9 files clean except this one. Root cause: `Scene::from_snapshot` applied `snap.seed` via `seed_set` after all spawn loops, but `Scene::spawn` reads `self.seed` synchronously per-instance to derive `instance_phase_seed`, so every snapshot-loaded instance was salted with the default seed (`0`) instead of the snapshot's declared one. Fixed by moving the `seed_set` call to before the spawn loops. Verified via a new `scene_model_compile_test.rs` test (confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun) plus the combined scoped suite and clean clippy. Filed as BUG-265 (not the provisionally-scanned BUG-263, nor BUG-264) after two successive fresh on-disk scans found concurrent review groups had already claimed BUG-263 (`CatalogBuilder`, same crate) and then BUG-264 (`vertex::canonicalize`, `tilemap_renderer`) before this bug's own report file could be written; renumbered this bug's own `Fix( BUG-NNN )` source comment from 263 to 264 and then to 265 accordingly. A separate finding from the same review -- `src/source.rs`'s module doc comment falsely claiming composite-source nesting "is an error caught at validation time" when `validate.rs` does not yet implement that check -- was deliberately left unactioned: the project's own `docs/invariant/001_renderspec_referential_integrity.md` and `roadmap.md` "Polish items" already explicitly document and track that exact gap as deferred future work, so it is pre-existing catalogued debt rather than a fresh defect within this review's remit. |
