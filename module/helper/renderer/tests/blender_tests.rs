//! Tests for animation modifier Blender
#![ cfg( feature = "animation" ) ]

use std::{ rc::Rc, cell::RefCell };
use renderer::webgl::
{
  Node,
  animation::
  {
    AnimatableComposition, Blender, weights_normalize,
    base::{ TRANSLATION_PREFIX, ROTATION_PREFIX, SCALE_PREFIX }
  }
};
use animation::{ Tween, Sequence, Sequencer, easing::{ EasingBuilder, Linear } };
use mingl::{ F64x3, QuatF32, QuatF64 };
use core::f64;
use std::f64::consts::PI;
use rustc_hash::FxHashMap;

/// Helper to create a simple translation tween sequence
fn translation_sequence_create( start : F64x3, end : F64x3, duration : f64 ) -> Sequence< Tween< F64x3 > >
{
  let tweens =
  vec![
    Tween::new( start, end, duration / 2.0, Linear::build() ),
    Tween::new( end, start, duration / 2.0, Linear::build() )
  ];
  Sequence::new( tweens ).unwrap()
}

/// Helper to create a simple rotation tween sequence
fn rotation_sequence_create( start : QuatF64, end : QuatF64, duration : f64 ) -> Sequence< Tween< QuatF64 > >
{
  let tweens =
  vec![
    Tween::new( start, end, duration / 2.0, Linear::build() ),
    Tween::new( end, start, duration / 2.0, Linear::build() )
  ];
  Sequence::new( tweens ).unwrap()
}

/// Helper to create a simple scale tween sequence
fn scale_sequence_create( start : F64x3, end : F64x3, duration : f64 ) -> Sequence< Tween< F64x3 > >
{
  let tweens =
  vec![
    Tween::new( start, end, duration / 2.0, Linear::build() ),
    Tween::new( end, start, duration / 2.0, Linear::build() )
  ];
  Sequence::new( tweens ).unwrap()
}

#[ test ]
fn test_weights_normalize_basic()
{
  let mut values = vec!
  [
    ( 1.0, 0.5_f32 ),
    ( 2.0, 0.5_f32 ),
  ];

  weights_normalize( &mut values );

  // Sum should be 1.0
  let sum : f32 = values.iter().map( | ( _, w ) | w ).sum();
  assert!( ( sum - 1.0 ).abs() < 1e-6, "Weights should sum to 1.0 after normalization" );
}

#[ test ]
fn test_weights_normalize_unequal()
{
  let mut values = vec!
  [
    ( 1.0, 0.3_f32 ),
    ( 2.0, 0.7_f32 ),
  ];

  weights_normalize( &mut values );

  // Sum should be 1.0
  let sum : f32 = values.iter().map( | ( _, w ) | w ).sum();
  assert!( ( sum - 1.0 ).abs() < 1e-6, "Weights should sum to 1.0 after normalization" );

  // Ratio should be preserved (approximately)
  let ratio = values[ 0 ].1 / values[ 1 ].1;
  assert!( ( ratio - 0.3 / 0.7 ).abs() < 1e-5, "Weight ratio should be preserved" );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "weights with zero sum are untouched by `weights_normalize`; the retrieved value is the exact input literal" ) ]
fn test_weights_normalize_zero_sum()
{
  let mut values = vec!
  [
    ( 1.0, 0.0_f32 ),
    ( 2.0, 0.0_f32 ),
  ];

  // Should not panic with zero sum
  weights_normalize( &mut values );

  // Weights should remain zero
  assert_eq!( values[ 0 ].1, 0.0 );
  assert_eq!( values[ 1 ].1, 0.0 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "weight literals round-trip unchanged through `weights_get`; no arithmetic in between" ) ]
fn test_blender_weights_get_mut()
{
  let mut blender = Blender::new();
  let mut sequencer = Sequencer::new();
  sequencer.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), sequencer, F64x3::new( 0.5, 0.5, 0.5 ) );

  let weights = blender.weights_get_mut( "anim1" ).unwrap();
  *weights = F64x3::new( 1.0, 1.0, 1.0 );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().name_set( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  let new_weights = blender.weights_get( "anim1" ).unwrap();
  assert_eq!( new_weights.x(), 1.0 );
  assert_eq!( new_weights.y(), 1.0 );
  assert_eq!( new_weights.z(), 1.0 );
}

