//! Tests for the `flowfield` module's public construction surface —
//! `IntegrationField` defaults, `FlowDirection` variants, and
//! `MultiGoalFlowField` creation.
//!
//! Relocated from `src/flowfield.rs` by task 072. Cross-module flowfield
//! scenarios (calculation calls, batch queries, multi-goal fields, ECS
//! interplay) live in `tests/integration/flowfield_tests.rs`, revived by
//! task 078. The two formerly-inline private-state tests moved here once the
//! `width()`/`height()`/`is_dirty()` getters made that state publicly
//! observable, per the all-tests-in-tests/ convention.

#![ cfg( feature = "enabled" ) ]


use tiles_tools::flowfield::{ DynamicFlowField, FlowDirection, FlowField, IntegrationField, MultiGoalFlowField };

#[ test ]
fn test_integration_field_creation()
{
  let integration = IntegrationField::< (), () >::new( 5, 5 );
  assert_eq!( integration.max_cost, u32::MAX );
}

#[ test ]
fn test_flow_direction_enum()
{
  let dir = FlowDirection::Move( 1, 0 );
  match dir
  {
    FlowDirection::Move( dx, dy ) =>
    {
      assert_eq!( dx, 1 );
      assert_eq!( dy, 0 );
    }
    FlowDirection::None => panic!( "Expected Move direction" ),
  }

  // Variant inequality pin (absorbed from the retired integration-suite
  // duplicate of this test by task 078).
  assert_ne!( FlowDirection::None, FlowDirection::Move( 1, 0 ) );
}

#[ test ]
fn test_multi_goal_flow_field_creation()
{
  let multi_field = MultiGoalFlowField::< (), () >::new( 8, 8 );
  assert_eq!( multi_field.goal_fields.len(), 0 );
}

/// Pins `FlowField::new`'s stored dimensions through the `width()`/`height()` getters.
#[ test ]
fn test_flow_field_creation()
{
  let flow_field = FlowField::< (), () >::new( 10, 10 );
  assert_eq!( flow_field.width(), 10 );
  assert_eq!( flow_field.height(), 10 );
}

/// Pins `DynamicFlowField::mark_dirty` accumulation through the `is_dirty()` query —
/// `incremental_update` consumes the set, so this is the only pre-consumption observable.
#[ test ]
fn test_dynamic_flow_field_dirty_marking()
{
  let mut dynamic_field = DynamicFlowField::< (), () >::new( 6, 6 );
  dynamic_field.mark_dirty( ( 3, 3 ) );
  assert!( dynamic_field.is_dirty( ( 3, 3 ) ) );
}
