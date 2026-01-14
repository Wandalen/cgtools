//! Tests for animation composer AnimationGraph
#![ cfg( feature = "animation" ) ]

use animation::{ Sequence, Sequencer, Tween, easing::{ EasingBuilder, Linear } };
use mingl::{ F64x3, QuatF64 };
use renderer::webgl::animation::
{
  AnimatableComposition, AnimationEdge, AnimationGraph, Mirror, MirrorPlane, Pose, base::
  {
    MORPH_TARGET_PREFIX,
    ROTATION_PREFIX,
    SCALE_PREFIX,
    TRANSLATION_PREFIX
  }, graph
};
use rustc_hash::FxHashMap;

fn create_animation() -> Sequencer
{
  let mut animation = Sequencer::new();

  let linear = Linear::new();
  animation.insert
  (
    TRANSLATION_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( F64x3::splat( -1.0 ), F64x3::splat( 0.0 ), 0.5, linear ),
        Tween::new( F64x3::splat( 0.0 ), F64x3::splat( 1.0 ), 0.5, linear ).with_delay( 0.5 )
      ]
    )
  );

  let linear = Linear::new();
  animation.insert
  (
    ROTATION_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( QuatF64::from( [ -1.0, -1.0, -1.0, 1.0 ] ), QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ), 0.5, linear ),
        Tween::new( QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ), QuatF64::from( [ 1.0, 1.0, 1.0, 1.0 ] ), 0.5, linear ).with_delay( 0.5 )
      ]
    )
  );

  let linear = Linear::new();
  animation.insert
  (
    SCALE_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( F32x3::splat( 1.0 ), F32x3::splat( 2.0 ), 0.5, linear ),
        Tween::new( F32x3::splat( 2.0 ), F32x3::splat( 3.0 ), 0.5, linear ).with_delay( 0.5 )
      ]
    )
  );

  let linear = Linear::new();
  animation.insert
  (
    MORPH_TARGET_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( vec![ 0.5, 0.5, 0.5 ], vec![ 0.75, 0.75, 0.75 ], 0.5, linear ),
        Tween::new( vec![ 0.75, 0.75, 0.75 ], vec![ 1.0, 1.0, 1.0 ], 0.5, linear ).with_delay( 0.5 )
      ]
    )
  );

  animation
}

fn create_graph() -> AnimationGraph
{
  let animation = create_animation();
  let animation1 = create_animation();

  let mut graph = AnimationGraph::new( &FxHashMap::default() );

  graph.node_add( "a", animation );
  graph.node_add( "b", animation1 );

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::new() );
  let true_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    true
  };
  graph.edge_add( "a", "b", "ab", instant_tween, true_condition );

  graph
}

#[ test ]
fn animation_graph_clone_test()
{
  let mut graph = create_graph();
  graph.update( 0.5 );

  let graph_clone = graph.clone();

  let animation1_clone = graph_clone.node_get( "b" );
  assert!( animation1_clone.unwrap().progress() > 0.0 );
}

#[ test ]
fn animation_graph_transition_many_choices_test()
{
  let mut graph = create_graph();
  let animation = create_animation();

  graph.node_add( "c", animation );

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::new() );
  let true_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    true
  };
  graph.edge_add( "a", "c", "ac", instant_tween, true_condition );

  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "ab".to_string().into_boxed_str() ) );
}

#[ test ]
fn animation_graph_conditions_test()
{
  let mut graph = AnimationGraph::new( &FxHashMap::default() );
  let animation = create_animation();

  graph.node_add( "a", animation.clone() );
  graph.node_add( "b", animation.clone() );
  graph.node_add( "c", animation );

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::new() );
  let false_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    false
  };

  let true_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    true
  };

  graph.edge_add( "a", "b", "ab", instant_tween.clone(), false_condition );
  graph.edge_add( "a", "c", "ac", instant_tween, true_condition );

  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "ac".to_string().into_boxed_str() ) );
}

#[ test ]
fn animation_graph_current_name_get_test()
{
  let mut graph = create_graph();
  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "b".to_string().into_boxed_str() ) );
}

#[ test ]
fn animation_graph_current_set_test()
{
  let mut graph = create_graph();
  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "b".to_string().into_boxed_str() ) );

  graph.current_set( "a" );

  assert_eq!( graph.current_name_get(), Some( "a".to_string().into_boxed_str() ) );
}

#[ test ]
fn animation_graph_node_add_test()
{
  let mut graph = create_graph();
  graph.update( 0.5 );

  let animation = create_animation();

  assert!( graph.node_get( "c" ).is_none() );

  graph.node_add( "c", animation );

  assert!( graph.node_get( "c" ).is_some() );
}

#[ test ]
fn animation_graph_node_remove_test()
{
  let mut graph = create_graph();
  graph.update( 0.5 );

  assert!( graph.node_get( "b" ).is_some() );

  graph.node_remove( "b" );

  assert!( graph.node_get( "b" ).is_none() );
}

#[ test ]
fn animation_graph_edge_add_test()
{
  let mut graph = create_graph();
  graph.update( 0.5 );

  graph.node_add( "c", create_animation() );

  assert!( graph.node_get( "a" ).is_some() );
  assert!( graph.node_get( "c" ).is_some() );
  assert!( graph.edge_get( "a", "ac" ).is_none() );

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::new() );
  let true_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    true
  };
  graph.edge_add( "a", "c", "ac", instant_tween, true_condition );

  assert!( graph.edge_get( "a", "ac" ).is_some() );
}

#[ test ]
fn animation_graph_edge_remove_test()
{
  let mut graph = create_graph();
  graph.update( 0.5 );

  assert!( graph.node_get( "a" ).is_some() );
  assert!( graph.node_get( "b" ).is_some() );
  assert!( graph.edge_get( "a", "ab" ).is_some() );

  graph.edge_remove( "a", "ab" );

  assert!( graph.edge_get( "a", "ac" ).is_none() );
}

#[ test ]
fn animation_graph_update_test()
{
  let mut graph = create_graph();
  graph.update( 0.5 );

  assert!( graph.node_get( "b" ).unwrap().progress() > 0.0 );
}