#[ test ]
fn test_blender_animation_get()
{
  let mut blender = Blender::new();
  let sequencer = Sequencer::new();

  blender.add( "anim1".into(), sequencer, F64x3::splat( 0.5 ) );

  let anim = blender.animation_get( "anim1" );
  assert!( anim.is_some(), "Should be able to retrieve animation" );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "weight literals round-trip unchanged through `weights_get`; no arithmetic in between" ) ]
fn test_blender_multiple_animations_with_different_weights()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), seq1, F64x3::new( 0.7, 0.0, 0.0 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.3, 0.0, 0.0 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().name_set( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  let weights1 = blender.weights_get( "anim1" ).unwrap();
  let weights2 = blender.weights_get( "anim2" ).unwrap();

  assert_eq!( weights1.x(), 0.7, "First animation should have weight 0.7" );
  assert_eq!( weights2.x(), 0.3, "Second animation should have weight 0.3" );
}

#[ test ]
fn test_blender_normalization_enabled()
{
  let mut blender = Blender::new();
  blender.normalize = true;

  let mut seq1 = Sequencer::new();
  seq1.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 2.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 2.0, 0.0 ), 1.0 )
  );

  // Weights don't sum to 1.0
  blender.add( "anim1".into(), seq1, F64x3::new( 0.6, 0.0, 0.0 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.6, 0.0, 0.0 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().name_set( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  assert!( blender.normalize, "Normalization should be enabled" );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "weight literals round-trip unchanged through `weights_get`; no arithmetic in between" ) ]
fn test_blender_independent_transform_blend()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );
  seq1.insert
  (
    format!( "node1{ROTATION_PREFIX}" ).as_str(),
    rotation_sequence_create
    (
      QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ),
      QuatF64::from_axis_angle( F64x3::new( 0.0, 0.0, 1.0 ), PI / 2.0 ),
      1.0
    )
  );
  seq1.insert
  (
    format!( "node1{SCALE_PREFIX}" ).as_str(),
    scale_sequence_create( F64x3::new( 1.0, 1.0, 1.0 ), F64x3::new( 2.0, 2.0, 2.0 ), 1.0 )
  );

  // Different weights for translation, rotation, and scale
  let weights = F64x3::new( 0.5, 0.7, 0.3 );
  blender.add( "anim1".into(), seq1, weights );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().name_set( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  let retrieved_weights = blender.weights_get( "anim1" ).unwrap();
  assert_eq!( retrieved_weights.x(), 0.5, "Translation weight should be 0.5" );
  assert_eq!( retrieved_weights.y(), 0.7, "Rotation weight should be 0.7" );
  assert_eq!( retrieved_weights.z(), 0.3, "Scale weight should be 0.3" );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "weight literals round-trip unchanged through `weights_get`; no arithmetic in between" ) ]
