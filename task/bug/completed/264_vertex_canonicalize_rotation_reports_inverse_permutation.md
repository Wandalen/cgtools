# BUG-264: `vertex::canonicalize` reports the inverse rotation permutation, selecting the wrong `{rot}` sprite variant

- **Severity:** Medium (visibly wrong sprite/texture orientation chosen for 2 of every 3 physically
  distinct triangle rotations in any `SpriteSource::VertexCorners` blend pattern whose
  `sprite_pattern` uses `{rot}` -- the entire purpose of the rotation value -- but no panic, no data
  corruption, deterministic per input, confined to one rendering feature)
- **state:** Completed
- **Affects:** `canonicalize` (`src/compile/vertex.rs`), reached via `vertex_pass_compile`
  (`src/compile/frame.rs`) for every dual-mesh triangle using a `{rot}`-parameterised
  `TriBlendPattern`
- **Component:** `module/helper/tilemap_scene` (`src/compile/vertex.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`canonicalize()` in `src/compile/vertex.rs` sorts a triangle's three corner terrain ids
lexicographically and returns a `rotation : u8` meant to record "which original slot landed in
canonical slot 0" (per its own doc comment, and per the SPEC-level docs in
`docs/format/003_anchor_placement_types.md` / `docs/invariant/002_edge_and_vertex_canonical_uniqueness.md`).
The implementation instead computed the position, in the *sorted* array, where original corner-0's
value ended up -- the inverse permutation of the documented contract. Both readings agree only when
`rotation == 0` (the identity/fixed-point case); for either non-identity cyclic rotation of the 3
corners, the reported value is wrong (the two non-zero rotation values are swapped relative to the
documented contract).

## Impact

**Who is affected:** any scene using `SpriteSource::VertexCorners` triangle-blend patterns whose
`sprite_pattern` contains a `{rot}` placeholder -- i.e. any tileset providing distinct sprite art
per triangle orientation (SPEC §5.6 dual-mesh vertex blending). This is the entire reason the
rotation value exists.

**What breaks:** `vertex_pass_compile` (`src/compile/frame.rs`) substitutes
`rotation.to_string()` into `{rot}` to pick the sprite frame id. For 2 of every 3 physically
distinct triangle orientations (the two non-identity cyclic rotations), the wrong sprite frame id
is selected -- a real, pre-allocated frame (so no missing-asset error or panic), just visually the
wrong one, silently swapped with its "mirror" rotation. Every affected triangle in the compiled
scene renders with an incorrect corner-blend orientation.

**Entity Scope:** `None` -- source-level logic defect in a compile-layer pure function, not entity
directory instances.

## How Discovered

During this session's Group K review of
`module/helper/tilemap_scene/src/compile/{animation,assets,camera,conditions,coords,edges,error,frame,ids,mod,neighbors,resolver,vertex,viewport}.rs`,
cross-checking `vertex.rs::canonicalize`'s `rotation` computation against its own doc comment and
the two SPEC-level docs describing the same contract
(`docs/format/003_anchor_placement_types.md`,
`docs/invariant/002_edge_and_vertex_canonical_uniqueness.md`) revealed the code computed the
inverse of what all three descriptions specify. Confirmed by hand-deriving a concrete
counter-example (`["water","grass","sand"]` sorts to `["grass","sand","water"]`; doc-correct
rotation = 1, code returned 2) and confirming no existing test pinned a specific rotation value --
`compile_units_test.rs::canonicalize_sorts_ids` discards `rotation` entirely
(`let ( sorted, _rot ) = ...`), and the one integration assertion touching `{rot}` in
`scene_model_compile_test.rs` accepts *any* of the 3 rotations (`any_rot_emitted`).

## Minimum Reproducible Example

No GL/rendering context needed -- `canonicalize` is a pure function over `[String; 3]`. Call it
with three terrain ids whose lexicographic sort performs a genuine cyclic rotation (not the
identity) and check the returned `rotation` against the documented "original index of the value
now in slot 0" contract. See
`tests/compile_units_test.rs::canonicalize_rotation_reports_forward_permutation`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p tilemap_scene --all-features canonicalize_rotation_reports_forward_permutation
```
**Expected** (fixed): 1 passed. **Actual** (pre-fix, confirmed via temporary swap of `vertex.rs`
for a saved pre-fix copy and rerun): 1 failed -- `assertion \`left == right\` failed: rotation
should report which original corner landed in slot 0 / left: 2 / right: 1`.

## Root Cause

Pre-fix:
```rust
indexed.sort_by( | a, b | a.1.cmp( &b.1 ) );
let rotation = indexed.iter().position( | ( orig, _ ) | *orig == 0 ).unwrap_or( 0 ) as u8;
```
`.position(|(orig,_)| *orig == 0)` searches the *sorted* array for wherever original corner 0's
value ended up -- i.e. "original corner 0 is now at slot N" -- which is the **inverse** of the
documented "which original corner's value is now in slot 0" (`indexed[0].0`). The two readings are
literal inverses of the same permutation; they coincide only at the `rotation == 0` fixed point,
and disagree for either of the two non-identity 3-cycles.

## Why Not Caught

No existing test pinned a specific `rotation` value: `compile_units_test.rs::canonicalize_sorts_ids`
explicitly discards it (`let ( sorted, _rot ) = canonicalize(...)`), and
`scene_model_compile_test.rs`'s only `{rot}`-touching assertion accepts *any* of the 3
rotation-substituted sprite ids (`any_rot_emitted`), by construction unable to distinguish "correct
rotation" from "some in-range rotation." The value stays in the documented `0..3` range either way,
produces no panic, and resolves to a real pre-allocated sprite frame either way -- only a worked
example checked against the doc's exact wording surfaces the swap.

## Fix Applied (2026-08-17)

**`src/compile/vertex.rs`** (`canonicalize`): replaced the `.position()` search with a direct read
of `indexed[ 0 ].0` -- the original index of the corner whose value landed in canonical slot 0,
matching the function's own doc comment and both SPEC-level docs.

**`tests/compile_units_test.rs`** (edited): 1 new test,
`canonicalize_rotation_reports_forward_permutation`, calling `canonicalize` with a concrete
3-corner input where the forward permutation and its inverse diverge, asserting the doc-correct
rotation value.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p tilemap_scene --all-features canonicalize_rotation_reports_forward_permutation`
  -- pre-fix (temporary swap of `vertex.rs` for a saved pre-fix copy): 0 passed, 1 failed
  (`left: 2, right: 1`). Post-fix (restored): 1 passed, 0 failed.
- `cargo test -p tilemap_scene --all-features canonicalize` (combined scoped run, post-fix):
  `compile_units_test.rs` -- 2 passed (`canonicalize_rotation_reports_forward_permutation`,
  `canonicalize_sorts_ids`), 0 failed -- confirming the fix does not disturb the pre-existing
  pinned test.
- `cargo clippy -p tilemap_scene --all-targets --all-features -- -D warnings`: clean (see final
  scoped verification run below).

## Generalized Version

**Broken assumption:** a scalar "rotation/permutation index" derived by searching a sorted array
for where a *fixed reference point* ended up is not automatically the same value as "which original
element is now at a *fixed slot*" -- these are inverse permutations of each other, both valid-looking
`u8`s in the same documented range, and only coincide at the identity/fixed-point case. Whenever a
function's doc comment describes a permutation in words ("which original slot landed in slot N"),
verify the implementation reads out the permutation in that exact direction rather than its
inverse -- and pin a concrete non-identity example in a test, since an identity-only or "any valid
value" test cannot distinguish a permutation from its inverse.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group K review of `tilemap_scene::compile::{animation,assets,camera,conditions,coords,edges,error,frame,ids,mod,neighbors,resolver,vertex,viewport}`. Root cause: `canonicalize`'s `rotation` was computed as the *inverse* of the documented permutation (searching where original corner 0 ended up in the sorted array, rather than reading out which original corner is now in slot 0) -- correct only at the `rotation == 0` fixed point, wrong for both non-identity cyclic rotations, silently selecting the wrong `{rot}`-substituted sprite variant in `vertex_pass_compile`. Fixed by reading `indexed[ 0 ].0` directly. Verified via 1 new native unit test (confirmed fail pre-fix via temporary revert-and-rerun / pass post-fix), a combined scoped rerun confirming no regression on the pre-existing `canonicalize_sorts_ids` pin, and clean clippy. Filed as BUG-264, not BUG-263, after a fresh on-disk ID re-scan immediately before filing found a concurrent session actor had already claimed 263 (`catalog_builder_duplicate_missing_state_double_reported`) between this session's earlier scan and this filing -- the in-source fix comment was renumbered from BUG-263 to BUG-264 accordingly before filing. Closed same-session (Tier 2 Dual-Role Self-Check). |
