use browser_input::*;
use browser_input::mouse::MouseButton;
use minwebgl::math::I32x2;

fn ev( event_type : EventType ) -> Event
{
  Event { event_type, alt : false, ctrl : false, shift : false }
}

fn p( x : i32, y : i32 ) -> I32x2
{
  I32x2::from_array( [ x, y ] )
}

fn press( id : i32, x : i32, y : i32 ) -> Event
{
  ev( EventType::PointerButton( id, p( x, y ), MouseButton::Main, Action::Press ) )
}

fn release( id : i32, x : i32, y : i32 ) -> Event
{
  ev( EventType::PointerButton( id, p( x, y ), MouseButton::Main, Action::Release ) )
}

fn move_to( id : i32, x : i32, y : i32 ) -> Event
{
  ev( EventType::PointerMove( id, p( x, y ) ) )
}

fn cancel( id : i32 ) -> Event
{
  ev( EventType::PointerCancel( id ) )
}

#[ test ]
fn press_adds_one_entry()
{
  let mut state = State::new();
  apply_events_to_state( &mut state, &[ press( 1, 10, 20 ) ] );
  assert_eq!( state.active_pointers, [ ( 1, p( 10, 20 ) ) ] );
}

#[ test ]
fn two_presses_add_two_entries()
{
  let mut state = State::new();
  apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), press( 2, 30, 40 ) ] );
  assert_eq!( state.active_pointers.len(), 2 );
  assert!( state.active_pointers.contains( &( 1, p( 10, 20 ) ) ) );
  assert!( state.active_pointers.contains( &( 2, p( 30, 40 ) ) ) );
}

#[ test ]
fn move_updates_position()
{
  let mut state = State::new();
  apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), move_to( 1, 50, 60 ) ] );
  assert_eq!( state.active_pointers, [ ( 1, p( 50, 60 ) ) ] );
}

#[ test ]
fn release_removes_entry()
{
  let mut state = State::new();
  apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), press( 2, 30, 40 ), release( 1, 10, 20 ) ] );
  assert_eq!( state.active_pointers, [ ( 2, p( 30, 40 ) ) ] );
}

#[ test ]
fn cancel_removes_entry()
{
  let mut state = State::new();
  apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), press( 2, 30, 40 ), cancel( 2 ) ] );
  assert_eq!( state.active_pointers, [ ( 1, p( 10, 20 ) ) ] );
}

#[ test ]
fn duplicate_press_is_idempotent()
{
  let mut state = State::new();
  apply_events_to_state( &mut state, &[ press( 1, 10, 20 ), press( 1, 15, 25 ) ] );
  // Guard fires: second press for the same id does not add a duplicate entry.
  assert_eq!( state.active_pointers, [ ( 1, p( 10, 20 ) ) ] );
}

#[ test ]
fn full_sequence_ends_empty()
{
  let mut state = State::new();
  apply_events_to_state
  (
    &mut state,
    &
    [
      press( 1, 10, 20 ),
      press( 2, 30, 40 ),
      release( 1, 10, 20 ),
      cancel( 2 ),
    ]
  );
  assert!( state.active_pointers.is_empty() );
}
