//! Tests for animation modifier Blender

use std::{ rc::Rc, cell::RefCell };
use renderer::webgl::
{
  Node,
  animation::{ AnimatableComposition, Blender, normalize_weights }
};
use animation::{ Tween, Sequence, Sequencer, easing::{ EasingBuilder, Linear } };
use mingl::{ F64x3, QuatF64 };
use core::f64;
use std::f64::consts::PI;
use rustc_hash::FxHashMap;

const TRANSLATION_PREFIX: &str = "_translation";
const ROTATION_PREFIX: &str = "_rotation";
const SCALE_PREFIX: &str = "_scale";

/// Helper to create a simple translation tween sequence
fn create_translation_sequence( start : F64x3, end : F64x3, duration : f64 ) -> Sequence< Tween< F64x3 > >
{
  let tweens =
  vec![
    Tween::new( start, end, duration / 2.0, Linear::new() ),
    Tween::new( end, start, duration / 2.0, Linear::new() )
  ];
  Sequence::new( tweens ).unwrap()
}

/// Helper to create a simple rotation tween sequence
fn create_rotation_sequence( start : QuatF64, end : QuatF64, duration : f64 ) -> Sequence< Tween< QuatF64 > >
{
  let tweens =
  vec![
    Tween::new( start, end, duration / 2.0, Linear::new() ),
    Tween::new( end, start, duration / 2.0, Linear::new() )
  ];
  Sequence::new( tweens ).unwrap()
}

/// Helper to create a simple scale tween sequence
fn create_scale_sequence( start : F64x3, end : F64x3, duration : f64 ) -> Sequence< Tween< F64x3 > >
{
  let tweens =
  vec![
    Tween::new( start, end, duration / 2.0, Linear::new() ),
    Tween::new( end, start, duration / 2.0, Linear::new() )
  ];
  Sequence::new( tweens ).unwrap()
}

#[ test ]
fn test_normalize_weights_basic()
{
  let mut values = vec!
  [
    ( 1.0, 0.5_f32 ),
    ( 2.0, 0.5_f32 ),
  ];

  normalize_weights( &mut values );

  // Sum should be 1.0
  let sum : f32 = values.iter().map( | ( _, w ) | w ).sum();
  assert!( ( sum - 1.0 ).abs() < 1e-6, "Weights should sum to 1.0 after normalization" );
}

#[ test ]
fn test_normalize_weights_unequal()
{
  let mut values = vec!
  [
    ( 1.0, 0.3_f32 ),
    ( 2.0, 0.7_f32 ),
  ];

  normalize_weights( &mut values );

  // Sum should be 1.0
  let sum : f32 = values.iter().map( | ( _, w ) | w ).sum();
  assert!( ( sum - 1.0 ).abs() < 1e-6, "Weights should sum to 1.0 after normalization" );

  // Ratio should be preserved (approximately)
  let ratio = values[ 0 ].1 / values[ 1 ].1;
  assert!( ( ratio - 0.3 / 0.7 ).abs() < 1e-5, "Weight ratio should be preserved" );
}

#[ test ]
fn test_normalize_weights_zero_sum()
{
  let mut values = vec!
  [
    ( 1.0, 0.0_f32 ),
    ( 2.0, 0.0_f32 ),
  ];

  // Should not panic with zero sum
  normalize_weights( &mut values );

  // Weights should remain zero
  assert_eq!( values[ 0 ].1, 0.0 );
  assert_eq!( values[ 1 ].1, 0.0 );
}

#[ test ]
fn test_blender_weights_get_mut()
{
  let mut blender = Blender::new();
  let mut sequencer = Sequencer::new();
  sequencer.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), sequencer, F64x3::new( 0.5, 0.5, 0.5 ) );

  let weights = blender.weights_get_mut( "anim1".into() ).unwrap();
  *weights = F64x3::new( 1.0, 1.0, 1.0 );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().set_name( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  let new_weights = blender.weights_get( "anim1".into() ).unwrap();
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

  let anim = blender.animation_get( "anim1".into() );
  assert!( anim.is_some(), "Should be able to retrieve animation" );
}

