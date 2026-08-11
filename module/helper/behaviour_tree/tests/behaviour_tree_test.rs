//! Behavior tests for `behaviour_tree` — context/blackboard state, composite and
//! decorator node semantics, builder, and convenience constructors, exercised
//! through the crate's public API only.

use std::time::Duration;
use behaviour_tree::*;

#[ test ]
fn test_behavior_context_creation()
{
  let context = BehaviorContext::new();
  assert!( context.entity_id.is_none() );
  assert!( context.blackboard.is_empty() );
  assert!( context.properties.is_empty() );
}

#[ test ]
fn test_behavior_context_blackboard()
{
  let mut context = BehaviorContext::new();
  context.set_blackboard( "health", 100 );
  context.set_blackboard( "position", ( 5, 10 ) );

  assert_eq!( context.get_blackboard( "health" ), Some( &BehaviorValue::Int( 100 ) ) );
  assert_eq!( context.get_blackboard( "position" ), Some( &BehaviorValue::Position2D { x : 5, y : 10 } ) );
  assert_eq!( context.get_blackboard( "missing" ), None );
}

#[ test ]
fn test_behavior_context_for_entity_and_properties()
{
  let mut context = BehaviorContext::for_entity( 7 );
  assert_eq!( context.entity_id, Some( 7 ) );

  context.set_property( "speed", 3 );
  assert_eq!( context.get_property( "speed" ), Some( &BehaviorValue::Int( 3 ) ) );
  assert_eq!( context.get_property( "missing" ), None );
}

#[ test ]
fn test_sequence_node_success()
{
  let mut sequence = SequenceNode::new
  (
    vec!
    [
      Box::new( SetBlackboardAction::new( "step1", true ) ),
      Box::new( SetBlackboardAction::new( "step2", true ) ),
    ]
  );

  let mut context = BehaviorContext::new();
  let status = sequence.execute( &mut context );

  assert_eq!( status, BehaviorStatus::Success );
  assert_eq!( context.get_blackboard( "step1" ), Some( &BehaviorValue::Bool( true ) ) );
  assert_eq!( context.get_blackboard( "step2" ), Some( &BehaviorValue::Bool( true ) ) );
}

#[ test ]
fn test_sequence_node_running()
{
  let mut sequence = SequenceNode::new
  (
    vec!
    [
      Box::new( SetBlackboardAction::new( "step1", true ) ),
      Box::new( WaitAction::new( 1.0 ) ), // This will be running
    ]
  );

  let mut context = BehaviorContext::new();
  let status = sequence.execute( &mut context );

  assert_eq!( status, BehaviorStatus::Running );
  assert_eq!( context.get_blackboard( "step1" ), Some( &BehaviorValue::Bool( true ) ) );
}

#[ test ]
fn test_selector_node()
{
  let mut selector = SelectorNode::new
  (
    vec!
    [
      Box::new( BlackboardCondition::new( "should_fail", true ) ), // This will fail
      Box::new( SetBlackboardAction::new( "executed", true ) ),    // This should execute
    ]
  );

  let mut context = BehaviorContext::new();
  context.set_blackboard( "should_fail", false ); // Make first condition fail

  let status = selector.execute( &mut context );

  assert_eq!( status, BehaviorStatus::Success );
  assert_eq!( context.get_blackboard( "executed" ), Some( &BehaviorValue::Bool( true ) ) );
}

#[ test ]
fn test_parallel_node()
{
  let mut parallel = ParallelNode::new
  (
    vec!
    [
      Box::new( SetBlackboardAction::new( "action1", true ) ),
      Box::new( SetBlackboardAction::new( "action2", true ) ),
    ]
  );

  let mut context = BehaviorContext::new();
  let status = parallel.execute( &mut context );

  assert_eq!( status, BehaviorStatus::Success );
  assert_eq!( context.get_blackboard( "action1" ), Some( &BehaviorValue::Bool( true ) ) );
  assert_eq!( context.get_blackboard( "action2" ), Some( &BehaviorValue::Bool( true ) ) );
}

#[ test ]
fn test_repeat_node()
{
  let mut repeat = RepeatNode::times
  (
    Box::new( SetBlackboardAction::new( "counter", 1 ) ),
    3
  );

  let mut context = BehaviorContext::new();
  let status = repeat.execute( &mut context );

  assert_eq!( status, BehaviorStatus::Success );
  // The action would have been executed 3 times, but since it just sets the same value,
  // we can't easily verify the count without more sophisticated tracking
}

#[ test ]
fn test_invert_node()
{
  let mut invert = InvertNode::new
  (
    Box::new( BlackboardCondition::new( "should_succeed", true ) )
  );

  let mut context = BehaviorContext::new();
  context.set_blackboard( "should_succeed", false ); // Make condition fail

  let status = invert.execute( &mut context );
  assert_eq!( status, BehaviorStatus::Success ); // Inverted failure becomes success
}

#[ test ]
fn test_wait_action()
{
  let mut wait = WaitAction::new( 0.1 ); // 100ms wait
  let mut context = BehaviorContext::new();

  // First execution should return Running
  let status1 = wait.execute( &mut context );
  assert_eq!( status1, BehaviorStatus::Running );

  // Simulate time passing
  std::thread::sleep( Duration::from_millis( 150 ) );
  context.update( Duration::from_millis( 150 ) );

  // Second execution should return Success
  let status2 = wait.execute( &mut context );
  assert_eq!( status2, BehaviorStatus::Success );
}

