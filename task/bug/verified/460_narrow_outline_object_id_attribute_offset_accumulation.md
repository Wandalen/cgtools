# BUG-460: `narrow_outline`'s `attributes_add` only produces correct per-vertex object ids for the first mesh in a multi-mesh scene

- **Severity:** Medium (no crash -- only affects scenes with more than one mesh; every mesh after
  the first silently renders with wrong per-vertex object ids)
- **state:** Verified
- **Affects:** `examples/minwebgl/narrow_outline`'s object-id-driven rendering (per-object flat
  color via `u_object_colors[ uint( v_object_id ) ]`, confirmed in `object.frag`) for any loaded
  glTF scene with 2+ meshes.
- **Component:** `examples/minwebgl/narrow_outline` (`src/main.rs`, `attributes_add`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Fix Task:** [504](../../verifying/504_register_narrow_outline_object_id_attribute_offset_fix_closes_bug460.md)

## Symptom

```rust
// pre-fix -- src/main.rs, attributes_add
let mut object_vertex_count = 0; // declared OUTSIDE the mesh loop
for ( object_id, mesh ) in ( 1.. ).zip( gltf.meshes.iter() )
{
  for primitive in &mesh.borrow().primitives { /* object_vertex_count += ... */ }
  object_id_data.extend( vec![ object_id; object_vertex_count ] );
}
```

`object_vertex_count` never resets between meshes, so it accumulates: mesh 2's `extend` call
writes `(mesh_1_count + mesh_2_count)` copies of `object_id = 2`, not just `mesh_2_count` -- every
mesh after the first inflates `object_id_data`'s total length beyond the scene's real total vertex
count. Separately, the single `object_id_info` attribute descriptor was built once with a
hardcoded `offset = 0` and reused, byte-for-byte identical, for every mesh's `attribute_add` call
-- so every mesh's vertex shader read starting from the very beginning of the (now-corrupted)
buffer, regardless of where that mesh's own data actually landed.

## Impact

**Who is affected:** Any scene with 2+ meshes loaded through `attributes_add`. A single-mesh scene
is unaffected -- the accumulator's un-reset value coincidentally equals the one mesh's own count,
and offset 0 is correct because there is only one segment.

**What breaks:** Meshes after the first read back the wrong slice of `object_id_data` (effectively
mesh 1's ids, or an out-of-bounds/misaligned read once the corrupted buffer's layout diverges far
enough), which the fragment shader (`object.frag`) uses to index `u_object_colors` per pixel --
so every mesh but the first renders with the wrong per-object flat color/outline id instead of its
own.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `examples/minwebgl/narrow_outline`, reading
`attributes_add` end to end and hand-tracing the accumulator and descriptor across a hypothetical
2-mesh scene.

## Manual Reproduction / Verification

No dedicated automated MRE test was added for this fix -- this logic is inseparable from
`gl`/`gltf::Gltf`/`Rc<RefCell<Geometry>>` (a real WebGL context and a loaded glTF document), so a
native reproducer would require substantial WASM/browser test scaffolding this crate does not have,
consistent with this sweep's granted exception for example crates where real automated testing
isn't practical. Verified instead by:

1. Hand-tracing `attributes_add` against a hypothetical 2-mesh scene (mesh A: 100 vertices, mesh B:
   50 vertices) both before and after the fix -- pre-fix, `object_id_data.len()` would be
   `100 + 150 = 250` (not the correct `150`), and both meshes' `object_id_info` descriptors would
   be byte-identical (`offset = 0`); post-fix, `object_id_data.len() == 150` and mesh B's
   descriptor has `offset = 100` (mesh A's own vertex count), matching where mesh B's data
   actually starts in the shared buffer.
2. `cargo check -p narrow_outline --target wasm32-unknown-unknown` -- clean, no errors.

**Verify Command:**
```bash
cd examples/minwebgl/narrow_outline && cargo check --target wasm32-unknown-unknown
```

## Root Cause

Two compounding defects in the same function: (1) `object_vertex_count` was declared once outside
the mesh loop instead of being reset to `0` at the top of each iteration, so it accumulated a
running total instead of holding "this mesh's own vertex count"; (2) the `object_id_info`
attribute descriptor was built once, before the per-mesh loop that attaches it, with a hardcoded
`offset = 0`, and that single descriptor instance was cloned unchanged onto every mesh's geometry
instead of being rebuilt per mesh with that mesh's own running offset into the shared buffer.

## Why Not Caught

This crate has no existing test coverage for `attributes_add`, and the demo's own bundled scene
happens to be single-mesh (or effectively exercises only the first mesh's coloring visibly), so
the accumulation bug never manifested as an obviously wrong render in casual manual testing --
it only surfaces with a multi-mesh glTF scene.

## Fix Location

`examples/minwebgl/narrow_outline/src/main.rs`, `attributes_add`: (1) `object_vertex_count` moved
inside the mesh loop (reset to `0` each iteration) and each mesh's own count recorded into a new
`mesh_vertex_counts : Vec< usize >`; (2) the single shared descriptor build replaced with a second
loop that builds one `object_id_info` per mesh, offset by a running `object_offset : i32`
accumulated from `mesh_vertex_counts`, incremented after each mesh's `attribute_add` calls.

## Prevention

None added beyond the fix itself and the wasm32 compile check -- per this sweep's exception for
example crates, a full native regression harness for GL-context-dependent per-vertex attribute
data was judged impractical relative to the fix's scope. A future contributor extracting the
offset/count bookkeeping into a pure, GL-independent helper (taking `&[usize]` mesh vertex counts,
returning `Vec<i32>` offsets) would make this specific invariant natively unit-testable.

## Pitfall

An accumulator that must reset per loop iteration has to be *declared* inside the loop body --
declaring it once outside the loop looks identical at a glance but silently changes "this
iteration's own count" into "the running total so far". Likewise, an attribute descriptor built
once and reused unchanged across a per-item loop only stays correct if every item's layout is
identical (offset, stride, etc.) -- here it wasn't, since each mesh occupies its own distinct
segment of the same shared buffer.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX sweep of `examples/minwebgl/narrow_outline`. |
| 2026-08-20 | fixed | Per-mesh vertex count tracking and per-mesh offset-aware descriptor construction applied; documented with `Fix(BUG-460)`/`Root cause`/`Pitfall`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Fix correctness (hand-trace + compile) | — | 🟢 | Adversarial pass: re-derived the pre-fix behavior algebraically for a 2-mesh scene (see Manual Reproduction / Verification) to confirm the described defect actually reproduces the stated symptom, not just a plausible-sounding story; confirmed `cargo check -p narrow_outline --target wasm32-unknown-unknown` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-460)`/`Root cause`/`Pitfall` 3-field format applied at both edit sites in `attributes_add`, matching this workspace's established source-comment convention. | — |

**Reproduced:** Confirmed via algebraic hand-trace against the pre-fix code (not a live browser
render -- see Manual Reproduction / Verification for why an automated MRE was not added). 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/narrow_outline/src/main.rs` | `attributes_add`: per-mesh vertex-count reset and tracking (`mesh_vertex_counts`), per-mesh offset-aware `object_id_info` descriptor construction (`object_offset`), `Fix(BUG-460)`/`Root cause`/`Pitfall` comments. |
