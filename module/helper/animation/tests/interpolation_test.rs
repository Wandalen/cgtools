//! Integration tests related to Tween struct and trait Animatable

#[ cfg( test ) ]
mod tests
{
  use animation::
  {
    Tween,
    Animatable,
    AnimatableValue,
    AnimationState,
    easing::base::{ EasingBuilder, Linear }
  };

  // --- Animatable Trait Tests ---

  #[ test ]
  fn test_f32_interpolation()
  {
    let start = 10.0_f32;
    let end = 20.0_f32;
    assert_eq!( start.interpolate( &end, 0.0 ), 10.0 );
    assert_eq!( start.interpolate( &end, 1.0 ), 20.0 );
    assert_eq!( start.interpolate( &end, 0.5 ), 15.0 );
  }

  #[ test ]
  fn test_i32_interpolation()
  {
    let start = 5_i32;
    let end = 15_i32;
    assert_eq!( start.interpolate( &end, 0.0 ), 5 );
    assert_eq!( start.interpolate( &end, 1.0 ), 15 );
    assert_eq!( start.interpolate( &end, 0.5 ), 10 );
  }

  // --- Tween Core Logic Tests ---

  #[ test ]
  fn test_tween_initial_state()
  {
    let tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() );
    assert_eq!( tween.state(), AnimationState::Pending );
    assert_eq!( tween.progress(), 0.0 );
    assert!( !tween.is_completed() );
  }

  #[ test ]
  fn test_tween_progress_and_completion()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() );

    let val1 = tween.update( 0.5 );
    assert_eq!( tween.state(), AnimationState::Running );
    assert_eq!( val1, 5.0 );
    assert_eq!( tween.progress(), 0.5 );

    let val2 = tween.update( 0.5 );
    assert_eq!( tween.state(), AnimationState::Completed );
    assert_eq!( val2, 10.0 );
    assert_eq!( tween.progress(), 1.0 );
    assert!( tween.is_completed() );
  }

  #[ test ]
  fn test_tween_with_delay_behavior()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    .with_delay( 0.5 );

    // First update: still in delay
    let val1 = tween.update( 0.2 );
    assert_eq!( val1, 0.0 );
    assert_eq!( tween.state(), AnimationState::Pending );

    // Second update: delay ends, animation starts
    let val2 = tween.update( 0.3 ); // 0.2 + 0.3 = 0.5 total elapsed time
    assert_eq!( tween.state(), AnimationState::Running );
    assert_eq!( val2, 0.0 ); // Since 0 remaining time for animation

    // Third update: animates
    let val3 = tween.update( 0.5 );
    assert_eq!( tween.state(), AnimationState::Running );
    assert_eq!( val3, 5.0 );
  }

  #[ test ]
  fn test_tween_pause_resume()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 2.0, Linear::new() );
    tween.update( 0.5 ); // Progress to 2.5
    assert_eq!( tween.get_current_value(), 2.5 );

    tween.pause();
    assert_eq!( tween.state(), AnimationState::Paused );

    let val = tween.update( 1.0 ); // Update while paused, value should not change
    assert_eq!( val, 2.5 );
    assert_eq!( tween.state(), AnimationState::Paused );

    tween.resume();
    assert_eq!( tween.state(), AnimationState::Running );

    let val2 = tween.update( 1.5 ); // Update for remaining duration
    assert_eq!( val2, 10.0 );
    assert!( tween.is_completed() );
  }

  #[ test ]
  fn test_tween_finite_repeat()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() ).with_repeat( 2 );

    tween.update( 1.0 ); // First loop finishes
    assert!( !tween.is_completed() );
    assert_eq!( tween.current_repeat(), 1 );

    tween.update( 1.0 ); // Second loop finishes
    assert!( !tween.is_completed() );
    assert_eq!( tween.current_repeat(), 2 );

    tween.update( 1.0 ); // Third loop finishes, which is the final repeat
    assert!( tween.is_completed() );
  }

  #[ test ]
  fn test_tween_infinite_repeat()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    .with_repeat( -1 );

    tween.update( 1.0 );
    assert!( !tween.is_completed() );
    assert_eq!( tween.current_repeat(), 1 );

    tween.update( 10.0 );
    assert!( !tween.is_completed() );
    assert_eq!( tween.current_repeat(), 11 );
  }

  #[ test ]
  fn test_tween_yoyo_with_repeat()
  {
    let mut tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    .with_repeat( 1 ).with_yoyo( true );

    // First loop: 0.0 -> 10.0
    let val1 = tween.update( 0.5 );
    assert_eq!( val1, 5.0 );
    tween.update( 0.5 );
    assert_eq!( tween.get_current_value(), 10.0 );
    assert_eq!( tween.current_repeat(), 1 );

    // Second loop: 10.0 -> 0.0 (yoyo)
    let val2 = tween.update( 0.5 );
    assert_eq!( val2, 5.0 );
    tween.update( 0.5 );
    assert_eq!( tween.get_current_value(), 0.0 );
    assert!( tween.is_completed() );
  }
}