fn test_blender_scale_blend_independence()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.insert
  (
    format!( "node1{SCALE_PREFIX}" ).as_str(),
    scale_sequence_create( F64x3::new( 1.0, 1.0, 1.0 ), F64x3::new( 2.0, 2.0, 2.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.insert
  (
    format!( "node1{SCALE_PREFIX}" ).as_str(),
    scale_sequence_create( F64x3::new( 1.0, 1.0, 1.0 ), F64x3::new( 0.5, 0.5, 0.5 ), 1.0 )
  );

  blender.add( "anim1".into(), seq1, F64x3::new( 0.0, 0.0, 0.6 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.0, 0.0, 0.4 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().name_set( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  let weights1 = blender.weights_get( "anim1" ).unwrap();
  let weights2 = blender.weights_get( "anim2" ).unwrap();

  assert_eq!( weights1.z(), 0.6, "First animation scale weight should be 0.6" );
  assert_eq!( weights2.z(), 0.4, "Second animation scale weight should be 0.4" );
}

#[ test ]
fn test_blender_reset()
{
  let mut blender = Blender::new();
  let mut sequencer = Sequencer::new();

  sequencer.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), sequencer, F64x3::splat( 1.0 ) );

  // Update to advance time
  blender.update( 0.5 );

  // Reset should work without panic
  blender.reset();
}

#[ test ]
fn test_blender_update()
{
  let mut blender = Blender::new();
  let mut sequencer = Sequencer::new();

  sequencer.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), sequencer, F64x3::splat( 1.0 ) );

  // Update should not panic
  blender.update( 0.5 );
}

#[ test ]
fn test_blender_as_any()
{
  let blender = Blender::new();

  let any_ref = blender.as_any();
  assert!( any_ref.is::< Blender >(), "Should be able to downcast to Blender" );
}

#[ test ]
fn test_is_completed_single_animation_not_completed()
{
  let mut blender = Blender::new();
  let mut sequencer = Sequencer::new();

  sequencer.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), sequencer, F64x3::splat( 1.0 ) );

  // Update halfway through
  blender.update( 0.5 );

  assert!( !blender.is_completed(), "Animation should not be completed at 0.5s of 1.0s duration" );
}

#[ test ]
fn test_is_completed_single_animation_completed()
{
  let mut blender = Blender::new();
  let mut sequencer = Sequencer::new();

  sequencer.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), sequencer, F64x3::splat( 1.0 ) );

  // Update past the duration
  blender.update( 1.5 );

  assert!( !blender.is_completed(), "Animation should reset after completed 1.5s from animation start" );
}

