//! Tools for managing [`AnimatableValue`] playback in every time moment 

mod private
{
  use std::collections::HashMap;

  use crate::
  {
    AnimationState,
    Animatable,
    Tween
  };

  /// Sequencer for managing multiple animations with sequencing and grouping.
  #[ derive( Debug ) ]
  pub struct Sequencer 
  {
    /// Map of animation names to their tween data
    tweens : HashMap< String, Box< dyn AnimatableValue > >,
    /// Current Sequencer time
    time : f32,
    /// Sequencer state
    state : AnimationState,
  }

  impl Sequencer
  {
    /// Creates a new animation Sequencer.
    pub fn new() -> Self 
    {
      Self 
      {
        tweens : HashMap::new(),
        time : 0.0,
        state : AnimationState::Pending,
      }
    }

    /// Adds a tween to the Sequencer.
    pub fn add_tween< T >( &mut self, name : &str, tween : Tween< T > ) 
    where T : Animatable + 'static
    {
      self.tweens.insert( name.to_string(), Box::new( tween ) );
      if self.state == AnimationState::Pending && !self.tweens.is_empty() 
      {
        self.state = AnimationState::Running;
      }
    }

    /// Updates all animations in the Sequencer.
    pub fn update( &mut self, delta_time : f32 ) 
    {
      if self.state != AnimationState::Running 
      {
        return;
      }

      self.time += delta_time;
      let mut all_completed = true;

      for tween in self.tweens.values_mut() 
      {
        tween.update( delta_time );
        if !tween.is_completed() 
        {
          all_completed = false;
        }
      }

      if all_completed && !self.tweens.is_empty() 
      {
        self.state = AnimationState::Completed;
      }
    }

    /// Gets the current value of a named animation.
    pub fn get_value< T >( &self, name : &str ) -> Option< T > 
    where T : Animatable + 'static
    {
      let tween_box = self.tweens.get( name )?;
      let any_ref = tween_box.as_any();
      if let Some( concrete_tween ) = any_ref.downcast_ref::< Tween< T > >() 
      {
        Some( concrete_tween.get_current_value() )
      } 
      else 
      {
        None
      }
    }

    /// Checks if the Sequencer has completed all animations.
    pub fn is_completed( &self ) -> bool 
    {
      self.state == AnimationState::Completed
    }

    /// Pauses all animations in the Sequencer.
    pub fn pause( &mut self ) 
    {
      self.state = AnimationState::Paused;
      for tween in self.tweens.values_mut() 
      {
        tween.pause();
      }
    }

    /// Resumes all animations in the Sequencer.
    pub fn resume( &mut self ) 
    {
      self.state = AnimationState::Running;
      for tween in self.tweens.values_mut() 
      {
        tween.resume();
      }
    }

    /// Resets the  Sequencer and all animations.
    pub fn reset( &mut self ) 
    {
      self.time = 0.0;
      self.state = if self.tweens.is_empty() 
      {
        AnimationState::Pending
      } 
      else 
      {
        AnimationState::Running
      };
      for tween in self.tweens.values_mut() 
      {
        tween.reset();
      }
    }

    /// Removes an animation from the Sequencer.
    pub fn remove_tween( &mut self, name : &str ) -> bool 
    {
      self.tweens.remove( name ).is_some()
    }

    /// Gets the current  Sequencer time.
    pub fn time( &self ) -> f32 
    {
      self.time
    }

    /// Gets the Sequencer state.
    pub fn state( &self ) -> AnimationState 
    {
      self.state
    }

    /// Gets the number of active animations.
    pub fn animation_count( &self ) -> usize 
    {
      self.tweens.len()
    }
  }

  impl Default for Sequencer 
  {
    fn default() -> Self 
    {
      Self::new()
    }
  }

  /// Trait for type-erased animatable values in Sequencer.
  pub trait AnimatableValue : std::fmt::Debug
  {
    /// Updates the animation state based on time.
    fn update( &mut self, delta_time : f32 );
    /// Returns true if the animation has completed.
    fn is_completed( &self ) -> bool;
    /// Pauses the animation.
    fn pause( &mut self );
    /// Resumes the animation.
    fn resume( &mut self );
    /// Resets the animation to its initial state.
    fn reset( &mut self );
    /// Returns a type-erased reference to the underlying value.
    fn as_any( &self ) -> &dyn std::any::Any;
  }

  #[ cfg( test ) ]
  mod tests 
  {
    use super::*;
    use crate::easing::
    { 
      base::EasingBuilder, 
      Linear, 
      EaseInSine 
    };
    
