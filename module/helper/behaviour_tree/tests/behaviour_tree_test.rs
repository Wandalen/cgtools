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
  context.blackboard_set( "health", 100 );
  context.blackboard_set( "position", ( 5, 10 ) );

  assert_eq!( context.blackboard_get( "health" ), Some( &BehaviorValue::Int( 100 ) ) );
  assert_eq!( context.blackboard_get( "position" ), Some( &BehaviorValue::Position2D { x : 5, y : 10 } ) );
  assert_eq!( context.blackboard_get( "missing" ), None );
}

#[ test ]
fn test_behavior_context_for_entity_and_properties()
{
  let mut context = BehaviorContext::for_entity( 7 );
  assert_eq!( context.entity_id, Some( 7 ) );

  context.property_set( "speed", 3 );
  assert_eq!( context.property_get( "speed" ), Some( &BehaviorValue::Int( 3 ) ) );
  assert_eq!( context.property_get( "missing" ), None );
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
  assert_eq!( context.blackboard_get( "step1" ), Some( &BehaviorValue::Bool( true ) ) );
  assert_eq!( context.blackboard_get( "step2" ), Some( &BehaviorValue::Bool( true ) ) );
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
  assert_eq!( context.blackboard_get( "step1" ), Some( &BehaviorValue::Bool( true ) ) );
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
  context.blackboard_set( "should_fail", false ); // Make first condition fail

  let status = selector.execute( &mut context );

  assert_eq!( status, BehaviorStatus::Success );
  assert_eq!( context.blackboard_get( "executed" ), Some( &BehaviorValue::Bool( true ) ) );
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
  assert_eq!( context.blackboard_get( "action1" ), Some( &BehaviorValue::Bool( true ) ) );
  assert_eq!( context.blackboard_get( "action2" ), Some( &BehaviorValue::Bool( true ) ) );
}

// test_kind: bug_reproducer(BUG-145)
/// ## Root Cause
/// `ParallelNode::execute` never called `self.reset()`/any `child.reset()` on any terminal
/// path, unlike `SequenceNode`/`SelectorNode` -- a sibling child still `Running` when another
/// child fails is abandoned with stale internal state.
/// ## Why Not Caught
/// The existing `test_parallel_node` only exercises a single activation where every child
/// succeeds together; nothing re-activates the same node a second time after an earlier
/// activation left a child `Running`.
/// ## Fix Applied
/// `execute` now calls `self.reset()` before returning any terminal `Success`/`Failure`,
/// cascading `child.reset()` to every child including abandoned `Running` ones.
/// ## Prevention
/// This test activates a `ParallelNode` where one child is a long `WaitAction` (starts
/// `Running`) and the other fails immediately, forcing a `Failure` return with the wait
/// abandoned mid-flight; a second, independent activation then confirms the wait restarted
/// from scratch (`Running`, not a stale-timer `Success`).
/// ## Pitfall
/// Invisible whenever a `ParallelNode` is only ever activated once, or every activation happens
/// to have all children complete together -- only a node reused across genuinely independent
/// activations, with an earlier `Running` child abandoned, exposes the stale state.
#[ test ]
fn test_parallel_node_resets_abandoned_running_child_on_failure()
{
  let mut parallel = ParallelNode::new
  (
    vec!
    [
      Box::new( WaitAction::new( 10.0 ) ),                // child 0: long-running wait
      Box::new( BlackboardCondition::new( "go", true ) ),  // child 1: fails initially
    ]
  );

  let mut context = BehaviorContext::new();
  context.blackboard_set( "go", false );

  // Tick 1: child 0 starts (Running), child 1 fails -> Parallel returns Failure,
  // abandoning child 0's in-flight wait.
  assert_eq!( parallel.execute( &mut context ), BehaviorStatus::Failure );

  // A lot of simulated time passes before this same node is independently reactivated.
  context.update( Duration::from_secs_f32( 20.0 ) );
  context.blackboard_set( "go", true );

  // Tick 2: a fresh, independent activation -- child 0 must need a full fresh 10s, not
  // instantly report Success off its abandoned tick-1 timer.
  assert_eq!( parallel.execute( &mut context ), BehaviorStatus::Running );
}

