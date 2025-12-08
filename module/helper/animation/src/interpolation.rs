//! Tweening system for smooth entity movement in tile-based games.
//!
//! This module provides comprehensive animation capabilities for creating smooth,
//! visually appealing movement and transformations in tile-based games. It supports
//! various easing functions, animation composition, and frame-based updates.
//!
//! # Animation System
//!
//! The animation system is built around tweening ( interpolation ) between values
//! over time. It supports animating positions, rotations, scales, and
//! custom properties with different easing functions.
//!
//! ## Core Concepts
//!
//! - **Tween**: Interpolates between start and end values over duration
//! - **Easing**: Mathematical functions that control animation timing
//! - **Animation**: Collection of tweens that can run sequentially or in parallel
//! - **Sequencer**: Manages multiple animations with precise timing control
//!
//! ## Supported Value Types
//!
//! - Position coordinates ( any coordinate system )
//! - Floating point values ( scale, rotation, opacity )
//! - Custom interpolatable values
//!

mod private
{
  use crate::sequencer::AnimatableValue;
  #[ allow( unused_imports ) ]
  use crate::easing::base::EasingBuilder;
  use crate::easing::base::EasingFunction;
  use minwebgl as gl;
  use gl::
  {
    NdFloat,
    F64x3,
    F32x3,
    Quat,
    MatEl
  };

  /// Trait for types that can be animated ( interpolated ).
  pub trait Animatable : Clone + core::fmt::Debug
  {
    /// Interpolates between two values at time t ( 0.0 to 1.0 ).
    fn interpolate( &self, other : &Self, t : f64 ) -> Self;
  }

  /// Animation state for tracking tween progress.
  #[ non_exhaustive ]
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub enum AnimationState
  {
    /// Animation hasn't started yet
    Pending,
    /// Animation is currently running
    Running,
    /// Animation has completed
    Completed,
    /// Animation is paused
    Paused,
  }

  /// Core tween structure for animating between two values.
  #[ derive( Debug ) ]
  pub struct Tween< T >
  {
    /// Starting value
    start_value : T,
    /// Target value
    end_value : T,
    /// Animation duration in seconds
    duration : f64,
    /// Current elapsed time
    elapsed : f64,
    /// Easing function to use
    easing : Box< dyn EasingFunction< AnimatableType = T > >,
    /// Current animation state
    state : AnimationState,
    /// Delay before animation starts
    delay : f64,
    /// Time remains before animation starts
    remain : f64,
    /// Number of times to repeat ( 0 = no repeat, -1 = infinite )
    repeat_count : i32,
    /// Current repeat iteration
    current_repeat : i32,
    /// Whether to reverse on repeat ( ping-pong )
    yoyo : bool,
  }

