//! Tools for managing [`AnimatablePlayer`] playback in every time moment

mod private
{
  use std::collections::HashMap;
  use error_tools::*;
  use crate::
  {
    AnimatablePlayer, AnimationState
  };
  #[ allow( unused_imports ) ]
  use crate::Tween;

  /// Sequencer for managing multiple animations with sequencing and grouping.
  #[ derive( Debug ) ]
  pub struct Sequencer
  {
    /// Map of animation names to their animation behavior data
    players : HashMap< Box< str >, Box< dyn AnimatablePlayer > >,
    /// Current Sequencer time
    time : f64,
    /// Sequencer state
    state : AnimationState,
  }

  impl Clone for Sequencer
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        players : self.players.iter()
        .map
        (
          | ( k, v ) |
          {
            ( k.clone(), clone_dyn_types::clone_into_box( v.as_ref() ) )
          }
        )
        .collect::< HashMap< _, _ > >(),
        time : self.time.clone(),
        state : self.state.clone()
      }
    }
  }

  impl Sequencer
  {
    /// Creates a new animation Sequencer.
    pub fn new() -> Self
    {
      Self
      {
        players : HashMap::new(),
        time : 0.0,
        state : AnimationState::Pending,
      }
    }

    /// Gets the current value of a named animation.
    pub fn get_value< T >( &self, name : &str ) -> Option< &T >
    where T : AnimatablePlayer + 'static
    {
      let player_box = self.players.get( name )?;
      let any_ref = player_box.as_any();
      any_ref.downcast_ref::< T >()
    }

    /// Returns list of contained [`AnimatablePlayer`]'s names
    pub fn keys( &self ) -> Vec< Box< str > >
    {
      self.players.keys().cloned()
      .collect::< Vec< _ > >()
    }

    /// Adds a [`AnimatablePlayer`] to the Sequencer.
    pub fn add< T >( &mut self, name : &str, player : T )
    where T : AnimatablePlayer + 'static
    {
      self.players.insert( name.to_string().into(), Box::new( player ) );
      if self.state == AnimationState::Pending && !self.players.is_empty()
      {
        self.state = AnimationState::Running;
      }
    }

    /// Updates all animations in the Sequencer.
    pub fn update( &mut self, delta_time : f64 )
    {
      if self.state != AnimationState::Running
      {
        return;
      }

      self.time += delta_time;
      let mut all_completed = true;

      for player in self.players.values_mut()
      {
        player.update( delta_time );
        if !player.is_completed()
        {
          all_completed = false;
        }
      }

      if all_completed && !self.players.is_empty()
      {
        self.state = AnimationState::Completed;
      }
    }

    /// Gets reference to named player
    pub fn get< T >( &self, name : &str ) -> Option< &T >
    where T : AnimatablePlayer + 'static
    {
      let player_box = self.players.get( name )?;
      let any_ref = player_box.as_any();
      any_ref.downcast_ref::< T >()
    }

    /// Gets mutable reference to named player
    pub fn get_mut< T >( &mut self, name : &str ) -> Option< &mut T >
    where T : AnimatablePlayer + 'static
    {
      let player_box = self.players.get_mut( name )?;
      player_box.as_any_mut().downcast_mut::< T >()
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
      for player in self.players.values_mut()
      {
        player.pause();
      }
    }

    /// Resumes all animations in the Sequencer.
    pub fn resume( &mut self )
    {
      self.state = AnimationState::Running;
      for player in self.players.values_mut()
      {
        player.resume();
      }
    }

    /// Resets the  Sequencer and all animations.
    pub fn reset( &mut self )
    {
      self.time = 0.0;
      self.state = if self.players.is_empty()
      {
        AnimationState::Pending
      }
      else
      {
        AnimationState::Running
      };
      for player in self.players.values_mut()
      {
        player.reset();
      }
    }

    /// Removes an animation from the Sequencer.
    pub fn remove( &mut self, name : &str ) -> bool
    {
      self.players.remove( name ).is_some()
    }

    /// Gets the current  Sequencer time.
    pub fn time( &self ) -> f64
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
      self.players.len()
    }
  }

  impl Default for Sequencer
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  /// Error for handling wrong [`Sequence`] input data
  #[ derive( Debug, error::typed::Error ) ]
  pub enum SequenceError
  {
    /// Input tweens aren't sorted in time
    #[ error( "Input tweens aren't sorted by delay" ) ]
    Unsorted,
    /// Input tweens count isn't enough for animation
    #[ error( "Input tweens count isn't enough for animation" ) ]
    NotEnough
  }

  /// Sequence of [`AnimatablePlayer`]s of one type
  #[ derive( Debug, Clone ) ]
  pub struct Sequence< T >
  {
    /// Sequence of [`AnimatablePlayer`]s of one type
    tweens : Vec< T >,
    /// Current [`AnimatablePlayer`] index
    current : usize,
    /// Animation duration in seconds
    duration : f64,
    /// Current elapsed time
    elapsed : f64,
    /// Current animation state
    state : AnimationState,
    /// Delay before animation starts
    delay : f64,
  }

  impl< T > Sequence< T >
  where T : AnimatablePlayer + 'static
  {
    /// [`Sequence`] constructor
    pub fn new( mut tweens : Vec< T > ) -> Result< Self, SequenceError >
    {
      if tweens.len() < 2
      {
        return Err( SequenceError::NotEnough );
      }

      let last_delay = 0.0;
      for t in tweens.iter_mut()
      {
        if last_delay > t.get_delay()
        {
          return Err( SequenceError::Unsorted );
        }
      }

      let delay = tweens.first().unwrap().get_delay();
      let tween = tweens.last().unwrap();
      let duration = tween.get_delay() + tween.get_duration() - delay;

      Ok
      (
        Self
        {
          tweens,
          current : 0,
          duration,
          elapsed : 0.0,
          state : AnimationState::Pending,
          delay
        }
      )
    }

    /// Returns active [`AnimatablePlayer`] at current elapsed time
    pub fn get_current( &self ) -> Option< &T >
    {
      self.tweens.get( self.current )
    }

    /// Returns active [`AnimatablePlayer`] index in tweens array
    pub fn get_current_id( &self ) -> usize
    {
      self.current
    }

    /// Returns all sequence of [`Tween`]'s
    pub fn get_tweens( &self ) -> Vec< T >
    where T : Clone
    {
      self.tweens.clone()
    }

    /// Returns elapsed time
    pub fn time( &self ) -> f64
    {
      self.elapsed
    }
  }

  impl< T > AnimatablePlayer for Sequence< T >
  where T : AnimatablePlayer + Clone + 'static
  {
    fn update( &mut self, delta_time : f64 )
    {
      if self.state == AnimationState::Completed || self.state == AnimationState::Paused
      {
        return;
      }

      self.elapsed += delta_time;

      let index = self.tweens.binary_search_by
      (
        | t |
        {
          t.get_delay().partial_cmp( &self.elapsed ).expect( "Animation keyframes can't be NaN" )
        }
      );

      let index = match index
      {
        Ok( id ) | Err( id ) => id
      };

      let mut current_id = index;

      if index >= self.tweens.len()
      {
        current_id = self.tweens.len().saturating_sub( 1 );
      }

      if self.current == current_id
      {
        let Some( current ) = self.tweens.get_mut( self.current )
        else
        {
          return;
        };
        let old_elapsed = current.get_delay() + ( current.progress() * current.get_duration() );
        current.update( old_elapsed + delta_time );
      }
      else if self.current < current_id
      {
        self.current = current_id;
        let Some( current ) = self.tweens.get_mut( self.current )
        else
        {
          return;
        };
        current.update( self.elapsed );
      }

      let Some( current ) = self.tweens.get_mut( self.current )
      else
      {
        return;
      };

      match self.state
      {
        AnimationState::Pending if self.elapsed - current.get_delay() > 0.0 =>
        {
          self.state = AnimationState::Running;
        },
        AnimationState::Running
        if self.current >= self.tweens.len() - 1 &&
        self.tweens.get( self.current ).map( | t | t.is_completed() )
        .unwrap_or( true ) =>
        {
          self.state = AnimationState::Completed;
        },
        _ => {}
      }
    }

    fn is_completed( &self ) -> bool
    {
      self.state == AnimationState::Completed
    }

    fn pause( &mut self )
    {
      if self.state == AnimationState::Running
      {
        self.state = AnimationState::Paused;
      }

      self.tweens.iter_mut()
      .for_each( | t | t.pause() );
    }

    fn resume( &mut self )
    {
      if self.state == AnimationState::Paused
      {
        self.state = AnimationState::Running;
      }

      self.tweens.iter_mut()
      .for_each( | t | t.resume() );
    }

    fn reset( &mut self )
    {
      self.current = 0;
      self.state = AnimationState::Pending;
      self.elapsed = 0.0;

      self.tweens.iter_mut()
      .for_each( | t | t.reset() );
    }

    fn get_duration( &self ) -> f64
    {
      self.duration
    }

    fn get_delay( &self ) -> f64
    {
      self.delay
    }

    fn progress( &self ) -> f64
    {
      if self.state == AnimationState::Pending
      {
        0.0
      }
      else
      {
        ( ( self.elapsed - self.delay ) / self.duration ).clamp( 0.0, 1.0 )
      }
    }

    fn as_any( &self ) -> &dyn core::any::Any
    {
      self
    }

    fn as_any_mut( &mut self ) -> &mut dyn core::any::Any
    {
      self
    }
  }

  #[ cfg( test ) ]
  mod tests
  {
    use super::*;
    use crate::easing::
    {
      base::EasingBuilder,
      Linear,
      cubic::bezier::EaseInSine
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
      assert_eq!( value.get_value(), 5.0 );

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
      assert_eq!( sequencer.get_value::< Tween< f32 > >( "test" ).unwrap().get_value(), 5.0 );

      sequencer.pause();
      assert_eq!( sequencer.state(), AnimationState::Paused );

      sequencer.update( 0.5 );
      let value = sequencer.get_value::< Tween< f32 > >( "test" ).unwrap();
      assert_eq!( value.get_value(), 5.0 );

      sequencer.resume();
      assert_eq!( sequencer.state(), AnimationState::Running );

      sequencer.update( 0.5 );
      assert!( sequencer.is_completed() );
      let value = sequencer.get_value::< Tween< f32 > >( "test" ).unwrap();
      assert_eq!( value.get_value(), 10.0 );
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
      assert_eq!( sequencer.get_value::< Tween< f32 > >( "test" ).unwrap().get_value(), 5.0 );

      sequencer.reset();

      assert_eq!( sequencer.time(), 0.0 );
      assert_eq!( sequencer.state(), AnimationState::Running );
      assert_eq!( sequencer.get_value::< Tween< f32 > >( "test" ).unwrap().get_value(), 0.0 );

      sequencer.update( 1.0 );
      assert!( sequencer.is_completed() );
      assert_eq!( sequencer.get_value::< Tween< f32 > >( "test" ).unwrap().get_value(), 10.0 );
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
      assert_eq!( value.get_value(), 1.25 );

      sequencer.update( 0.5 );
      assert!( sequencer.is_completed() );
      let value = sequencer.get_value::< Tween< f32 > >( "ease_in_tween" ).unwrap();
      assert_eq!( value.get_value(), 10.0 );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Sequencer,
    Sequence,
    SequenceError
  };
}