// test_kind: bug_reproducer(BUG-228)
/// ## Root Cause
/// `ParallelNode::execute` re-invoked EVERY child EVERY tick, with no memory of which children
/// had already reached `Success` in an earlier tick of the same still-`Running` activation.
/// ## Why Not Caught
/// `test_parallel_node` only exercises a single tick where every child completes together;
/// nothing re-ticks a `ParallelNode` where one child already succeeded but another is still
/// `Running`.
/// ## Fix Applied
/// `execute` now tracks a per-child `succeeded` flag and skips re-invoking any child already
/// marked `Success`, counting its remembered result instead.
/// ## Prevention
/// This test pairs a `CooldownNode` (succeeds tick 1, then fails any re-poll within its long
/// cooldown window) with a slower `WaitAction`. Tick 2 fails pre-fix (the cooldown child gets
/// wrongly re-polled and reports `Failure` from being back inside its own cooldown) and
/// succeeds post-fix (the cooldown child's tick-1 success is remembered, never re-polled).
/// ## Pitfall
/// A composite that keeps polling every child every tick must stop polling a child once it
/// reaches a terminal status -- re-polling an already-succeeded child can trigger that child's
/// own unrelated internal state (like a cooldown window) instead of just reconfirming success.
#[ test ]
fn test_parallel_node_does_not_repoll_already_succeeded_child()
{
  let mut parallel = ParallelNode::new
  (
    vec!
    [
      // child 0: succeeds tick 1, then FAILS if polled again within its 100s cooldown.
      Box::new( CooldownNode::new
      (
        Box::new( SetBlackboardAction::new( "fast_done", true ) ),
        Duration::from_secs_f32( 100.0 )
      ) ),
      // child 1: needs 1 simulated second to complete.
      Box::new( WaitAction::new( 1.0 ) ),
    ]
  );

  let mut context = BehaviorContext::new();

  // Tick 1: child 0 succeeds immediately (cooldown starts empty); child 1 starts its wait.
  assert_eq!( parallel.execute( &mut context ), BehaviorStatus::Running );

  // Fast-forward past child 1's 1s wait -- well within child 0's 100s cooldown window.
  context.update( Duration::from_secs_f32( 1.1 ) );

  // Tick 2: child 1 finishes. Child 0 must NOT be re-polled -- if it were, it would still be
  // inside its own 100s cooldown and report `Failure`, wrongly failing the whole composite even
  // though both children have, in truth, already succeeded.
  assert_eq!( parallel.execute( &mut context ), BehaviorStatus::Success );
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

// test_kind: bug_reproducer(BUG-146)
/// ## Root Cause
/// `RepeatNode::execute`'s completion check (`current_repeats >= max`) ran only AFTER
/// executing the child, so `RepeatNode::times( child, 0 )` still ran its child once on the
/// loop's first iteration before the check could ever fire.
/// ## Why Not Caught
/// The existing `test_repeat_node` only exercises `count = 3`; nothing exercised `count = 0`.
/// ## Fix Applied
/// The completion check now runs at the TOP of every loop iteration, before the child
/// executes, so a repeat count already satisfied (zero) short-circuits before any execution.
/// ## Prevention
/// This test builds `RepeatNode::times( .., 0 )` around an action with an observable side
/// effect (a blackboard write) and asserts the blackboard key was never set.
/// ## Pitfall
/// A "run N times" decorator that checks its stop condition only after acting is correct for
/// every N >= 1 but silently wrong at the N = 0 boundary -- check-then-act, not act-then-check,
/// whenever the stop condition can already hold before any work is done.
#[ test ]
fn test_repeat_node_zero_count_never_executes_child()
{
  let mut repeat = RepeatNode::times
  (
    Box::new( SetBlackboardAction::new( "ran", true ) ),
    0
  );

  let mut context = BehaviorContext::new();
  let status = repeat.execute( &mut context );

  assert_eq!( status, BehaviorStatus::Success );
  assert_eq!( context.blackboard_get( "ran" ), None );
}

#[ test ]
fn test_invert_node()
{
  let mut invert = InvertNode::new
  (
    Box::new( BlackboardCondition::new( "should_succeed", true ) )
  );

  let mut context = BehaviorContext::new();
  context.blackboard_set( "should_succeed", false ); // Make condition fail

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

// test_kind: bug_reproducer(BUG-144)
/// ## Root Cause
/// `BehaviorContext::update` resampled `current_time` from `Instant::now()` on every call,
/// discarding the caller-supplied `delta_time` entirely.
/// ## Why Not Caught
/// The only two timing-dependent tests (`test_wait_action`, `test_cooldown_node`) both paired
/// `context.update(...)` with a real `std::thread::sleep(...)` of the same duration immediately
/// before it -- real time elapsing masked `delta_time` being ignored.
/// ## Fix Applied
/// `update` now accumulates `self.current_time += delta_time` instead of re-sampling
/// `Instant::now()`, making simulated time fully caller-controlled.
/// ## Prevention
/// This test advances a `WaitAction` purely via `context.update(...)`, with NO real sleep at
/// all, and asserts the wait completes -- this is only possible if `delta_time` genuinely drives
/// `current_time`.
/// ## Pitfall
/// A "game time" field driven by `Instant::now()` instead of an accumulated `delta_time` is
/// deaf to fast-forward, replay, and paused (`delta_time = 0`) ticks -- only real wall-clock
/// time can ever complete a wait or tick down a cooldown.
#[ test ]
fn test_context_update_advances_purely_from_delta_time()
{
  let mut wait = WaitAction::new( 1.0 ); // needs 1 simulated second to complete
  let mut context = BehaviorContext::new();

  assert_eq!( wait.execute( &mut context ), BehaviorStatus::Running );

  // No real sleep anywhere -- fast-forward the *simulation* by 5 seconds.
  context.update( Duration::from_secs_f32( 5.0 ) );

  assert_eq!( wait.execute( &mut context ), BehaviorStatus::Success );
}

#[ test ]
fn test_blackboard_condition()
{
  let mut condition = BlackboardCondition::new( "health_low", true );
  let mut context = BehaviorContext::new();

  // Condition should fail when value doesn't exist
  assert_eq!( condition.execute( &mut context ), BehaviorStatus::Failure );

  // Condition should fail when value doesn't match
  context.blackboard_set( "health_low", false );
  assert_eq!( condition.execute( &mut context ), BehaviorStatus::Failure );

  // Condition should succeed when value matches
  context.blackboard_set( "health_low", true );
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
      blackboard_set( "init", true ),
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
  context.blackboard_set( "enemy_near", false );
  context.blackboard_set( "health_full", false );

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
