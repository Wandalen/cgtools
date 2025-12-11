//! Behavior Tree system for advanced AI decision making in tile-based games.
//!
//! This module provides a comprehensive behavior tree implementation for creating
//! sophisticated AI behaviors in game entities. Behavior trees are hierarchical
//! structures that model decision-making processes, commonly used in game AI
//! for NPCs, enemies, and autonomous agents.
//!
//! # Behavior Trees
//!
//! A behavior tree is composed of nodes that represent actions, conditions, and
//! control flow logic. The tree is executed from the root, and each node returns
//! a status indicating success, failure, or that it's still running.
//!
//! ## Node Types
//!
//! - **Action Nodes**: Perform specific actions (move, attack, patrol)
//! - **Condition Nodes**: Check game state conditions (health low, enemy near)
//! - **Composite Nodes**: Control execution flow (sequence, selector, parallel)
//! - **Decorator Nodes**: Modify child behavior (repeat, invert, cooldown)
//!
//! ## Execution Flow
//!
//! - **Success**: Node completed successfully
//! - **Failure**: Node failed to complete
//! - **Running**: Node is still executing (multi-frame actions)
//!
//! # Examples
//!
//! ```
//! use behaviour_tree::*;
//!
//! // Create a simple patrol behavior using blackboard values
//! let mut patrol_tree = BehaviorTreeBuilder::new()
//! .sequence
//! (
//!   vec!
//!   [
//!     set_blackboard( "target_x", 10 ),
//!     set_blackboard( "target_y", 10 ),
//!     wait( 2.0 ), // Wait 2 seconds
//!     set_blackboard( "target_x", 5 ),
//!     set_blackboard( "target_y", 5 ),
//!     wait( 2.0 ),
//!   ]
//! )
//! .build();
//!
//! // Execute the behavior tree
//! let mut context = BehaviorContext::new();
//! let status = patrol_tree.execute( &mut context );
//! ```

use std::collections::HashMap;
use core::time::Duration;
use std::time::Instant;

/// Status returned by behavior tree nodes during execution.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
#[ non_exhaustive ]
pub enum BehaviorStatus
{
  /// Node completed successfully
  Success,
  /// Node failed to complete
  Failure,
  /// Node is still executing (will continue next frame)
  Running,
}

/// Context passed through behavior tree execution containing game state.
#[ derive( Debug ) ]
#[ non_exhaustive ]
pub struct BehaviorContext
{
  /// Entity ID this behavior tree belongs to
  pub entity_id : Option< u32 >,
  /// Current game time for time-based behaviors
  pub current_time : Instant,
  /// Delta time since last execution
  pub delta_time : Duration,
  /// Blackboard for storing behavior-specific data
  pub blackboard : HashMap< String, BehaviorValue >,
  /// Custom properties for game-specific data
  pub properties : HashMap< String, BehaviorValue >,
}

impl BehaviorContext
{
  /// Creates a new behavior context.
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Self
  {
    Self
    {
      entity_id : None,
      current_time : Instant::now(),
      delta_time : Duration::from_secs_f32( 1.0 / 60.0 ), // Default 60 FPS
      blackboard : HashMap::new(),
      properties : HashMap::new(),
    }
  }

  /// Creates a context for a specific entity.
  #[ inline ]
  #[ must_use ]
  pub fn for_entity( entity_id : u32 ) -> Self
  {
    let mut context = Self::new();
    context.entity_id = Some( entity_id );
    context
  }

  /// Updates the context with new timing information.
  #[ inline ]
  pub fn update( &mut self, delta_time : Duration )
  {
    self.delta_time = delta_time;
    self.current_time = Instant::now();
  }

  /// Sets a value in the blackboard.
  #[ inline ]
  pub fn set_blackboard< T : Into< BehaviorValue > >( &mut self, key : &str, value : T )
  {
    self.blackboard.insert( key.to_string(), value.into() );
  }

