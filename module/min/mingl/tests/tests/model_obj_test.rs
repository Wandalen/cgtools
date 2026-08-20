//! Verifies `model::obj::num_faces_compute`'s face-count derivation: `face_arities`-driven
//! count when it is populated, and an `indices.len() / 3` fallback when `face_arities` is
//! empty -- per `tobj` 4.0.5's own documented contract, `face_arities` is empty whenever a mesh
//! was loaded with the `triangulate` option (e.g. `tobj::GPU_LOAD_OPTIONS`, used by
//! `examples/minwebgl/obj_load`) or was already natively all-triangles. See BUG-273.

use super::*;
use the_module::model::obj::num_faces_compute;

#[ test ]
fn nonuniform_face_arities_are_counted_directly()
{
  // 2 faces: a quad (arity 4) and a triangle (arity 3). `indices` content is irrelevant to
  // this branch -- only `face_arities.len()` is used when it is non-empty.
  let face_arities = [ 4u32, 3u32 ];
  let indices = [ 0u32, 1, 2, 3, 4, 5, 6 ];
  assert_eq!( num_faces_compute( &face_arities, &indices ), 2 );
}

// test_kind: bug_reproducer(BUG-273)
/// ## Root Cause
/// `ReportObjModel::new` computed `num_faces` as `mesh.face_arities.len()` directly. Per
/// `tobj` 4.0.5's own doc comments, `Mesh::face_arities` is *empty* whenever
/// `LoadOptions::triangulate` was set (as `tobj::GPU_LOAD_OPTIONS` -- used by the real
/// `examples/minwebgl/obj_load` example -- always sets it) or the source mesh was already
/// natively all-triangles; in both cases `Mesh::indices` still holds the real, non-empty
/// triangle data (3 indices per face). Reading `face_arities.len()` in that state always
/// yields `0`, silently misreporting "zero faces" for meshes that can have thousands.
///
/// ## Why Not Caught
/// No test in this crate ever exercised `model::obj`'s report-building logic at all --
/// `ReportObjModel`/`reports_make` require a live `tobj::Model`, which the previous test
/// suite never constructed. The only place this path runs for real is
/// `examples/minwebgl/obj_load`, which merely logs the (silently wrong) report via
/// `gl::log::info!` -- nothing asserts on it, so a `0` face count for a model with visible
/// geometry never surfaced as a visible failure.
///
/// ## Fix Applied
/// Extracted the face-count logic into `num_faces_compute( face_arities, indices )`: returns
/// `face_arities.len()` when non-empty (unchanged behavior), and
/// `indices.len() / 3` when `face_arities` is empty (the fix -- every face is known to be a
/// triangle in that state, so its index count divided by 3 is the real face count).
/// `ReportObjModel::new`'s `num_of_arities` branch, which used to key off the
/// now-decoupled `num_faces == 0`, was updated to key off `mesh.face_arities.is_empty()`
/// directly so its own "assume arity 3" behavior is unchanged by this fix.
///
/// ## Prevention
/// This test drives the exact empty-`face_arities`-with-real-`indices` state a triangulated
/// `tobj` load produces and asserts the derived count matches `indices.len() / 3`, not `0`.
///
/// ## Pitfall
/// An external crate leaving a field empty as a documented "assume this default instead"
/// signal is not the same as that field being a valid, readable count of anything -- never
/// read `.len()` off a field without first checking whether emptiness means "zero" or "look
/// elsewhere for the real value" in that crate's own contract.
#[ test ]
fn empty_face_arities_derives_count_from_indices()
{
  // Simulates a tobj-triangulated mesh ( `LoadOptions::triangulate == true`, e.g. via
  // `tobj::GPU_LOAD_OPTIONS` ) or a mesh that is natively all-triangles: `face_arities` is
  // empty in both cases, so the true face count must come from `indices.len() / 3`.
  let face_arities : [ u32 ; 0 ] = [];
  let indices = [ 0u32, 1, 2, 0, 2, 3 ]; // 2 triangles => 6 indices
  assert_eq!( num_faces_compute( &face_arities, &indices ), 2 );

  // RED state (empirically confirmed): reverting the fix to plain `face_arities.len()` makes
  // this assertion fail with `0`, not `2` -- the exact silent-zero symptom this bug reports.
}

#[ test ]
fn empty_mesh_reports_zero_faces()
{
  // A genuinely empty mesh ( no faces at all ) must still report 0 -- the fix must not turn
  // "no data" into a spurious non-zero count.
  let face_arities : [ u32 ; 0 ] = [];
  let indices : [ u32 ; 0 ] = [];
  assert_eq!( num_faces_compute( &face_arities, &indices ), 0 );
}
