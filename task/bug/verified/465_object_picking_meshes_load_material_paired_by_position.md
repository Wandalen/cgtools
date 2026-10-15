# BUG-465: `object_picking`'s `meshes_load` pairs models with materials by iteration position instead of `material_id`

- **Severity:** Medium (no crash -- wrong textures/materials silently applied, or trailing models
  silently dropped, depending on the loaded `.obj`/`.mtl` asset's own material ordering)
- **state:** Verified
- **Affects:** `examples/minwebgl/object_picking`'s mesh loader, for any `.obj`/`.mtl` asset whose
  materials are not both (a) listed in exactly the same order the models reference them and (b)
  exactly as many materials as models, with every model referencing exactly one.
- **Component:** `examples/minwebgl/object_picking` (`src/main.rs`, `meshes_load`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Fix Task:** [509](../../verifying/509_register_object_picking_meshes_load_material_pairing_fix_closes_bug465.md)

## Symptom

```rust
// pre-fix -- src/main.rs, meshes_load
async fn meshes_load( models : &[ tobj::Model ], materials : &[ tobj::Material ], gl : &GL ) -> Vec< Mesh >
{
  let mut meshes = vec![];
  for ( model, material ) in models.iter().zip( materials )
  {
    // ... uses `material.diffuse_texture` for this model, paired purely by position
  }
}
```

`tobj::Model.mesh.material_id : Option<usize>` is the actual index into `materials` that a model
references -- it is not guaranteed to equal the model's own position in the `models` slice.
`.zip( materials )` pairs the Nth model with the Nth material regardless of what `material_id`
actually says, and additionally truncates silently to the shorter of the two iterators whenever
`materials.len() < models.len()`, dropping trailing models from the returned `Vec<Mesh>` entirely.

## Impact

**Who is affected:** Any `.obj`/`.mtl` asset loaded through `meshes_load` where materials aren't
already known to be co-indexed with models by construction (matching order, one-to-one, no gaps).

**What breaks:** Silently wrong textures applied to one or more meshes (whichever model's
`material_id` doesn't match its own position gets some other model's material instead), or -- if
`materials.len() < models.len()` -- trailing models silently missing from the render entirely, with
no error or warning either way.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `examples/minwebgl/object_picking`, comparing
`meshes_load`'s pairing strategy against the equivalent, correctly-implemented pattern in the
sibling `obj_viewer` example (`src/mesh.rs`'s `GLMesh::from_tobj_model`, which matches via
`model.mesh.material_id` with an explicit bounds check and fallback).

## Manual Reproduction / Verification

No dedicated automated MRE test was added -- this defect requires a real `.obj`/`.mtl` asset with
non-identity model/material ordering and a WebGL context to observe (texture selection is only
meaningful once uploaded and rendered), consistent with this sweep's granted exception for example
crates. Verified instead by:

1. Hand-tracing a hypothetical 3-model, 2-material asset where model 0 references
   `material_id = Some(1)` and model 2 has no material (`material_id = None`) -- pre-fix,
   `.zip( materials )` would pair model 0 with `materials[0]` (wrong -- should be `materials[1]`)
   and silently drop model 2 entirely (shorter iterator wins); post-fix, model 0 correctly resolves
   `materials[1]` via its own `material_id`, and model 2 correctly falls back to `None` (no
   texture) instead of being dropped from the output `Vec<Mesh>`.
2. `cargo check -p object_picking --target wasm32-unknown-unknown` -- clean, no errors.

**Verify Command:**
```bash
cd examples/minwebgl/object_picking && cargo check --target wasm32-unknown-unknown
```

## Root Cause

`models.iter().zip( materials )` assumes the Nth model always uses the Nth material, which only
holds when the `.obj`/`.mtl` pair happens to list both in matching order with no gaps -- the
correct pairing key, `tobj::Model.mesh.material_id`, was available but unused.

## Why Not Caught

The demo's own bundled `.obj` asset (`static/cat/...`, per the texture path referenced in
`meshes_load`) apparently happens to list materials in an order that coincides with position-based
pairing closely enough to look correct, so the defect had no visible symptom against that specific
asset -- it would only surface with a differently-ordered or partially-material-less asset.

## Fix Location

`examples/minwebgl/object_picking/src/main.rs`, `meshes_load`: replaced
`for ( model, material ) in models.iter().zip( materials )` with `for model in models` plus an
explicit `let material = model.mesh.material_id.and_then( | id | materials.get( id ) );` lookup;
the texture-loading branch now reads
`material.and_then( | m | m.diffuse_texture.as_ref() )` instead of `&material.diffuse_texture`,
correctly falling back to `None` (no texture) for a model with no material or an out-of-range id,
matching the reference pattern in `obj_viewer/src/mesh.rs`'s `GLMesh::from_tobj_model`.

## Prevention

None added beyond the fix itself and the wasm32 compile check, per this sweep's exception for
example crates -- the fix itself is now structurally aligned with `obj_viewer`'s own established,
correct pattern for the identical `tobj::Model`/`materials` pairing problem, which is the most
direct prevention available without introducing asset-loading test scaffolding this crate does not
have.

## Pitfall

Pairing two same-length-looking slices by iteration position instead of by an explicit id/foreign-
key field is a silent correctness bug whenever the two lists aren't already *known* to be
co-indexed by construction -- prefer the explicit id lookup even when position-pairing happens to
work for the current asset, since a future asset swap (or even just re-exporting the same asset
from different tooling) can silently break it with no compiler or runtime signal.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX sweep of `examples/minwebgl/object_picking`, cross-checked against the sibling `obj_viewer` example's correct pattern for the same problem. |
| 2026-08-20 | fixed | Replaced position-based `.zip` pairing with an explicit `material_id`-based lookup; documented with `Fix(BUG-465)`/`Root cause`/`Pitfall`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Fix correctness (hand-trace + compile + reference-pattern match) | — | 🟢 | Adversarial pass: specifically checked the `None`/out-of-range `material_id` fallback path (not just the "reordered but valid" case) against `obj_viewer`'s own reference pattern, confirming the fix's `.and_then` chain degrades to `None` (no texture) rather than panicking or defaulting incorrectly. `cargo check -p object_picking --target wasm32-unknown-unknown` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-465)`/`Root cause`/`Pitfall` 3-field format applied at the fix site, cross-referencing the `obj_viewer` reference pattern by file path. | — |

**Reproduced:** Confirmed via hand-trace of a hypothetical non-identity-ordered asset against the
pre-fix code (not a live browser render against a crafted asset -- see Manual Reproduction /
Verification for why an automated MRE was not added). 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/object_picking/src/main.rs` | `meshes_load`: replaced `.zip( materials )` position-pairing with `model.mesh.material_id`-based lookup; texture-loading branch updated for the new `Option<&tobj::Material>`. `Fix(BUG-465)`/`Root cause`/`Pitfall` comment. |
