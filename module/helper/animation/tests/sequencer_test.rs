//! Integration tests related to Sequencer struct

#[ cfg( test ) ]
mod tests
{
  use animation::
  {
    Tween,
    Sequencer,
    AnimationState,
    easing::
    {
      base::EasingBuilder,
      Linear,
      cubic::bezier::EaseInSine
    }
  };

  #[ test ]
  fn test_sequencer_basic_flow()
  {
    let mut sequencer = Sequencer::new();

    assert_eq!( sequencer.state(), AnimationState::Pending );
    assert_eq!( sequencer.animation_count(), 0 );

    let float_tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() );
    sequencer.add( "test", float_tween );

    assert_eq!( sequencer.state(), AnimationState::Running );
    assert_eq!( sequencer.animation_count(), 1 );
    assert!( !sequencer.is_completed() );

    sequencer.update( 0.5 );
    assert_eq!( sequencer.time(), 0.5 );
    assert_eq!( sequencer.state(), AnimationState::Running );

    let value = sequencer.get_value::< Tween< f32 > >( "test" ).unwrap();
    assert_eq!( value.get_current_value(), 5.0 );

    sequencer.update( 0.5 );
    assert_eq!( sequencer.time(), 1.0 );

    assert!( sequencer.is_completed() );
    assert_eq!( sequencer.state(), AnimationState::Completed );
  }

  #[ test ]
  fn test_sequencer_multiple_tweens()
  {
    let mut sequencer = Sequencer::new();

    let tween1 = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() );
    let tween2 = Tween::new( 0.0_f32, 10.0_f32, 2.0, Linear::new() );
    sequencer.add( "short_tween", tween1 );
    sequencer.add( "long_tween", tween2 );

    sequencer.update( 1.5 );

    assert!( !sequencer.is_completed() );
    assert_eq!( sequencer.state(), AnimationState::Running );
    assert_eq!( sequencer.time(), 1.5 );

    sequencer.update( 0.5 );

    assert!( sequencer.is_completed() );
    assert_eq!( sequencer.time(), 2.0 );
    assert_eq!( sequencer.state(), AnimationState::Completed );
  }

  #[ test ]
  fn test_sequencer_pause_resume()
  {
    let mut sequencer = Sequencer::new();
    sequencer.add
    (
      "test",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    );

    sequencer.update( 0.5 );
    assert_eq!( sequencer.get_value::< Tween< f32 > >( "test" ).unwrap().get_current_value(), 5.0 );

    sequencer.pause();
    assert_eq!( sequencer.state(), AnimationState::Paused );

    sequencer.update( 0.5 );
    let value = sequencer.get_value::< Tween< f32 > >( "test" ).unwrap();
    assert_eq!( value.get_current_value(), 5.0 );

    sequencer.resume();
    assert_eq!( sequencer.state(), AnimationState::Running );

    sequencer.update( 0.5 );
    assert!( sequencer.is_completed() );
    let value = sequencer.get_value::< Tween< f32 > >( "test" ).unwrap();
    assert_eq!( value.get_current_value(), 10.0 );
  }

  #[ test ]
  fn test_sequencer_reset()
  {
    let mut sequencer = Sequencer::new();
    sequencer.add
    (
      "test",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    );

    sequencer.update( 0.5 );
    assert_eq!( sequencer.time(), 0.5 );
    assert_eq!( sequencer.get_value::< Tween< f32 > >( "test" ).unwrap().get_current_value(), 5.0 );

    sequencer.reset();

    assert_eq!( sequencer.time(), 0.0 );
    assert_eq!( sequencer.state(), AnimationState::Running );
    assert_eq!( sequencer.get_value::< Tween< f32 > >( "test" ).unwrap().get_current_value(), 0.0 );

    sequencer.update( 1.0 );
    assert!( sequencer.is_completed() );
    assert_eq!( sequencer.get_value::< Tween< f32 > >( "test" ).unwrap().get_current_value(), 10.0 );
  }

  #[ test ]
  fn test_sequencer_remove()
  {
    let mut sequencer = Sequencer::new();

    sequencer.add
    (
      "tween1",
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::new() )
    );
    sequencer.add
    (
      "tween2",
      Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::new() )
    );
    assert_eq!( sequencer.animation_count(), 2 );

    assert!( sequencer.remove( "tween1" ) );
    assert_eq!( sequencer.animation_count(), 1 );

    assert!( sequencer.get_value::< Tween< f32 > >( "tween1" ).is_none() );
    assert!( sequencer.get_value::< Tween< f32 > >( "tween2" ).is_some() );

    assert!( !sequencer.remove( "tween1" ) );
  }

  #[ test ]
  fn test_sequencer_get_value_wrong_type()
  {
    let mut sequencer = Sequencer::new();

    sequencer.add
    (
      "float_tween",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() )
    );

    assert!( sequencer.get_value::< Tween< i32 > >( "float_tween" ).is_none() );

    assert!( sequencer.get_value::< Tween< f32 > >( "float_tween" ).is_some() );
  }

  #[ test ]
  fn test_sequencer_ease_in()
  {
    let mut sequencer = Sequencer::new();

    sequencer.add
    (
      "ease_in_tween",
      Tween::new( 0.0_f32, 10.0_f32, 1.0, EaseInSine::new() )
    );

    sequencer.update( 0.5 );

    let value = sequencer.get_value::< Tween< f32 > >( "ease_in_tween" ).unwrap();
    assert_eq!( value.get_current_value(), 1.25 );

    sequencer.update( 0.5 );
    assert!( sequencer.is_completed() );
    let value = sequencer.get_value::< Tween< f32 > >( "ease_in_tween" ).unwrap();
    assert_eq!( value.get_current_value(), 10.0 );
  }
}
