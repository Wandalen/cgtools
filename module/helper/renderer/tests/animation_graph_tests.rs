//! Tests for animation composer AnimationGraph
#![ cfg( feature = "animation" ) ]

use animation::{ Sequence, Sequencer, Tween, easing::{ EasingBuilder, Linear } };
use mingl::{ F64x3, QuatF64 };
use renderer::webgl::animation::
{
  AnimatableComposition,
  AnimationEdge,
  AnimationGraph,
  Pose,
  base::
  {
    MORPH_TARGET_PREFIX,
    ROTATION_PREFIX,
    SCALE_PREFIX,
    TRANSLATION_PREFIX
  }
};
use rustc_hash::FxHashMap;

fn animation_create() -> Sequencer
{
  let mut animation = Sequencer::new();

  let linear = Linear::build();
  animation.insert
  (
    TRANSLATION_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( F64x3::splat( -1.0 ), F64x3::splat( 0.0 ), 0.5, linear.clone() ),
        Tween::new( F64x3::splat( 0.0 ), F64x3::splat( 1.0 ), 0.5, linear ).with_delay( 0.5 )
      ]
    ).unwrap()
  );

  let linear = Linear::build();
  animation.insert
  (
    ROTATION_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( QuatF64::from( [ -1.0, -1.0, -1.0, 1.0 ] ), QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ), 0.5, linear.clone() ),
        Tween::new( QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ), QuatF64::from( [ 1.0, 1.0, 1.0, 1.0 ] ), 0.5, linear ).with_delay( 0.5 )
      ]
    ).unwrap()
  );

  let linear = Linear::build();
  animation.insert
  (
    SCALE_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( F64x3::splat( 1.0 ), F64x3::splat( 2.0 ), 0.5, linear.clone() ),
        Tween::new( F64x3::splat( 2.0 ), F64x3::splat( 3.0 ), 0.5, linear ).with_delay( 0.5 )
      ]
    ).unwrap()
  );

  let linear = Linear::build();
  animation.insert
  (
    MORPH_TARGET_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( vec![ 0.5, 0.5, 0.5 ], vec![ 0.75, 0.75, 0.75 ], 0.5, linear.clone() ),
        Tween::new( vec![ 0.75, 0.75, 0.75 ], vec![ 1.0, 1.0, 1.0 ], 0.5, linear ).with_delay( 0.5 )
      ]
    ).unwrap()
  );

  animation
}

/// A single-channel animation whose total logical duration ( 100s ) is far longer than any
/// test drives it for, so its Sequencer never reaches `Completed` and `.time()` accumulates
/// as a plain, exact running total of every `delta_time` passed to `update()`.
fn long_animation_create() -> Sequencer
{
  let mut animation = Sequencer::new();

  let linear = Linear::build();
  animation.insert
  (
    TRANSLATION_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( F64x3::splat( 0.0 ), F64x3::splat( 1.0 ), 50.0, linear.clone() ),
        Tween::new( F64x3::splat( 1.0 ), F64x3::splat( 2.0 ), 50.0, linear ).with_delay( 50.0 )
      ]
    ).unwrap()
  );

  animation
}

fn graph_create() -> AnimationGraph
{
  let animation = animation_create();
  let mut graph = AnimationGraph::new( &FxHashMap::default() );

  graph.node_add( "a", animation.clone() );
  graph.node_add( "b", animation );

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::build() );
  let true_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    true
  };
  graph.edge_add( "a", "b", "ab", instant_tween, true_condition );

  graph
}

#[ test ]
fn animation_graph_conditions_test()
{
  let mut graph = AnimationGraph::new( &FxHashMap::default() );
  let animation = animation_create();

  graph.node_add( "a", animation.clone() );
  graph.node_add( "b", animation.clone() );
  graph.node_add( "c", animation );

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::build() );
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
  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "c".to_string().into_boxed_str() ) );
}

#[ test ]
fn animation_graph_current_name_get_test()
{
  let mut graph = graph_create();
  graph.update( 0.5 );
  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "b".to_string().into_boxed_str() ) );
}

#[ test ]
fn animation_graph_current_set_test()
{
  let mut graph = graph_create();
  graph.update( 0.5 );
  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "b".to_string().into_boxed_str() ) );

  graph.current_set( "a" );

  assert_eq!( graph.current_name_get(), Some( "a".to_string().into_boxed_str() ) );
}

#[ test ]
fn animation_graph_node_add_test()
{
  let mut graph = graph_create();
  graph.update( 0.5 );
  graph.update( 0.5 );

  let animation = animation_create();

  assert!( graph.node_get( "c" ).is_none() );

  graph.node_add( "c", animation );

  assert!( graph.node_get( "c" ).is_some() );
}