  /// Gets a value from the blackboard.
  #[ inline ]
  #[ must_use ]
  pub fn get_blackboard( &self, key : &str ) -> Option< &BehaviorValue >
  {
    self.blackboard.get( key )
  }

  /// Sets a property value.
  #[ inline ]
  pub fn set_property< T : Into< BehaviorValue > >( &mut self, key : &str, value : T )
  {
    self.properties.insert( key.to_string(), value.into() );
  }

  /// Gets a property value.
  #[ inline ]
  #[ must_use ]
  pub fn get_property( &self, key : &str ) -> Option< &BehaviorValue >
  {
    self.properties.get( key )
  }
}

impl Default for BehaviorContext
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

/// Flexible value type for blackboard and property storage.
#[ derive( Debug, Clone, PartialEq ) ]
#[ non_exhaustive ]
pub enum BehaviorValue
{
  /// Boolean value
  Bool( bool ),
  /// 32-bit signed integer
  Int( i32 ),
  /// 32-bit unsigned integer
  UInt( u32 ),
  /// 32-bit floating point number
  Float( f32 ),
  /// String value
  String( String ),
  /// 2D position with x and y coordinates
  Position2D
  {
    /// X coordinate
    x : i32,
    /// Y coordinate
    y : i32
  },
  /// Entity identifier
  EntityId( u32 ),
}

impl From< bool > for BehaviorValue
{
  #[ inline ]
  fn from( value : bool ) -> Self
  {
    BehaviorValue::Bool( value )
  }
}

impl From< i32 > for BehaviorValue
{
  #[ inline ]
  fn from( value : i32 ) -> Self
  {
    BehaviorValue::Int( value )
  }
}

impl From< f32 > for BehaviorValue
{
  #[ inline ]
  fn from( value : f32 ) -> Self
  {
    BehaviorValue::Float( value )
  }
}

impl From< String > for BehaviorValue
{
  #[ inline ]
  fn from( value : String ) -> Self
  {
    BehaviorValue::String( value )
  }
}

impl From< &str > for BehaviorValue
{
  #[ inline ]
  fn from( value : &str ) -> Self
  {
    BehaviorValue::String( value.to_string() )
  }
}

impl From< u32 > for BehaviorValue
{
  #[ inline ]
  fn from( value : u32 ) -> Self
  {
    BehaviorValue::EntityId( value )
  }
}

impl From< ( i32, i32 ) > for BehaviorValue
{
  #[ inline ]
  fn from( ( x, y ) : ( i32, i32 ) ) -> Self
  {
    BehaviorValue::Position2D { x, y }
  }
}

/// Core trait for all behavior tree nodes.
pub trait BehaviorNode : core::fmt::Debug
{
  /// Executes this node and returns the resulting status.
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus;

  /// Resets the node to its initial state.
  #[ inline ]
  fn reset( &mut self ) {}

  /// Gets a human-readable name for this node (for debugging).
  fn name( &self ) -> &str;
}

/// Root behavior tree structure containing the tree hierarchy.
#[ derive( Debug ) ]
pub struct BehaviorTree
{
  /// Root node of the behavior tree
  root : Box< dyn BehaviorNode >,
  /// Name identifier for this behavior tree
  name : String,
}

impl BehaviorTree
{
  /// Creates a new behavior tree with the given root node.
  #[ inline ]
  #[ must_use ]
  pub fn new( root : Box< dyn BehaviorNode >, name : String ) -> Self
  {
    Self { root, name }
  }

  /// Executes the behavior tree and returns the root status.
  #[ inline ]
  #[ must_use ]
  pub fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    self.root.execute( context )
  }

  /// Resets the entire behavior tree to its initial state.
  #[ inline ]
  pub fn reset( &mut self )
  {
    self.root.reset();
  }

  /// Gets the name of this behavior tree.
  #[ inline ]
  #[ must_use ]
  pub fn name( &self ) -> &str
  {
    &self.name
  }
}

// === COMPOSITE NODES ===