#[ test ]
fn test_blackboard_condition()
{
  let mut condition = BlackboardCondition::new( "health_low", true );
  let mut context = BehaviorContext::new();

  // Condition should fail when value doesn't exist
  assert_eq!( condition.execute( &mut context ), BehaviorStatus::Failure );

  // Condition should fail when value doesn't match
  context.set_blackboard( "health_low", false );
  assert_eq!( condition.execute( &mut context ), BehaviorStatus::Failure );

  // Condition should succeed when value matches
  context.set_blackboard( "health_low", true );
  assert_eq!( condition.execute( &mut context ), BehaviorStatus::Success );
}

#[ test ]
fn test_behavior_tree_builder()
{
  let tree = BehaviorTreeBuilder::new()
  .sequence
  (
    vec!
    [
      Box::new( SetBlackboardAction::new( "step1", true ) ),
      Box::new( SetBlackboardAction::new( "step2", true ) ),
    ]
  )
  .build_named( "TestTree".to_string() );

  assert_eq!( tree.name(), "TestTree" );
}

#[ test ]
fn test_convenience_functions()
{
  let node = sequence
  (
    vec!
    [
      set_blackboard( "init", true ),
      selector
      (
        vec!
        [
          condition( "enemy_near", true ),
          wait( 1.0 ),
        ]
      ),
      invert( condition( "health_full", false ) ),
    ]
  );

  let mut context = BehaviorContext::new();
  context.set_blackboard( "enemy_near", false );
  context.set_blackboard( "health_full", false );

  // We can't easily test the full execution without more setup,
  // but we can verify the node was created
  assert_eq!( node.name(), "Sequence" );
}

#[ test ]
fn test_cooldown_node()
{
  let mut cooldown = CooldownNode::new
  (
    Box::new( SetBlackboardAction::new( "executed", true ) ),
    Duration::from_millis( 100 )
  );
  let mut context = BehaviorContext::new();

  // First execution should succeed
  let status1 = cooldown.execute( &mut context );
  assert_eq!( status1, BehaviorStatus::Success );

  // Immediate second execution should fail (cooldown active)
  let status2 = cooldown.execute( &mut context );
  assert_eq!( status2, BehaviorStatus::Failure );

  // After cooldown period, should succeed again
  std::thread::sleep( Duration::from_millis( 150 ) );
  context.update( Duration::from_millis( 150 ) );
  let status3 = cooldown.execute( &mut context );
  assert_eq!( status3, BehaviorStatus::Success );
}

/// ## Root Cause
/// `RepeatNode::execute` re-invoked its child inside an unconditional
/// `loop { ... }`. The loop only had two exit branches: the child
/// returning `Running`, or `current_repeats` reaching `max_repeats`.
/// When a node is built via `RepeatNode::infinite` (`max_repeats ==
/// None`) and wraps a child that never returns `Running` (e.g. an
/// instant action/condition, or a Sequence/Selector made entirely of
/// instant nodes), neither branch is ever reachable, so the loop spins
/// forever inside a single call to `execute`, hanging the calling
/// thread (a livelock: full CPU use, zero progress toward returning).
///
/// ## Why Not Caught
/// The existing `test_repeat_node` only exercised the finite
/// `RepeatNode::times` path with a small count, which always terminates
/// in a handful of iterations regardless of this bug. Nothing exercised
/// `RepeatNode::infinite` paired with a child that never yields
/// `Running`, so the non-terminating branch was never reached.
///
/// ## Fix Applied
/// `RepeatNode::execute` now bounds synchronous child re-invocations
/// within a single call to `RepeatNode::MAX_SYNC_ITERATIONS`; once that
/// cap is hit without the child returning `Running` or `current_repeats`
/// reaching `max_repeats`, the node yields `BehaviorStatus::Running`
/// back to the caller instead of continuing to loop.
///
/// ## Prevention
/// This test runs the risky `execute()` call on a background thread
/// (building the whole tree inside that thread, since
/// `Box< dyn BehaviorNode >` is not `Send` and cannot cross the thread
/// boundary itself) and receives the result through a channel with
/// `recv_timeout`, so a real hang fails the test after a bounded wait
/// instead of blocking the suite. Confirmed against pre-fix source: this
/// assertion reproducibly timed out instead of observing `Running`.
///
/// ## Pitfall
/// A decorator that synchronously re-invokes an opaque
/// `Box< dyn BehaviorNode >` child in a loop must bound the number of
/// re-invocations independently of any user-supplied repeat count --
/// "infinite repeat" must mean "unbounded across ticks", never
/// "unbounded within a single tick".
#[ test ]
fn test_repeat_node_infinite_livelock_guard()
{
  let ( tx, rx ) = std::sync::mpsc::channel();

  // Built and executed entirely inside the spawned thread: `RepeatNode`
  // holds a `Box< dyn BehaviorNode >`, which is not `Send`, so only the
  // `BehaviorStatus` result is allowed to cross the channel.
  std::thread::spawn( move ||
  {
    let mut repeat = RepeatNode::infinite
    (
      Box::new( SetBlackboardAction::new( "tick", true ) ) // never returns Running
    );
    let mut context = BehaviorContext::new();
    let status = repeat.execute( &mut context );
    let _ = tx.send( status );
  } );

  let status = rx.recv_timeout( Duration::from_secs( 2 ) )
  .expect( "RepeatNode::infinite over a non-Running child hung past the bounded-time guard" );

  assert_eq!( status, BehaviorStatus::Running );
}
