//! Tests for animation modifier Scaler

use renderer::webgl::animation::Scaler;
use animation::{ Sequence, Sequencer, Tween, easing::{ EasingBuilder, Linear } };
use mingl::{ F64x3, QuatF64 };
use std::f64::consts::PI;

const TRANSLATION_PREFIX: &str = "_translation";
const ROTATION_PREFIX: &str = "_rotation";

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

#[ test ]
fn test_scaler_new()
{
  let sequencer = Sequencer::new();
  let scaler = Scaler::new( sequencer );

  // Verify internal state through public API
  assert!( scaler.group_get( "nonexistent" ).is_none(), "New scaler should have no scaled nodes" );
}

#[ test ]
fn test_scaler_add_group()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  let nodes = vec![ "node1".into(), "node2".into() ];
  let scale = F64x3::new( 0.5, 0.5, 1.0 );

  scaler.add( "group1", nodes.clone(), scale );

  let group = scaler.group_get( "group1" ).unwrap();
  assert_eq!( group.len(), 2, "Group should have 2 nodes" );
}

#[ test ]
fn test_scaler_remove_group()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  scaler.add( "group1", vec![ "node1".into() ], F64x3::splat( 0.5 ) );
  assert!( scaler.group_get( "group1" ).is_some(), "Group should exist" );

  scaler.remove( "group1".into() );
  assert!( scaler.group_get( "group1" ).is_none(), "Group should be removed" );
}

#[ test ]
fn test_scaler_scale_get_mut()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  scaler.add( "group1", vec![ "node1".into() ], F64x3::new( 0.5, 0.5, 1.0 ) );

  let scale = scaler.scale_get_mut( "group1" ).unwrap();
  *scale = F64x3::new( 1.0, 1.0, 1.0 );

  let new_scale = scaler.scale_get( "group1" ).unwrap();
  assert_eq!( new_scale.x(), 1.0 );
  assert_eq!( new_scale.y(), 1.0 );
  assert_eq!( new_scale.z(), 1.0 );
}

#[ test ]
fn test_scaler_clear()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  scaler.add( "group1", vec![ "node1".into() ], F64x3::splat( 0.5 ) );
  scaler.add( "group2", vec![ "node2".into() ], F64x3::splat( 0.8 ) );
  assert!( scaler.group_get( "group1" ).is_some() );
  assert!( scaler.group_get( "group2" ).is_some() );

  scaler.clear();
  assert!( scaler.group_get( "group1" ).is_none(), "All groups should be cleared" );
  assert!( scaler.group_get( "group2" ).is_none(), "All groups should be cleared" );
}

#[ test ]
fn test_grouped_nodes_independence()
{
  let mut sequencer = Sequencer::new();

  // Add two rotation animations with different angles
  let rot1_start = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let rot1_end = QuatF64::from_axis_angle( F64x3::new( 0.0, 0.0, 1.0 ), PI / 2.0 );
  let seq1 = create_rotation_sequence( rot1_start, rot1_end, 1.0 );
  sequencer.add( format!( "node1{}", ROTATION_PREFIX ).as_str(), seq1 );

  let rot2_start = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let rot2_end = QuatF64::from_axis_angle( F64x3::new( 1.0, 0.0, 0.0 ), PI );
  let seq2 = create_rotation_sequence( rot2_start, rot2_end, 1.0 );
  sequencer.add( format!( "node2{}", ROTATION_PREFIX ).as_str(), seq2 );

  let mut scaler = Scaler::new( sequencer );

  // Add two groups with different scaling factors
  scaler.add( "group1", vec![ "node1".into() ], F64x3::new( 1.0, 0.5, 1.0 ) ); // 50% rotation scaling
  scaler.add( "group2", vec![ "node2".into() ], F64x3::new( 1.0, 0.25, 1.0 ) ); // 25% rotation scaling

  // Verify groups are independent
  let group1_scale = scaler.scale_get( "group1" ).unwrap();
  let group2_scale = scaler.scale_get( "group2" ).unwrap();

  assert_eq!( group1_scale.y(), 0.5, "Group1 should have 0.5 rotation scale" );
  assert_eq!( group2_scale.y(), 0.25, "Group2 should have 0.25 rotation scale" );
}

#[ test ]
fn test_animatable_composition_update()
{
  let mut sequencer = Sequencer::new();

  // Add a simple translation animation
  let start = F64x3::new( 0.0, 0.0, 0.0 );
  let end = F64x3::new( 1.0, 1.0, 1.0 );
  let seq = create_translation_sequence( start, end, 1.0 );
  sequencer.add( format!( "node1{}", TRANSLATION_PREFIX ).as_str(), seq );

  let mut scaler = Scaler::new( sequencer );

  // Update should not panic
  use renderer::webgl::animation::AnimatableComposition;
  scaler.update( 0.5 );
}

#[ test ]
fn test_scaler_weights_structure()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  // Test that weights have three components: translation (x), rotation (y), scale (z)
  scaler.add( "group1", vec![ "node1".into() ], F64x3::new( 0.5, 0.7, 0.3 ) );

  let weights = scaler.scale_get( "group1" ).unwrap();
  assert_eq!( weights.x(), 0.5, "X component should be translation weight" );
  assert_eq!( weights.y(), 0.7, "Y component should be rotation weight" );
  assert_eq!( weights.z(), 0.3, "Z component should be scale weight" );
}