/// Executes children in sequence until one fails or all succeed.
#[ derive( Debug ) ]
pub struct SequenceNode
{
  children : Vec< Box< dyn BehaviorNode > >,
  current_child : usize,
  name : String,
}

impl SequenceNode
{
  /// Creates a new sequence node with the given children.
  #[ inline ]
  #[ must_use ]
  pub fn new( children : Vec< Box< dyn BehaviorNode > > ) -> Self
  {
    Self
    {
      children,
      current_child : 0,
      name : "Sequence".to_string(),
    }
  }

  /// Creates a named sequence node.
  #[ inline ]
  #[ must_use ]
  pub fn named( children : Vec< Box< dyn BehaviorNode > >, name : String ) -> Self
  {
    Self { children, current_child : 0, name }
  }
}

impl BehaviorNode for SequenceNode
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    while self.current_child < self.children.len()
    {
      match self.children[ self.current_child ].execute( context )
      {
        BehaviorStatus::Success =>
        {
          self.current_child += 1;
        }
        BehaviorStatus::Failure =>
        {
          self.reset();
          return BehaviorStatus::Failure;
        }
        BehaviorStatus::Running =>
        {
          return BehaviorStatus::Running;
        }
      }
    }

    self.reset();
    BehaviorStatus::Success
  }

  #[ inline ]
  fn reset( &mut self )
  {
    self.current_child = 0;
    for child in &mut self.children
    {
      child.reset();
    }
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

/// Executes children until one succeeds or all fail.
#[ derive( Debug ) ]
pub struct SelectorNode
{
  children : Vec< Box< dyn BehaviorNode > >,
  current_child : usize,
  name : String,
}

impl SelectorNode
{
  /// Creates a new selector node with the given children.
  #[ inline ]
  #[ must_use ]
  pub fn new( children : Vec< Box< dyn BehaviorNode > > ) -> Self
  {
    Self
    {
      children,
      current_child : 0,
      name : "Selector".to_string(),
    }
  }

  /// Creates a named selector node.
  #[ inline ]
  #[ must_use ]
  pub fn named( children : Vec< Box< dyn BehaviorNode > >, name : String ) -> Self
  {
    Self { children, current_child : 0, name }
  }
}

impl BehaviorNode for SelectorNode
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    while self.current_child < self.children.len()
    {
      match self.children[ self.current_child ].execute( context )
      {
        BehaviorStatus::Success =>
        {
          self.reset();
          return BehaviorStatus::Success;
        }
        BehaviorStatus::Failure =>
        {
          self.current_child += 1;
        }
        BehaviorStatus::Running =>
        {
          return BehaviorStatus::Running;
        }
      }
    }

    self.reset();
    BehaviorStatus::Failure
  }

  #[ inline ]
  fn reset( &mut self )
  {
    self.current_child = 0;
    for child in &mut self.children
    {
      child.reset();
    }
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

/// Executes all children in parallel, succeeding when all succeed.
#[ derive( Debug ) ]
pub struct ParallelNode
{
  children : Vec< Box< dyn BehaviorNode > >,
  name : String,
}

impl ParallelNode
{
  /// Creates a new parallel node with the given children.
  #[ inline ]
  #[ must_use ]
  pub fn new( children : Vec< Box< dyn BehaviorNode > > ) -> Self
  {
    Self
    {
      children,
      name : "Parallel".to_string(),
    }
  }

  /// Creates a named parallel node.
  #[ inline ]
  #[ must_use ]
  pub fn named( children : Vec< Box< dyn BehaviorNode > >, name : String ) -> Self
  {
    Self { children, name }
  }
}

impl BehaviorNode for ParallelNode
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    let mut running_count = 0;
    let mut success_count = 0;

    for child in &mut self.children
    {
      match child.execute( context )
      {
        BehaviorStatus::Success => success_count += 1,
        BehaviorStatus::Failure => return BehaviorStatus::Failure,
        BehaviorStatus::Running => running_count += 1,
      }
    }

    if running_count > 0
    {
      BehaviorStatus::Running
    }
    else if success_count == self.children.len()
    {
      BehaviorStatus::Success
    }
    else
    {
      BehaviorStatus::Failure
    }
  }

  #[ inline ]
  fn reset( &mut self )
  {
    for child in &mut self.children
    {
      child.reset();
    }
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

// === DECORATOR NODES ===

/// Repeats a child node a specified number of times or indefinitely.
#[ derive( Debug ) ]
pub struct RepeatNode
{
  child : Box< dyn BehaviorNode >,
  max_repeats : Option< u32 >,
  current_repeats : u32,
  name : String,
}

impl RepeatNode
{
  /// Creates a repeat node that runs indefinitely.
  #[ inline ]
  #[ must_use ]
  pub fn infinite( child : Box< dyn BehaviorNode > ) -> Self
  {
    Self
    {
      child,
      max_repeats : None,
      current_repeats : 0,
      name : "Repeat(∞)".to_string(),
    }
  }

  /// Creates a repeat node that runs a specific number of times.
  #[ inline ]
  #[ must_use ]
  pub fn times( child : Box< dyn BehaviorNode >, count : u32 ) -> Self
  {
    Self
    {
      child,
      max_repeats : Some( count ),
      current_repeats : 0,
      name : format!( "Repeat({count})" ),
    }
  }
}

impl BehaviorNode for RepeatNode
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    loop
    {
      match self.child.execute( context )
      {
        BehaviorStatus::Running => return BehaviorStatus::Running,
        BehaviorStatus::Success | BehaviorStatus::Failure =>
        {
          self.current_repeats += 1;
          self.child.reset();

          if let Some( max ) = self.max_repeats && self.current_repeats >= max
          {
            self.reset();
            return BehaviorStatus::Success;
          }
          // Continue looping for infinite repeat or more iterations
        }
      }
    }
  }

  #[ inline ]
  fn reset( &mut self )
  {
    self.current_repeats = 0;
    self.child.reset();
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

/// Inverts the success/failure status of its child.
#[ derive( Debug ) ]
pub struct InvertNode
{
  child : Box< dyn BehaviorNode >,
  name : String,
}

impl InvertNode
{
  /// Creates a new invert decorator.
  #[ inline ]
  #[ must_use ]
  pub fn new( child : Box< dyn BehaviorNode > ) -> Self
  {
    Self
    {
      child,
      name : "Invert".to_string(),
    }
  }
}

impl BehaviorNode for InvertNode
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    match self.child.execute( context )
    {
      BehaviorStatus::Success => BehaviorStatus::Failure,
      BehaviorStatus::Failure => BehaviorStatus::Success,
      BehaviorStatus::Running => BehaviorStatus::Running,
    }
  }

  #[ inline ]
  fn reset( &mut self )
  {
    self.child.reset();
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

/// Adds a cooldown period before allowing child execution.
#[ derive( Debug ) ]
pub struct CooldownNode
{
  child : Box< dyn BehaviorNode >,
  cooldown_duration : Duration,
  last_execution : Option< Instant >,
  name : String,
}

impl CooldownNode
{
  /// Creates a new cooldown decorator.
  #[ inline ]
  #[ must_use ]
  pub fn new( child : Box< dyn BehaviorNode >, cooldown_duration : Duration ) -> Self
  {
    Self
    {
      child,
      cooldown_duration,
      last_execution : None,
      name : format!( "Cooldown({:.1}s)", cooldown_duration.as_secs_f32() ),
    }
  }
}

impl BehaviorNode for CooldownNode
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    if let Some( last ) = self.last_execution && context.current_time.duration_since( last ) < self.cooldown_duration
    {
      return BehaviorStatus::Failure;
    }

    let result = self.child.execute( context );
    if result != BehaviorStatus::Running
    {
      self.last_execution = Some( context.current_time );
    }
    result
  }

  #[ inline ]
  fn reset( &mut self )
  {
    self.child.reset();
    // Note: Don't reset last_execution as cooldown persists across resets
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

// === CONDITION NODES ===

/// Checks if a blackboard value meets a condition.
#[ derive( Debug ) ]
pub struct BlackboardCondition
{
  key : String,
  expected_value : BehaviorValue,
  name : String,
}

impl BlackboardCondition
{
  /// Creates a new blackboard condition.
  #[ inline ]
  #[ must_use ]
  pub fn new< T : Into< BehaviorValue > >( key : &str, expected_value : T ) -> Self
  {
    let expected = expected_value.into();
    Self
    {
      key : key.to_string(),
      expected_value : expected.clone(),
      name : format!( "Check({key})" ),
    }
  }
}

impl BehaviorNode for BlackboardCondition
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    if let Some( value ) = context.get_blackboard( &self.key )
    {
      if *value == self.expected_value
      {
        BehaviorStatus::Success
      }
      else
      {
        BehaviorStatus::Failure
      }
    }
    else
    {
      BehaviorStatus::Failure
    }
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

// === ACTION NODES ===

/// Waits for a specified duration.
#[ derive( Debug ) ]
pub struct WaitAction
{
  duration : Duration,
  start_time : Option< Instant >,
  name : String,
}

impl WaitAction
{
  /// Creates a new wait action.
  #[ inline ]
  #[ must_use ]
  pub fn new( seconds : f32 ) -> Self
  {
    Self
    {
      duration : Duration::from_secs_f32( seconds ),
      start_time : None,
      name : format!( "Wait({seconds:.1}s)" ),
    }
  }
}

impl BehaviorNode for WaitAction
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    if self.start_time.is_none()
    {
      self.start_time = Some( context.current_time );
    }

    if let Some( start ) = self.start_time
    {
      if context.current_time.duration_since( start ) >= self.duration
      {
        self.reset();
        BehaviorStatus::Success
      }
      else
      {
        BehaviorStatus::Running
      }
    }
    else
    {
      BehaviorStatus::Running
    }
  }

  #[ inline ]
  fn reset( &mut self )
  {
    self.start_time = None;
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

/// Sets a value in the blackboard.
#[ derive( Debug ) ]
pub struct SetBlackboardAction
{
  key : String,
  value : BehaviorValue,
  name : String,
}

impl SetBlackboardAction
{
  /// Creates a new set blackboard action.
  #[ inline ]
  #[ must_use ]
  pub fn new< T : Into< BehaviorValue > >( key : &str, value : T ) -> Self
  {
    Self
    {
      key : key.to_string(),
      value : value.into(),
      name : format!( "Set({key})" ),
    }
  }
}

impl BehaviorNode for SetBlackboardAction
{
  #[ inline ]
  fn execute( &mut self, context : &mut BehaviorContext ) -> BehaviorStatus
  {
    context.set_blackboard( &self.key, self.value.clone() );
    BehaviorStatus::Success
  }

  #[ inline ]
  fn name( &self ) -> &str
  {
    &self.name
  }
}

// === BUILDER PATTERN ===

/// Builder for constructing behavior trees with fluent API.
#[ derive( Debug ) ]
pub struct BehaviorTreeBuilder
{
  root : Option< Box< dyn BehaviorNode > >,
}

impl BehaviorTreeBuilder
{
  /// Creates a new behavior tree builder.
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Self
  {
    Self { root : None }
  }

  /// Sets the root node to a sequence.
  #[ inline ]
  #[ must_use ]
  pub fn sequence( mut self, children : Vec< Box< dyn BehaviorNode > > ) -> Self
  {
    self.root = Some( Box::new( SequenceNode::new( children ) ) );
    self
  }

  /// Sets the root node to a selector.
  #[ inline ]
  #[ must_use ]
  pub fn selector( mut self, children : Vec< Box< dyn BehaviorNode > > ) -> Self
  {
    self.root = Some( Box::new( SelectorNode::new( children ) ) );
    self
  }

  /// Sets the root node to a parallel node.
  #[ inline ]
  #[ must_use ]
  pub fn parallel( mut self, children : Vec< Box< dyn BehaviorNode > > ) -> Self
  {
    self.root = Some( Box::new( ParallelNode::new( children ) ) );
    self
  }

  /// Sets a custom root node.
  #[ inline ]
  #[ must_use ]
  pub fn root( mut self, root : Box< dyn BehaviorNode > ) -> Self
  {
    self.root = Some( root );
    self
  }

  /// Builds the behavior tree with default name.
  #[ inline ]
  #[ must_use ]
  pub fn build( self ) -> BehaviorTree
  {
    self.build_named( "BehaviorTree".to_string() )
  }

  /// Builds the behavior tree with a specific name.
  ///
  /// # Panics
  ///
  /// Panics if root is not set.
  #[ inline ]
  #[ must_use ]
  pub fn build_named( self, name : String ) -> BehaviorTree
  {
    let root = self.root.expect( "Root node must be set before building" );
    BehaviorTree::new( root, name )
  }
}

impl Default for BehaviorTreeBuilder
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

// === CONVENIENCE FUNCTIONS ===

/// Creates a sequence node.
#[ inline ]
#[ must_use ]
pub fn sequence( children : Vec< Box< dyn BehaviorNode > > ) -> Box< dyn BehaviorNode >
{
  Box::new( SequenceNode::new( children ) )
}

/// Creates a selector node.
#[ inline ]
#[ must_use ]
pub fn selector( children : Vec< Box< dyn BehaviorNode > > ) -> Box< dyn BehaviorNode >
{
  Box::new( SelectorNode::new( children ) )
}

/// Creates a parallel node.
#[ inline ]
#[ must_use ]
pub fn parallel( children : Vec< Box< dyn BehaviorNode > > ) -> Box< dyn BehaviorNode >
{
  Box::new( ParallelNode::new( children ) )
}

/// Creates a repeat decorator.
#[ inline ]
#[ must_use ]
pub fn repeat( child : Box< dyn BehaviorNode >, count : u32 ) -> Box< dyn BehaviorNode >
{
  Box::new( RepeatNode::times( child, count ) )
}

/// Creates an infinite repeat decorator.
#[ inline ]
#[ must_use ]
pub fn repeat_forever( child : Box< dyn BehaviorNode > ) -> Box< dyn BehaviorNode >
{
  Box::new( RepeatNode::infinite( child ) )
}

/// Creates an invert decorator.
#[ inline ]
#[ must_use ]
pub fn invert( child : Box< dyn BehaviorNode > ) -> Box< dyn BehaviorNode >
{
  Box::new( InvertNode::new( child ) )
}

/// Creates a cooldown decorator.
#[ inline ]
#[ must_use ]
pub fn cooldown( child : Box< dyn BehaviorNode >, seconds : f32 ) -> Box< dyn BehaviorNode >
{
  Box::new( CooldownNode::new( child, Duration::from_secs_f32( seconds ) ) )
}

/// Creates a wait action.
#[ inline ]
#[ must_use ]
pub fn wait( seconds : f32 ) -> Box< dyn BehaviorNode >
{
  Box::new( WaitAction::new( seconds ) )
}

/// Creates a blackboard condition.
#[ inline ]
#[ must_use ]
pub fn condition< T : Into< BehaviorValue > >( key : &str, expected : T ) -> Box< dyn BehaviorNode >
{
  Box::new( BlackboardCondition::new( key, expected ) )
}

/// Creates a set blackboard action.
#[ inline ]
#[ must_use ]
pub fn set_blackboard< T : Into< BehaviorValue > >( key : &str, value : T ) -> Box< dyn BehaviorNode >
{
  Box::new( SetBlackboardAction::new( key, value ) )
}
#[ cfg( test ) ]
mod tests
{
  use super::*;
  use std::time::Duration;

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
}
