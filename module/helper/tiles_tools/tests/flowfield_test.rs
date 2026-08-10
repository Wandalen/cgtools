//! Tests for the `flowfield` module's public construction surface —
//! `IntegrationField` defaults, `FlowDirection` variants, and
//! `MultiGoalFlowField` creation.
//!
//! Relocated from `src/flowfield.rs` by task 072. Cross-module flowfield
//! scenarios (calculation calls, batch queries, multi-goal fields, ECS
//! interplay) live in `tests/integration/flowfield_tests.rs`, revived by
//! task 078. Two inline tests remain in `src/flowfield.rs` as a documented
//! exception (they pin private fields with no public accessor).

#![ cfg( feature = "enabled" ) ]


use tiles_tools::flowfield::{ FlowDirection, IntegrationField, MultiGoalFlowField };

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
