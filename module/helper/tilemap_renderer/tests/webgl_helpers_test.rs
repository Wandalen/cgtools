//! The GL-free helpers of the WebGL adapter: `StagedVec`, the CPU staging that
//! decides which part of an instance buffer each frame re-uploads, and
//! `mat3_mul`, which folds the `WebGlBackend::set_camera` view matrix into each
//! draw's transform.
//!
//! These run on the host: nothing here touches a WebGL2 context.

#![ cfg( all( feature = "adapter-webgl", feature = "test_internals" ) ) ]

use tilemap_renderer::adapters::webgl::webgl_helpers::internal::{ mat3_mul, StagedUpload, StagedVec };
use tilemap_renderer::types::Transform;

/// Pushes that stay within the GPU capacity upload as one range, and once
/// taken the upload is not repeated. A repeat would re-send the buffer every
/// frame although nothing changed.
#[ test ]
fn staged_vec_take_upload_pushes_within_capacity_as_one_range()
{
  let mut v = StagedVec::new( 16 );
  for i in 0..10_u32 { v.push( i ); }
  assert_eq!( v.take_upload(), StagedUpload::Range { start : 0, end : 10 } );
  assert_eq!( v.take_upload(), StagedUpload::None, "an upload is recorded as done" );
}

/// Growing past the GPU capacity asks for one reallocation with headroom, and
/// a push that fits after it uploads only itself. Without the headroom every
/// push past capacity would reallocate the buffer again.
#[ test ]
fn staged_vec_take_upload_reallocates_once_past_capacity()
{
  let mut v = StagedVec::new( 16 );
  for i in 0..40_u32 { v.push( i ); }
  assert_eq!( v.take_upload(), StagedUpload::Realloc { capacity : 60 } );
  v.push( 40 );
  assert_eq!( v.take_upload(), StagedUpload::Range { start : 40, end : 41 }, "fits after the realloc" );
  for i in 0..40_u32 { v.push( i ); }
  assert_eq!( v.take_upload(), StagedUpload::Realloc { capacity : 121 } );
}

/// Scattered sets and a swap-remove merge into the one range that covers all
/// of them, and the remove moves the last item into the hole. A frame with a
/// few edits then costs one `bufferSubData`, not one per edit.
#[ test ]
fn staged_vec_take_upload_merges_sets_and_removes_into_one_range()
{
  let mut v = StagedVec::new( 8 );
  for i in 0..8_u32 { v.push( i ); }
  let _ = v.take_upload();
  v.set( 5, 50 );
  v.set( 2, 20 );
  assert_eq!( v.swap_remove( 3 ), 7 );
  assert_eq!( v.items(), &[ 0, 1, 20, 7, 4, 50, 6 ] );
  assert_eq!( v.take_upload(), StagedUpload::Range { start : 2, end : 6 } );
}

/// Removing from the tail only shortens the drawn count, so nothing is
/// uploaded. The GPU copy below the new length is already correct.
#[ test ]
fn staged_vec_take_upload_skips_tail_removes()
{
  let mut v = StagedVec::new( 8 );
  for i in 0..8_u32 { v.push( i ); }
  let _ = v.take_upload();
  v.swap_remove( 7 );
  v.swap_remove( 6 );
  assert_eq!( v.take_upload(), StagedUpload::None );
  assert_eq!( v.len(), 6 );
}

/// A set on an item that is then removed from the tail uploads nothing: the
/// edited item is no longer drawn. Uploading it would write past the length.
#[ test ]
fn staged_vec_take_upload_drops_a_set_truncated_away()
{
  let mut v = StagedVec::new( 8 );
  for i in 0..8_u32 { v.push( i ); }
  let _ = v.take_upload();
  v.set( 7, 70 );
  v.swap_remove( 7 );
  assert_eq!( v.take_upload(), StagedUpload::None );
}

// --- mat3_mul ---

/// Column-major identity.
const IDENTITY : [ f32; 9 ] = [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ];

/// A view matrix that zooms by 2, then pans by `( 200, 120 )` pixels.
const CAMERA : [ f32; 9 ] = [ 2.0, 0.0, 0.0, 0.0, 2.0, 0.0, 200.0, 120.0, 1.0 ];

/// A quad transform: 40×30 at `( 100, 50 )`.
fn quad() -> Transform
{
  Transform { position : [ 100.0, 50.0 ], scale : [ 40.0, 30.0 ], ..Transform::default() }
}

/// The identity is neutral on either side, so an identity camera (the
/// backend's default) leaves every draw where its transform puts it.
#[ allow( clippy::float_cmp, reason = "every entry is a small integer or a product of small integers, exact in f32, so exact comparison is the test intent" ) ]
#[ test ]
fn mat3_mul_identity_is_neutral()
{
  let m = quad().to_mat3();
  assert_eq!( mat3_mul( &IDENTITY, &m ), m );
  assert_eq!( mat3_mul( &m, &IDENTITY ), m );
}

/// `mat3_mul( camera, transform )` applies the transform first, then the
/// camera: the quad lands at `2 · ( 100, 50 ) + ( 200, 120 )` at twice its
/// size. This is the order every world-space draw uses.
#[ allow( clippy::float_cmp, reason = "every entry is a small integer or a product of small integers, exact in f32, so exact comparison is the test intent" ) ]
#[ test ]
fn mat3_mul_applies_the_right_operand_first()
{
  let expected = Transform { position : [ 400.0, 220.0 ], scale : [ 80.0, 60.0 ], ..Transform::default() }.to_mat3();
  assert_eq!( mat3_mul( &CAMERA, &quad().to_mat3() ), expected );
}

/// Swapping the operands gives a different matrix: the pan would be scaled by
/// the quad's size. So a call site that passed the camera second would fail
/// [`mat3_mul_applies_the_right_operand_first`]'s expectation.
#[ allow( clippy::float_cmp, reason = "every entry is a small integer or a product of small integers, exact in f32, so exact comparison is the test intent" ) ]
#[ test ]
fn mat3_mul_operand_order_matters()
{
  let m = quad().to_mat3();
  assert_ne!( mat3_mul( &m, &CAMERA ), mat3_mul( &CAMERA, &m ) );
}

/// A full product with every entry in play, checked against the product
/// written out by hand. Neither operand is an affine transform or symmetric,
/// so a swapped row and column index in the multiply shows.
#[ allow( clippy::float_cmp, reason = "every entry is a small integer or a product of small integers, exact in f32, so exact comparison is the test intent" ) ]
#[ test ]
fn mat3_mul_matches_the_written_out_product()
{
  // Column-major: a = [ [ 1, 2, 3 ], [ 4, 5, 6 ], [ 7, 8, 9 ] ] by rows.
  let a = [ 1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0 ];
  // b = [ [ 9, 8, 7 ], [ 6, 5, 4 ], [ 3, 2, 1 ] ] by rows.
  let b = [ 9.0, 6.0, 3.0, 8.0, 5.0, 2.0, 7.0, 4.0, 1.0 ];
  // a · b = [ [ 30, 24, 18 ], [ 84, 69, 54 ], [ 138, 114, 90 ] ] by rows.
  let expected = [ 30.0, 84.0, 138.0, 24.0, 69.0, 114.0, 18.0, 54.0, 90.0 ];
  assert_eq!( mat3_mul( &a, &b ), expected );
}
