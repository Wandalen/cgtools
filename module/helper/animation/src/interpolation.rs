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
  use crate::traits::{ Animatable, AnimatablePlayer };
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
    pub start_value : T,
    /// Target value
    pub end_value : T,
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

  impl< T > Clone for Tween< T >
  where T : Animatable + Clone + 'static
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        start_value : self.start_value.clone(),
        end_value : self.end_value.clone(),
        duration : self.duration.clone(),
        elapsed : self.elapsed.clone(),
        easing : clone_dyn_types::clone_into_box( &*self.easing ) ,
        state : self.state.clone(),
        delay : self.delay.clone(),
        remain : self.remain.clone(),
        repeat_count : self.repeat_count.clone(),
        current_repeat : self.current_repeat.clone(),
        yoyo : self.yoyo.clone()
      }
    }
  }

  impl< T > Tween< T >
  where T : Animatable + 'static
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

    /// Sets an animation duration
    pub fn with_duration( mut self, duration : f64 ) -> Self
    {
      self.duration = duration.max( 0.0 );
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
          return self.value_get();
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
            self.repeat_handle();
          }
          else
          {
            self.state = AnimationState::Completed;
            self.elapsed = self.duration;
          }
        }
      }

      self.value_get()
    }

    /// Returns current interpolated value
    pub fn value_get( &self ) -> T
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
    fn repeat_handle( &mut self )
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

  impl< T > AnimatablePlayer for Tween< T >
  where T : Animatable + Clone + 'static
  {
    fn update( &mut self, delta_time : f64 )
    {
      self.update( delta_time );
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
    }

    fn resume( &mut self )
    {
      if self.state == AnimationState::Paused
      {
        self.state = AnimationState::Running;
      }
    }

    fn reset( &mut self )
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

    fn duration_get( &self ) -> f64
    {
      self.duration
    }

    fn delay_get( &self ) -> f64
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

  impl< T, const N : usize > AnimatablePlayer for [ Tween< T >; N ]
  where T : Animatable + Clone + 'static
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

    fn duration_get( &self ) -> f64
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

    fn delay_get( &self ) -> f64
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
        ( ( self[ 0 ].time() - self.delay_get() ) / self.duration_get() ).clamp( 0.0, 1.0 )
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

  impl< E, const N : usize > Animatable for mingl::Vector< E, N >
  where E : MatEl + Animatable
  {
    fn interpolate(&self, other : &Self, time : f64 ) -> Self
    {
      let v = self.iter().zip( other.iter() )
      .map
      (
        | ( a, b ) |
        a.interpolate( b, time )
      )
      .collect::< Vec< _ > >();

      Self::from_slice( v.as_slice() )
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

  #[ cfg( test ) ]
  mod tests
  {
    use super::*;
    use crate::easing::base::Linear;

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
      assert_eq!( tween.value_get(), 2.5 );

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
      assert_eq!( tween.current_repeat, 1 );

      tween.update( 1.0 ); // Second loop finishes
      assert!( !tween.is_completed() );
      assert_eq!( tween.current_repeat, 2 );

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
      assert_eq!( tween.current_repeat, 1 );

      tween.update( 10.0 );
      assert!( !tween.is_completed() );
      assert_eq!( tween.current_repeat, 11 );
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
      assert_eq!( tween.value_get(), 10.0 );
      assert_eq!( tween.current_repeat, 1 );

      // Second loop: 10.0 -> 0.0 (yoyo)
      let val2 = tween.update( 0.5 );
      assert_eq!( val2, 5.0 );
      tween.update( 0.5 );
      assert_eq!( tween.value_get(), 0.0 );
      assert!( tween.is_completed() );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    AnimationState,
    Tween
  };
}
