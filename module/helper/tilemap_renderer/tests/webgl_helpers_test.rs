//! The GL-free helpers of the WebGL adapter: `StagedVec`, the CPU staging that
//! decides which part of an instance buffer each frame re-uploads.
//!
//! These run on the host: nothing here touches a WebGL2 context.

#![ cfg( feature = "adapter-webgl" ) ]

use tilemap_renderer::adapters::webgl::webgl_helpers::{ StagedUpload, StagedVec };

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