  impl< T > Tween< T >
  where T : Animatable
  {
    /// Creates a new tween animation.
    pub fn new
    (
      start : T,
      end : T,
      duration : f64,
      easing : Box< dyn EasingFunction< AnimatableType = T > >
    ) -> Self
    {
      Self
      {
        start_value : start,
        end_value : end,
        duration : duration.max( 0.001 ), // Minimum duration to avoid division by zero
        elapsed : 0.0,
        easing,
        state : AnimationState::Pending,
        delay : 0.0,
        remain : 0.0,
        repeat_count : 0,
        current_repeat : 0,
        yoyo : false,
      }
    }

    /// Sets a delay before the animation starts.
    pub fn with_delay( mut self, delay : f64 ) -> Self
    {
      self.delay = delay.max( 0.0 );
      self.remain = self.delay;
      self
    }

    /// Sets the number of times to repeat the animation.
    pub fn with_repeat( mut self, count : i32 ) -> Self
    {
      self.repeat_count = count;
      self
    }

    /// Enables yoyo mode ( reverse direction on repeat ).
    pub fn with_yoyo( mut self, yoyo : bool ) -> Self
    {
      self.yoyo = yoyo;
      self
    }

    /// Updates the tween with the elapsed time and returns current value.
    pub fn update( &mut self, delta_time : f64 ) -> T
    {
      let mut remaining_time = delta_time;

      match self.state
      {
        AnimationState::Pending =>
        {
          if self.remain > 0.0
          {
            let delay_consumed = remaining_time.min( self.remain );
            self.remain -= delay_consumed;
            remaining_time -= delay_consumed;

            if self.remain <= 0.0
            {
              self.state = AnimationState::Running;
            }
            else
            {
              return self.start_value.clone();
            }
          }
          else
          {
            self.state = AnimationState::Running;
          }
        }
        AnimationState::Paused | AnimationState::Completed =>
        {
          return self.get_current_value();
        }
        AnimationState::Running => {}
      }

      // Apply remaining time to animation
      if remaining_time > 0.0 && self.state == AnimationState::Running
      {
        self.elapsed += remaining_time;

        if self.elapsed >= self.duration
        {
          // Animation completed this frame
          if self.repeat_count != 0
          {
            self.handle_repeat();
          }
          else
          {
            self.state = AnimationState::Completed;
            self.elapsed = self.duration;
          }
        }
      }

      self.get_current_value()
    }

    /// Gets the current interpolated value without updating time.
    pub fn get_current_value( &self ) -> T
    {
      if self.state == AnimationState::Pending
      {
        return self.start_value.clone();
      }

      // Handle yoyo mode
      let ( start, end ) = if self.yoyo && self.current_repeat % 2 == 1
      {
        ( self.end_value.clone(), self.start_value.clone() )
      }
      else
      {
        ( self.start_value.clone(), self.end_value.clone() )
      };

      let normalized_time = ( self.elapsed / self.duration ).clamp( 0.0, 1.0 );
      self.easing.apply( start, end, normalized_time )
    }

    /// Handles animation repeat logic.
    fn handle_repeat( &mut self )
    {
      let elapsed_repeats = ( self.elapsed / self.duration ).floor();
      if self.repeat_count == -1
      {
        // Infinite repeat
        self.current_repeat += elapsed_repeats as i32;
        self.elapsed = ( self.elapsed - ( self.duration * elapsed_repeats ) ).min( 0.0 );
        self.state = AnimationState::Running;
      }
      else if self.repeat_count > 0 && self.current_repeat < self.repeat_count
      {
        // Finite repeat
        self.current_repeat += elapsed_repeats as i32;
        self.elapsed = ( self.elapsed - ( self.duration * elapsed_repeats ) ).min( 0.0 );
        self.state = AnimationState::Running;
      }
      else
      {
        // No repeats left or invalid repeat count
        self.state = AnimationState::Completed;
        self.elapsed = self.duration;
      }
    }

    /// Pauses the animation.
    pub fn pause( &mut self )
    {
      if self.state == AnimationState::Running
      {
        self.state = AnimationState::Paused;
      }
    }

    /// Resumes a paused animation.
    pub fn resume( &mut self )
    {
      if self.state == AnimationState::Paused
      {
        self.state = AnimationState::Running;
      }
    }

    /// Resets the animation to its starting state.
    pub fn reset( &mut self )
    {
      self.elapsed = 0.0;
      self.current_repeat = 0;
      self.remain = self.delay;
      self.state = if self.delay > 0.0
      {
        AnimationState::Pending
      }
      else
      {
        AnimationState::Running
      };
    }

    /// Checks if the animation is completed.
    pub fn is_completed( &self ) -> bool
    {
      self.state == AnimationState::Completed
    }

    /// Gets the current animation state.
    pub fn state( &self ) -> AnimationState
    {
      self.state
    }

    /// Gets the current repeat count.
    pub fn current_repeat( &self ) -> i32
    {
      self.current_repeat
    }

    /// Gets elapsed time
    pub fn time( &self ) -> f64
    {
      self.elapsed
    }
  }