    #[ test ]
    fn test_sequencer_basic_flow() 
    {
      let mut sequencer = Sequencer::new();
      
      assert_eq!( sequencer.state(), AnimationState::Pending );
      assert_eq!( sequencer.animation_count(), 0 );
      
      let float_tween = Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() );
      sequencer.add_tween( "test", float_tween );
      
      assert_eq!( sequencer.state(), AnimationState::Running );
      assert_eq!( sequencer.animation_count(), 1 );
      assert!( !sequencer.is_completed() );
      
      sequencer.update( 0.5 );
      assert_eq!( sequencer.time(), 0.5 );
      assert_eq!( sequencer.state(), AnimationState::Running );
      
      let value = sequencer.get_value::< f32 >( "test" ).unwrap();
      assert_eq!( value, 5.0 );
      
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
      sequencer.add_tween( "short_tween", tween1 );
      sequencer.add_tween( "long_tween", tween2 );

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
      sequencer.add_tween
      ( 
        "test", 
        Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() ) 
      );
      
      sequencer.update( 0.5 );
      assert_eq!( sequencer.get_value::< f32 >( "test" ).unwrap(), 5.0 );
      
      sequencer.pause();
      assert_eq!( sequencer.state(), AnimationState::Paused );
      
      sequencer.update( 0.5 );
      let value = sequencer.get_value::< f32 >( "test" ).unwrap();
      assert_eq!( value, 5.0 );
      
      sequencer.resume();
      assert_eq!( sequencer.state(), AnimationState::Running );
      
      sequencer.update( 0.5 );
      assert!( sequencer.is_completed() );
      let value = sequencer.get_value::< f32 >( "test" ).unwrap();
      assert_eq!( value, 10.0 );
    }

    #[ test ]
    fn test_sequencer_reset() 
    {
      let mut sequencer = Sequencer::new();
      sequencer.add_tween
      ( 
        "test", 
        Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() ) 
      );
      
      sequencer.update( 0.5 );
      assert_eq!( sequencer.time(), 0.5 );
      assert_eq!( sequencer.get_value::< f32 >( "test" ).unwrap(), 5.0 );
      
      sequencer.reset();
      
      assert_eq!( sequencer.time(), 0.0 );
      assert_eq!( sequencer.state(), AnimationState::Running );
      assert_eq!( sequencer.get_value::< f32 >( "test" ).unwrap(), 0.0 );
      
      sequencer.update( 1.0 );
      assert!( sequencer.is_completed() );
      assert_eq!( sequencer.get_value::< f32 >( "test" ).unwrap(), 10.0 );
    }

    #[ test ]
    fn test_sequencer_remove_tween() 
    {
      let mut sequencer = Sequencer::new();
      
      sequencer.add_tween
      ( 
        "tween1", 
        Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::new() ) 
      );
      sequencer.add_tween
      ( 
        "tween2", 
        Tween::new( 0.0_f32, 1.0_f32, 1.0, Linear::new() ) 
      );
      assert_eq!( sequencer.animation_count(), 2 );
      
      assert!( sequencer.remove_tween( "tween1" ) );
      assert_eq!( sequencer.animation_count(), 1 );
      
      assert!( sequencer.get_value::< f32 >( "tween1" ).is_none() );
      assert!( sequencer.get_value::< f32 >( "tween2" ).is_some() );
      
      assert!( !sequencer.remove_tween( "tween1" ) );
    }

    #[ test ]
    fn test_sequencer_get_value_wrong_type() 
    {
      let mut sequencer = Sequencer::new();
      
      sequencer.add_tween
      ( 
        "float_tween", 
        Tween::new( 0.0_f32, 10.0_f32, 1.0, Linear::new() ) 
      );
      
      assert!( sequencer.get_value::< i32 >( "float_tween" ).is_none() );

      assert!( sequencer.get_value::< f32 >( "float_tween" ).is_some() );
    }

    #[ test ]
    fn test_sequencer_ease_in() 
    {
      let mut sequencer = Sequencer::new();
      
      sequencer.add_tween
      ( 
        "ease_in_tween", 
        Tween::new( 0.0_f32, 10.0_f32, 1.0, EaseInSine::new() ) 
      );
      
      sequencer.update( 0.5 );
      
      let value = sequencer.get_value::< f32 >( "ease_in_tween" ).unwrap();
      assert_eq!( value, 1.25 );
      
      sequencer.update( 0.5 );
      assert!( sequencer.is_completed() );
      let value = sequencer.get_value::< f32 >( "ease_in_tween" ).unwrap();
      assert_eq!( value, 10.0 );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    AnimatableValue,
    Sequencer
  };
}