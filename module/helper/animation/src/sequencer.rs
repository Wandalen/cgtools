//! Tools for managing [`AnimatableValue`] playback in every time moment

mod private
{
  use std::collections::HashMap;
  use error_tools::*;
  use crate::AnimationState;
  #[ allow( unused_imports ) ]
  use crate::Tween;

  /// Sequencer for managing multiple animations with sequencing and grouping.
  #[ derive( Debug ) ]
  pub struct Sequencer
  {
    /// Map of animation names to their tween data
    tweens : HashMap< Box< str >, Box< dyn AnimatableValue > >,
    /// Current Sequencer time
    time : f64,
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

    /// Returns list of contained [`AnimatableValue`]'s names
    pub fn keys( &self ) -> Vec< Box< str > >
    {
      self.tweens.keys().cloned()
      .collect::< Vec< _ > >()
    }

    /// Adds a [`AnimatableValue`] to the Sequencer.
    pub fn add< T >( &mut self, name : &str, tween : T )
    where T : AnimatableValue + 'static
    {
      self.tweens.insert( name.to_string().into(), Box::new( tween ) );
      if self.state == AnimationState::Pending && !self.tweens.is_empty()
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
    pub fn get_value< T >( &self, name : &str ) -> Option< &T >
    where T : AnimatableValue + 'static
    {
      let tween_box = self.tweens.get( name )?;
      let any_ref = tween_box.as_any();
      any_ref.downcast_ref::< T >()
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
    pub fn remove( &mut self, name : &str ) -> bool
    {
      self.tweens.remove( name ).is_some()
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
  pub trait AnimatableValue : core::fmt::Debug
  {
    /// Updates the animation state based on time.
    fn update( &mut self, delta_time : f64 );
    /// Returns true if the animation has completed.
    fn is_completed( &self ) -> bool;
    /// Pauses the animation.
    fn pause( &mut self );
    /// Resumes the animation.
    fn resume( &mut self );
    /// Resets the animation to its initial state.
    fn reset( &mut self );
    /// Returns a type-erased reference to the underlying value.
    fn as_any( &self ) -> &dyn core::any::Any;
    /// Returns animation duration
    fn get_duration( &self ) -> f64;
    /// Returns animation delay
    fn get_delay( &self ) -> f64;
    /// Gets the progress of the animated value ( 0.0 to 1.0 ).
    fn progress( &self ) -> f64;
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

  /// Sequence of [`AnimatableValue`]s of one type
  #[ derive( Debug ) ]
  pub struct Sequence< T >
  {
    /// Sequence of [`AnimatableValue`]s of one type
    tweens : Vec< T >,
    /// Current [`AnimatableValue`] index
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
  where T : AnimatableValue + 'static
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

    /// Returns active [`AnimatableValue`] at current elapsed time
    pub fn get_current( &self ) -> Option< &T >
    {
      self.tweens.get( self.current )
    }

    /// Returns elapsed time
    pub fn time( &self ) -> f64
    {
      self.elapsed
    }
  }

  impl< T > AnimatableValue for Sequence< T >
  where T : AnimatableValue + 'static
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

    fn as_any( &self ) -> &dyn core::any::Any
    {
      self
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
  }
}

crate::mod_interface!
{
  orphan use
  {
    AnimatableValue,
    Sequencer,
    Sequence,
    SequenceError
  };
}
