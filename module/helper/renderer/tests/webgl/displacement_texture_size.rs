//! Regression coverage for BUG-252: `Skeleton`'s morph-target displacement-texture sizing
//! formula ( `renderer::webgl::displacement_texture_size_compute` ) could compute a texture
//! width of `0` for a primitive with few vertices relative to its attribute/target count,
//! which then forced the height to `+inf` ( saturating to `u32::MAX` ) -- silently and
//! permanently failing the displacement-texture update every frame instead of ever writing
//! real morph-target data.
//
// test_kind: bug_reproducer(BUG-252)
//
// ## Root Cause
// `a`, the texture row width, was computed as the largest multiple of `vertex_displacement_len`
// ( attributes_count * targets_count, i.e. texels-per-vertex ) that is `<= sqrt(data_len)`, via
// plain `floor()`. Whenever `sqrt(data_len) < vertex_displacement_len` -- a small vertex count
// relative to the attribute/target count -- the only such multiple is the zeroth one, so `a`
// collapsed to `0`. `b` was then computed as `ceil(data_len / a)`, i.e. a division by zero,
// which for a positive `data_len` produces `+inf` and saturates to `u32::MAX` on the cast to
// `u32`.
//
// ## Why Not Caught
// No test exercised the size formula with a small vertex count relative to the attribute/target
// count. The failure mode has no crash and no obviously-wrong render: `a.max(b) > max_size`
// (the very next check) always caught the resulting `u32::MAX`, so the update was cleanly
// abandoned every frame with a console error reading like a legitimate "texture too large"
// condition -- never a hint that `a` itself had collapsed to `0` via unrelated division-by-zero
// arithmetic. `need_update_displacement` stays `true` forever in this state, so the primitive
// silently renders in its base pose ( no morph-target displacement ever applied ) with no other
// visible symptom.
//
// ## Fix Applied
// Extracted the size computation into its own pure function and changed `floor()` to
// `floor().max( 1.0 )`, guaranteeing at least one multiple of `vertex_displacement_len` is
// always chosen when there is data to store -- `a` can now only be `0` when
// `vertex_displacement_len` itself is `0`, a case already excluded by the caller's own
// `vertex_displacement_len != 0` guard.
//
// ## Prevention
// Any "round down to the nearest multiple of N" sizing formula must be checked at its own
// smallest legitimate input -- flooring a ratio to `0` is always representable and always
// compiles, so nothing signals the degenerate case except reasoning through the boundary by
// hand or a test that specifically targets it.
//
// ## Pitfall
// A downstream bounds/limit check ( here, `a.max(b) > max_size` ) can incidentally catch an
// unrelated arithmetic bug's symptom ( `u32::MAX` from a saturated `+inf` ) and make it look
// like the limit check is doing its job correctly, masking that the value it's checking was
// already nonsensical before the check ever ran.

use renderer::webgl::displacement_texture_size_compute;

#[ test ]
fn few_vertices_with_many_attributes_and_targets_does_not_collapse_width_to_zero()
{
  // 1 vertex, 1 attribute, 10 targets -> vertex_displacement_len = 10, data_len = 1*1*10*4 = 40.
  // sqrt(40) ~= 6.32 < 10, the exact condition that pre-fix collapsed `a` to 0.
  let ( a, b ) = displacement_texture_size_compute( 40, 10 );

  assert_ne!( a, 0, "row width must never be 0 when there is data to store" );
  assert_eq!( a, 10, "row width must be at least one full vertex-block wide" );
  assert_eq!( b, 4, "height must be a finite, tightly-fit value, not a saturated u32::MAX" );
}

#[ test ]
fn computed_capacity_always_covers_the_requested_data_length()
{
  // The property that actually matters downstream: `a * b * 4` ( float capacity ) must never
  // be less than `data_len`, or the caller's `data.extend(vec![0.0; a*b*4 - data_len])` would
  // underflow. Sweep several small vertex/attribute/target combinations, including the
  // degenerate few-vertices-many-targets shape.
  for ( vertices, attrs, targets ) in
  [
    ( 1_usize, 1_usize, 10_usize ),
    ( 1, 3, 10 ),
    ( 2, 3, 10 ),
    ( 7, 3, 10 ),
    ( 100, 1, 4 ),
    ( 1, 1, 1 ),
  ]
  {
    let vertex_displacement_len = attrs * targets;
    let data_len = vertices * vertex_displacement_len * 4;

    let ( a, b ) = displacement_texture_size_compute( data_len, vertex_displacement_len );
    let capacity = ( a as usize ) * ( b as usize ) * 4;

    assert!
    (
      capacity >= data_len,
      "capacity {capacity} ( a={a}, b={b} ) must cover data_len {data_len} \
      ( vertices={vertices}, attrs={attrs}, targets={targets} )"
    );
  }
}

#[ test ]
fn ordinary_vertex_count_is_unaffected_by_the_fix()
{
  // 100 vertices, 1 attribute, 4 targets -- `i` already floors to >= 1 here, so `.max(1.0)`
  // must be a no-op and the result must match the pre-fix formula's own intended output.
  let ( a, b ) = displacement_texture_size_compute( 1600, 4 );

  assert_eq!( ( a, b ), ( 40, 40 ), "ordinary case must be unchanged by the zero-width fix" );
}