#[ test ]
fn test_is_completed_multiple_animations_same_time_not_completed()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.insert
  (
    format!( "node2{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), seq1, F64x3::new( 0.5, 0.0, 0.0 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.5, 0.0, 0.0 ) );

  // Update both animations halfway through
  blender.update( 0.5 );

  assert!( !blender.is_completed(), "Multiple animations at same time not yet completed but marked as completed" );
}

#[ test ]
fn test_is_completed_multiple_animations_same_time_completed()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.insert
  (
    format!( "node2{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), seq1, F64x3::new( 0.5, 0.0, 0.0 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.5, 0.0, 0.0 ) );

  // Update both animations past their duration
  blender.update( 1.5 );

  assert!( !blender.is_completed(), "Multiple animations with same delay and duration are completed but should be applyied reset for both" );
}

#[ test ]
fn test_is_completed_multiple_animations_different_times()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.insert
  (
    format!( "node2{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 2.0 )
  );

  blender.add( "anim1".into(), seq1, F64x3::new( 0.5, 0.0, 0.0 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.5, 0.0, 0.0 ) );

  // Update: first animation completes but second doesn't
  blender.update( 1.5 );

  assert!( !blender.is_completed(), "Animations at different times should not be considered completed" );
}

#[ test ]
fn test_is_completed_multiple_animations_different_durations()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.insert
  (
    format!( "node2{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 2.0 )
  );

  blender.add( "anim1".into(), seq1, F64x3::new( 0.5, 0.0, 0.0 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.5, 0.0, 0.0 ) );

  // Update past both durations
  blender.update( 2.5 );

  // Even though both are completed, they're at different times, so should return false
  assert!( !blender.is_completed(), "Different duration animations at different times should not be considered completed" );
}

#[ test ]
fn test_is_completed_after_reset()
{
  let mut blender = Blender::new();
  let mut sequencer = Sequencer::new();

  sequencer.insert
  (
    format!( "node1{TRANSLATION_PREFIX}" ).as_str(),
    translation_sequence_create( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), sequencer, F64x3::splat( 1.0 ) );

  // Complete the animation
  blender.update( 1.5 );
  assert!( !blender.is_completed(), "Should automatically apply reset when any animation is completed" );

  // Reset
  blender.reset();

  // After reset, should not be completed
  assert!( !blender.is_completed(), "Should not be completed after reset" );
}

/// ## Root Cause
/// `Blender::rotation_blend` summed each blended clip's current rotation quaternion directly
/// ( `rotation += r * w` ) with no hemisphere check. A quaternion `q` and its negation `-q`
/// represent the identical rotation, but naive addition does not respect that equivalence --
/// blending two clips whose current rotations land in opposite hemispheres ( negative dot
/// product ) walks the long way around between them instead of the short way, producing a
/// result up to 180 degrees away from the intended blend.
///
/// ## Why Not Caught
/// Every pre-existing `Blender` test used a single weighted animation, or multiple animations
/// blending independent transform channels ( translation/scale ) rather than two rotation
/// clips whose sampled quaternions actually land in opposite hemispheres -- so the
/// dot-product-negative branch was never exercised.
///
/// ## Fix Applied
/// `rotation_blend`'s accumulation loop now checks `rotation.dot( &r ) < 0.0` before summing
/// each entry, negating `r` first when the check fires ( BUG-183, `blending.rs` ).
///
/// ## Prevention
/// This test blends two rotation clips whose ( unadvanced, `Pending`-state ) current values
/// are 0 degrees and 270 degrees about the same axis -- a negative-dot-product pair -- and
/// asserts the blended result matches the short-path ( -45 degree ) blend rather than the
/// long-path ( 135 degree ) blend a naive sum would produce.
///
/// ## Pitfall
/// `Blender` stores its clips in an `FxHashMap`, so the order `rotation_blend` visits them is
/// not guaranteed -- flipping which clip is "first" flips the overall sign of the accumulated
/// result ( `q` and `-q` are the same rotation, but not the same quaternion components ).
/// Assertions here compare via `|dot( got, expected )|` rather than direct component equality
/// so the test passes regardless of iteration order.
// test_kind: bug_reproducer(BUG-183)
#[ test ]
fn test_blender_rotation_blend_aligns_hemisphere_across_clips()
{
  let mut blender = Blender::new();

  let q_a = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let half_270 = 270.0_f64.to_radians() / 2.0;
  let q_b = QuatF64::from( [ 0.0, 0.0, half_270.sin(), half_270.cos() ] );
  assert!( q_a.dot( &q_b ) < 0.0, "fixture must exercise the negative-dot-product branch" );

  let mut seq_a = Sequencer::new();
  seq_a.insert
  (
    format!( "node1{ROTATION_PREFIX}" ).as_str(),
    rotation_sequence_create( q_a, q_a, 1.0 )
  );
  blender.add( "anim_a".into(), seq_a, F64x3::new( 0.0, 0.5, 0.0 ) );

  let mut seq_b = Sequencer::new();
  seq_b.insert
  (
    format!( "node1{ROTATION_PREFIX}" ).as_str(),
    rotation_sequence_create( q_b, q_b, 1.0 )
  );
  blender.add( "anim_b".into(), seq_b, F64x3::new( 0.0, 0.5, 0.0 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().name_set( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node.clone() );

  blender.set( &nodes );

  let got = node.borrow().rotation_get();

  let len_sq = got.dot( &got );
  assert!( ( len_sq - 1.0 ).abs() < 1e-5, "blended rotation must be unit-length, got squared length {len_sq}" );

  let neg45_half = ( -45.0_f64.to_radians() / 2.0 ) as f32;
  let expected = QuatF32::from( [ 0.0, 0.0, neg45_half.sin(), neg45_half.cos() ] );
  let dot_expected = ( got.x() * expected.x() + got.y() * expected.y() + got.z() * expected.z() + got.w() * expected.w() ).abs();
  assert!
  (
    dot_expected > 0.999,
    "blended rotation should match the short-path -45 degree blend up to sign, got {got:?} ( |dot| with expected = {dot_expected} )"
  );

  let half_135 = ( 135.0_f64.to_radians() / 2.0 ) as f32;
  let buggy = QuatF32::from( [ 0.0, 0.0, half_135.sin(), half_135.cos() ] );
  let dot_buggy = ( got.x() * buggy.x() + got.y() * buggy.y() + got.z() * buggy.z() + got.w() * buggy.w() ).abs();
  assert!
  (
    dot_buggy < 0.5,
    "blended rotation must not match the pre-fix long-path 135 degree blend, got {got:?} ( |dot| with buggy = {dot_buggy} )"
  );
}