#[ test ]
fn animation_graph_node_remove_test()
{
  let mut graph = graph_create();
  graph.update( 0.5 );
  graph.update( 0.5 );

  assert!( graph.node_get( "b" ).is_some() );

  graph.node_remove( "b" );

  assert!( graph.node_get( "b" ).is_none() );
}

#[ test ]
fn animation_graph_edge_add_test()
{
  let mut graph = graph_create();
  graph.update( 0.5 );
  graph.update( 0.5 );

  graph.node_add( "c", animation_create() );

  assert!( graph.node_get( "a" ).is_some() );
  assert!( graph.node_get( "c" ).is_some() );
  assert!( graph.edge_get( "a", "ac" ).is_none() );

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::build() );
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
  let mut graph = graph_create();
  graph.update( 0.5 );
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
  let mut graph = graph_create();

  graph.update( 0.75 );
  graph.update( 0.75 );

  assert_eq!( graph.current_name_get(), Some( "b".to_string().into_boxed_str() ) );
}

/// ## Root Cause
/// `AnimationGraph::update`'s transition-completion handler synced the re-entered node's own
/// persistent Sequencer via `next.animation.update( time )`, without ever resetting it first.
/// A node re-entered after having played earlier still has its old elapsed time sitting there,
/// so the sync added the transition's end time on top of that stale leftover instead of
/// starting the new activation cleanly.
///
/// ## Why Not Caught
/// No pre-existing test drove the graph through a re-entry ( A -> B -> A ) scenario or asserted
/// on any node's own Sequencer elapsed time after a transition -- the existing tests only check
/// `current_name_get()` or node/edge existence via `node_get`/`edge_get`.
///
/// ## Fix Applied
/// `graph.rs`'s `is_transited` block now calls `next.animation.reset()` immediately before
/// `next.animation.update( time )`, matching the "reset-before-use" idiom already established
/// elsewhere in the same file ( the normal-playback branch, and `Transition::update` ).
///
/// ## Prevention
/// This test lets "a" accumulate 3.0s of elapsed time, transitions away to "b", lets "b" play
/// for a while, then transitions back to "a" and asserts its Sequencer reads exactly 3.5 ( the
/// transition's own end time, from a clean reset ) rather than 6.5 ( 3.5 added on top of the
/// stale 3.0s ). All deltas are multiples of 0.5, exactly representable in `f64`, so the
/// pre-fix/post-fix values are unambiguous and not just close.
///
/// ## Pitfall
/// A per-node persistent Sequencer that free-runs while its node is not `current` is easy to
/// assume "hasn't moved" -- but nothing suspends it either; the only thing preventing corruption
/// is that no code just reads it back without a reset in between two activations.
// test_kind: bug_reproducer(BUG-187)
#[ test ]
fn animation_graph_reentry_resets_stale_elapsed_time_test()
{
  let mut graph = AnimationGraph::new( &FxHashMap::default() );

  graph.node_add( "a", long_animation_create() );
  graph.node_add( "b", long_animation_create() );

  // Let "a" accumulate elapsed time on its own persistent Sequencer before any edge exists.
  graph.update( 1.0 );
  graph.update( 1.0 );
  graph.update( 1.0 );

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::build() );
  let true_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    true
  };
  graph.edge_add( "a", "b", "ab", instant_tween.clone(), true_condition );

  // Transition a -> b ( a zero-duration tween completes across exactly 2 update() calls ).
  graph.update( 0.5 );
  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "b".to_string().into_boxed_str() ) );

  // Let "b" play normally for a while.
  graph.update( 1.0 );
  graph.update( 1.0 );

  let true_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    true
  };
  graph.edge_add( "b", "a", "ba", instant_tween, true_condition );

  // Transition b -> a, RE-ENTERING "a" -- whose own Sequencer still holds the 3.0s of elapsed
  // time it accumulated before it was first exited.
  graph.update( 0.5 );
  graph.update( 0.5 );

  assert_eq!( graph.current_name_get(), Some( "a".to_string().into_boxed_str() ) );

  let got = graph.node_get( "a" ).unwrap().time();

  assert!
  (
    ( got - 3.5 ).abs() < 1e-9,
    "re-entering \"a\" must sync its own Sequencer from a clean ( reset ) state to the \
    transition's own end time ( 3.5 ), not add that time on top of the 3.0s of stale elapsed \
    time \"a\" accumulated before it was first exited ( which would give 6.5 instead ) -- got {got}"
  );
}