#[ test ]
fn test_blender_multiple_animations_with_different_weights()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 1.0 )
  );

  blender.add( "anim1".into(), seq1, F64x3::new( 0.7, 0.0, 0.0 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.3, 0.0, 0.0 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().set_name( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  let weights1 = blender.weights_get( "anim1".into() ).unwrap();
  let weights2 = blender.weights_get( "anim2".into() ).unwrap();

  assert_eq!( weights1.x(), 0.7, "First animation should have weight 0.7" );
  assert_eq!( weights2.x(), 0.3, "Second animation should have weight 0.3" );
}

#[ test ]
fn test_blender_normalization_enabled()
{
  let mut blender = Blender::new();
  blender.normalize = true;

  let mut seq1 = Sequencer::new();
  seq1.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 2.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 2.0, 0.0 ), 1.0 )
  );

  // Weights don't sum to 1.0
  blender.add( "anim1".into(), seq1, F64x3::new( 0.6, 0.0, 0.0 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.6, 0.0, 0.0 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().set_name( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  assert!( blender.normalize, "Normalization should be enabled" );
}

#[ test ]
fn test_blender_independent_transform_blend()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );
  seq1.add
  (
    format!( "node1{}", ROTATION_PREFIX ).as_str(),
    create_rotation_sequence
    (
      QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ),
      QuatF64::from_axis_angle( F64x3::new( 0.0, 0.0, 1.0 ), PI / 2.0 ),
      1.0
    )
  );
  seq1.add
  (
    format!( "node1{}", SCALE_PREFIX ).as_str(),
    create_scale_sequence( F64x3::new( 1.0, 1.0, 1.0 ), F64x3::new( 2.0, 2.0, 2.0 ), 1.0 )
  );

  // Different weights for translation, rotation, and scale
  let weights = F64x3::new( 0.5, 0.7, 0.3 );
  blender.add( "anim1".into(), seq1, weights );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().set_name( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  let retrieved_weights = blender.weights_get( "anim1".into() ).unwrap();
  assert_eq!( retrieved_weights.x(), 0.5, "Translation weight should be 0.5" );
  assert_eq!( retrieved_weights.y(), 0.7, "Rotation weight should be 0.7" );
  assert_eq!( retrieved_weights.z(), 0.3, "Scale weight should be 0.3" );
}

#[ test ]
fn test_blender_scale_blend_independence()
{
  let mut blender = Blender::new();

  let mut seq1 = Sequencer::new();
  seq1.add
  (
    format!( "node1{}", SCALE_PREFIX ).as_str(),
    create_scale_sequence( F64x3::new( 1.0, 1.0, 1.0 ), F64x3::new( 2.0, 2.0, 2.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.add
  (
    format!( "node1{}", SCALE_PREFIX ).as_str(),
    create_scale_sequence( F64x3::new( 1.0, 1.0, 1.0 ), F64x3::new( 0.5, 0.5, 0.5 ), 1.0 )
  );

  blender.add( "anim1".into(), seq1, F64x3::new( 0.0, 0.0, 0.6 ) );
  blender.add( "anim2".into(), seq2, F64x3::new( 0.0, 0.0, 0.4 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().set_name( "node1" );
  nodes.insert( "node1".to_string().into_boxed_str(), node );
  blender.set( &nodes );

  let weights1 = blender.weights_get( "anim1".into() ).unwrap();
  let weights2 = blender.weights_get( "anim2".into() ).unwrap();

  assert_eq!( weights1.z(), 0.6, "First animation scale weight should be 0.6" );
  assert_eq!( weights2.z(), 0.4, "Second animation scale weight should be 0.4" );
}

#[ test ]
fn test_blender_reset()
{
  let mut blender = Blender::new();
  let mut sequencer = Sequencer::new();

  sequencer.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
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

  sequencer.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
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

  sequencer.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
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

  sequencer.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
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
  seq1.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.add
  (
    format!( "node2{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 1.0 )
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
  seq1.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.add
  (
    format!( "node2{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 1.0 )
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
  seq1.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.add
  (
    format!( "node2{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 2.0 )
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
  seq1.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
  );

  let mut seq2 = Sequencer::new();
  seq2.add
  (
    format!( "node2{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 0.0, 1.0, 0.0 ), 2.0 )
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

  sequencer.add
  (
    format!( "node1{}", TRANSLATION_PREFIX ).as_str(),
    create_translation_sequence( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 1.0, 0.0, 0.0 ), 1.0 )
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