  impl< T > AnimatableValue for Tween< T >
  where T : Animatable + 'static
  {
    fn update( &mut self, delta_time : f64 )
    {
      self.update( delta_time );
    }

    fn is_completed( &self ) -> bool
    {
      self.is_completed()
    }

    fn pause( &mut self )
    {
      self.pause();
    }

    fn resume( &mut self )
    {
      self.resume();
    }

    fn reset( &mut self )
    {
      self.reset();
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

  impl< T, const N : usize > AnimatableValue for [ Tween< T >; N ]
  where T : Animatable + 'static
  {
    fn update( &mut self, delta_time : f64 )
    {
      for tween in self
      {
        tween.update( delta_time );
      }
    }

    fn is_completed( &self ) -> bool
    {
      self.iter().all( | t | t.is_completed() )
    }

    fn pause( &mut self )
    {
      self.iter_mut()
      .for_each( | t | t.pause() );
    }

    fn resume( &mut self )
    {
      self.iter_mut()
      .for_each( | t | t.resume() );
    }

    fn reset( &mut self )
    {
      self.iter_mut()
      .for_each( | t | t.reset() );
    }

    fn as_any( &self ) -> &dyn core::any::Any
    {
      self
    }

    fn get_duration( &self ) -> f64
    {
      let mut min_start = 0.0;
      for t in self.iter()
      {
        min_start = t.delay.max( min_start );
      }

      let mut max_end = 0.0;
      for t in self.iter()
      {
        max_end = ( t.delay + t.duration ).max( max_end );
      }

      max_end - min_start
    }

    fn get_delay( &self ) -> f64
    {
      let mut min_delay = 0.0;
      for t in self.iter()
      {
        min_delay = t.delay.min( min_delay );
      }

      min_delay
    }

    fn progress( &self ) -> f64
    {
      if self[ 0 ].state == AnimationState::Pending
      {
        0.0
      }
      else
      {
        ( ( self[ 0 ].time() - self.get_delay() ) / self.get_duration() ).clamp( 0.0, 1.0 )
      }
    }
  }

  // === ANIMATABLE IMPLEMENTATIONS ===

  impl Animatable for f32
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      self + ( other - self ) * time as f32
    }
  }

  impl Animatable for f64
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      self + ( other - self ) * f64::from( time )
    }
  }

  impl Animatable for i32
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      ( *self as f64 + ( *other as f64 - *self as f64 ) * time ) as i32
    }
  }

  impl Animatable for ( f32, f32 )
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      (
        self.0.interpolate( &other.0, time ),
        self.1.interpolate( &other.1, time ),
      )
    }
  }

  impl Animatable for ( f64, f64 )
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      (
        self.0.interpolate( &other.0, time ),
        self.1.interpolate( &other.1, time ),
      )
    }
  }

  impl Animatable for ( i32, i32 )
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      (
        self.0.interpolate( &other.0, time ),
        self.1.interpolate( &other.1, time ),
      )
    }
  }

  impl Animatable for F64x3
  {
    fn interpolate(&self, other : &Self, time : f64 ) -> Self
    {
      Self::from
      (
        [
          self.x().interpolate( &other.x(), time ),
          self.y().interpolate( &other.y(), time ),
          self.z().interpolate( &other.z(), time )
        ]
      )
    }
  }

  impl Animatable for F32x3
  {
    fn interpolate(&self, other : &Self, time : f64 ) -> Self
    {
      Self::from
      (
        [
          self.x().interpolate( &other.x(), time ),
          self.y().interpolate( &other.y(), time ),
          self.z().interpolate( &other.z(), time )
        ]
      )
    }
  }

  impl< E > Animatable for Quat< E >
  where
    E : MatEl + core::fmt::Debug + NdFloat
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      self.slerp( other, E::from( time ).unwrap() )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    AnimationState,
    Tween,
    Animatable
  };
}
