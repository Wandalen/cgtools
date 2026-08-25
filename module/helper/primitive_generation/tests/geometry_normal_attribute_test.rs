//! Integration tests for the `normals` field of `AttributesData`, populated
//! by all three geometry generators (`curve_to_geometry`,
//! `contours_to_fill_geometry`, `plane_to_geometry`).
//!
//! Covers the BUG-217 fix: none of these functions ever populated a normal
//! attribute at all, and `renderer`'s `PbrMaterial` vertex shader
//! unconditionally reads and `normalize()`s a required `layout(location=1)
//! in vec3 normal` attribute -- an unbound attribute reads WebGL's default
//! `(0,0,0)`, and `normalize` of the zero vector is `NaN` in every
//! component, corrupting lighting for every primitive this crate generates.
//! This corruption happens entirely on the GPU/shader side and is outside
//! what a native `cargo nextest` run can observe directly (see the
//! `Wasm Native-Check Blind Spot` awareness already established for this
//! workspace) -- so these tests instead verify the testable, native-side
//! contract the fix actually establishes: every generator populates one
//! finite, unit-length, correctly-oriented normal per vertex.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::{ curve_to_geometry, plane_to_geometry };

  #[ cfg( feature = "font-processing" ) ]
  use primitive_generation::contours_to_fill_geometry;

  /// ## Root Cause
  /// `AttributesData` had no `normals` field at all, and none of
  /// `curve_to_geometry`/`contours_to_fill_geometry`/`plane_to_geometry`
  /// ever computed one. `primitives_data_to_gltf` (`src/primitive_data.rs`)
  /// consequently never created a normal buffer or bound a `"normal"`
  /// attribute, yet `renderer`'s `PbrMaterial` vertex shader
  /// (`module/helper/renderer/src/webgl/shaders/main.vert`) unconditionally
  /// computes `vNormal = normalize( normalMatrix * normal )` from a required
  /// `layout( location = 1 ) in vec3 normal` attribute. An attribute slot
  /// that is never bound reads WebGL's spec-mandated default value
  /// `(0, 0, 0, 1)`, so the geometric `normal` read as `vec3( 0, 0, 0 )`,
  /// and `normalize` of the zero vector is `NaN` in every component
  /// ( `dot( 0, 0 ) == 0`, `inversesqrt( 0 ) == +Inf`, `0 * Inf == NaN` per
  /// IEEE-754 ) -- corrupting every downstream lighting calculation for
  /// every primitive this crate generates.
  ///
  /// ## Why Not Caught
  /// No existing test inspected `AttributesData.normals` because the field
  /// did not exist -- there was nothing to assert on. The NaN itself only
  /// manifests inside the GPU/shader pipeline, which native `cargo nextest`
  /// runs cannot observe (see the workspace's established Wasm Native-Check
  /// Blind Spot: a green native check proves nothing about wasm/shader-gated
  /// code) -- so no native test could have caught the downstream symptom
  /// even if one had been written; only the missing-attribute root cause is
  /// testable at this layer.
  ///
  /// ## Fix Applied
  /// Added `AttributesData::normals : Vec< [ f32; 3 ] >`, populated it in
  /// all three geometry generators (`src/primitive.rs`), and wired a second
  /// `"normal"` buffer/attribute at slot 1 into `primitives_data_to_gltf`
  /// (`src/primitive_data.rs`), matching `main.vert`'s
  /// `layout( location = 1 )`. Each generator's normal is derived from its
  /// own actual triangle winding rather than assumed: `plane_to_geometry`
  /// and `curve_to_geometry` have fixed, direction-independent windings
  /// (`(0,0,1)` and `(0,0,-1)` respectively, each verified algebraically);
  /// `contours_to_fill_geometry`'s winding depends on caller-supplied
  /// contour data, so its normal is computed at runtime from each body's
  /// own first triangle, falling back to `(0,0,1)` for a degenerate or
  /// absent triangle to avoid reintroducing NaN via `normalize` of a zero
  /// vector.
  ///
  /// ## Prevention
  /// A shader that unconditionally reads and normalizes a vertex attribute
  /// gives no signal (no error, no panic) when the geometry producer never
  /// bound that attribute -- any new geometry generator feeding
  /// `primitives_data_to_gltf` must populate every attribute its target
  /// shader unconditionally reads, not only the ones the generator's own
  /// author happened to think about.
  ///
  /// ## Pitfall
  /// Flat/coplanar geometry still needs its normal's *sign* derived from
  /// the function's actual triangle winding, not assumed by symmetry with
  /// another function -- `plane_to_geometry` and `curve_to_geometry` face
  /// opposite directions despite both being flat Z=0 geometry, because
  /// their hardcoded/derived windings differ.
  #[ test ]
  fn plane_to_geometry_normals_face_positive_z()
  {
    let primitive = plane_to_geometry().expect( "plane_to_geometry always succeeds" );
    let attributes = primitive.attributes.expect( "geometry must have attributes" );
    let positions_len = attributes.borrow().positions.len();
    let normals = attributes.borrow().normals.clone();

    assert_eq!
    (
      normals.len(), positions_len,
      "normals must be parallel to positions -- one normal per vertex"
    );
    for [ x, y, z ] in normals
    {
      assert!
      (
        ( x - 0.0 ).abs() < 1e-5 && ( y - 0.0 ).abs() < 1e-5 && ( z - 1.0 ).abs() < 1e-5,
        "plane_to_geometry's 0,1,2 winding is CCW as seen from +Z, so every \
        normal must be (0,0,1); got [{x}, {y}, {z}]"
      );
    }
  }

  /// Regression guard for `curve_to_geometry`'s side of the BUG-217 fix --
  /// see `plane_to_geometry_normals_face_positive_z`'s doc comment for the
  /// full Root Cause / Fix Applied / Prevention writeup shared by all three
  /// generators. This function's winding is direction-independent (proven
  /// algebraically for any unit segment direction while designing the fix),
  /// so every stroked segment must emit the constant `(0,0,-1)` normal
  /// regardless of the curve's shape.
  #[ test ]
  fn curve_to_geometry_normals_face_negative_z()
  {
    let curve =
    [
      [ 0.0_f32, 0.0_f32 ],
      [ 1.0_f32, 0.0_f32 ],
      [ 1.0_f32, 1.0_f32 ],
    ];
    let primitive = curve_to_geometry( &curve, 0.1 ).expect( "a valid 3-point curve must produce geometry" );
    let attributes = primitive.attributes.expect( "geometry must have attributes" );
    let positions_len = attributes.borrow().positions.len();
    let normals = attributes.borrow().normals.clone();

    assert_eq!
    (
      normals.len(), positions_len,
      "normals must be parallel to positions -- one normal per vertex"
    );
    assert!( !normals.is_empty(), "expected at least one stroked segment" );
    for [ x, y, z ] in normals
    {
      assert!
      (
        ( x - 0.0 ).abs() < 1e-5 && ( y - 0.0 ).abs() < 1e-5 && ( z - ( -1.0 ) ).abs() < 1e-5,
        "curve_to_geometry's winding is direction-independent and always \
        faces -Z; got [{x}, {y}, {z}]"
      );
    }
  }

  /// Regression guard for `contours_to_fill_geometry`'s side of the
  /// BUG-217 fix -- see `plane_to_geometry_normals_face_positive_z`'s doc
  /// comment for the full writeup shared by all three generators. Unlike
  /// the other two generators, this function's winding depends on
  /// caller-supplied contour data, so the normal is computed at runtime
  /// rather than a fixed expectation -- this test instead asserts the
  /// invariants the runtime computation must uphold regardless of which
  /// way the input winds: exactly one normal per vertex, every normal
  /// finite and unit-length (proving `normalize` never saw a zero vector,
  /// i.e. the NaN defect stays fixed), every normal purely axis-aligned to
  /// Z (since the input contour is planar in XY), and every vertex in the
  /// body sharing the identical broadcast normal.
  #[ test ]
  #[ cfg( feature = "font-processing" ) ]
  fn contours_to_fill_geometry_normals_are_unit_length_and_z_aligned()
  {
    let contour = vec!
    [
      [ 0.0_f32, 0.0_f32 ],
      [ 4.0_f32, 0.0_f32 ],
      [ 4.0_f32, 4.0_f32 ],
      [ 0.0_f32, 4.0_f32 ],
    ];
    let primitive = contours_to_fill_geometry( &[ contour ] )
    .expect( "a valid square contour must produce geometry" );
    let attributes = primitive.attributes.expect( "geometry must have attributes" );
    let positions_len = attributes.borrow().positions.len();
    let normals = attributes.borrow().normals.clone();

    assert_eq!
    (
      normals.len(), positions_len,
      "normals must be parallel to positions -- one normal per vertex"
    );
    assert!( !normals.is_empty(), "expected at least one triangulated vertex" );

    let first = normals[ 0 ];
    for [ x, y, z ] in normals
    {
      let magnitude = ( x * x + y * y + z * z ).sqrt();
      assert!
      (
        ( magnitude - 1.0 ).abs() < 1e-5,
        "every normal must be unit-length ( proving normalize() never saw a \
        zero vector, i.e. the NaN defect stays fixed ); got magnitude \
        {magnitude} from [{x}, {y}, {z}]"
      );
      assert!
      (
        x.abs() < 1e-5 && y.abs() < 1e-5 && ( z.abs() - 1.0 ).abs() < 1e-5,
        "a planar XY contour must produce a Z-aligned normal; got [{x}, {y}, {z}]"
      );
      // the source broadcasts one `body_normal` via `std::iter::repeat(..)`
      // (`src/primitive.rs`), so every vertex's normal is a bit-exact copy
      // of the same value -- exact equality is the actual invariant being
      // tested here, not an approximate numerical comparison.
      #[ expect( clippy::float_cmp, reason = "asserting a verbatim broadcast copy, not an independently-computed approximation" ) ]
      let identical_to_first = [ x, y, z ] == first;
      assert!
      (
        identical_to_first,
        "every vertex in a single coplanar body must share the identical \
        broadcast normal; got [{x}, {y}, {z}] vs first {first:?}"
      );
    }
  }
}
