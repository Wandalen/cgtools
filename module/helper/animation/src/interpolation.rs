//! Tweening system for smooth entity movement in tile-based games.
//!
//! This module provides comprehensive animation capabilities for creating smooth,
//! visually appealing movement and transformations in tile-based games. It supports
//! various easing functions, animation composition, and frame-based updates.
//!
//! # Animation System
//!
//! The animation system is built around tweening ( interpolation ) between values
//! over time. It supports animating positions, rotations, scales, colors, and
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
//! - Colors ( RGB, RGBA )
//! - Custom interpolatable values
//!

mod private
{
  use crate::sequencer::AnimatableValue;
  use crate::easing::base::{ EasingFunction, Linear, EasingBuilder };
  //use crate::easing::cubic::{ EaseInSine, EaseInQuad, EaseOutQuad, EaseInOutCubic };
  use minwebgl as gl;
  use gl::
  {
    F32x3,
    QuatF32
  };

  /// Trait for types that can be animated ( interpolated ).
  pub trait Animatable : Clone + core::fmt::Debug 
  {
    /// Interpolates between two values at time t ( 0.0 to 1.0 ).
    fn interpolate( &self, other : &Self, t : f32 ) -> Self;
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
    duration : f32,
    /// Current elapsed time
    elapsed : f32,
    /// Easing function to use
    easing : Box< dyn EasingFunction >,
    /// Current animation state
    state : AnimationState,
    /// Delay before animation starts
    delay : f32,
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
      duration : f32, 
      easing : Box< dyn EasingFunction > 
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
        repeat_count : 0,
        current_repeat : 0,
        yoyo : false,
      }
    }

    /// Sets a delay before the animation starts.
    pub fn with_delay( mut self, delay : f32 ) -> Self 
    {
      self.delay = delay.max( 0.0 );
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
    pub fn update( &mut self, delta_time : f32 ) -> T 
    {
      let mut remaining_time = delta_time;

      match self.state 
      {
        AnimationState::Pending => 
        {
          if self.delay > 0.0 
          {
            let delay_consumed = remaining_time.min( self.delay );
            self.delay -= delay_consumed;
            remaining_time -= delay_consumed;
            
            if self.delay <= 0.0 
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

      let normalized_time = ( self.elapsed / self.duration ).clamp( 0.0, 1.0 );
      let eased_time = self.easing.apply( normalized_time );

      // Handle yoyo mode
      let ( start, end, time ) = if self.yoyo && self.current_repeat % 2 == 1 
      {
        ( &self.end_value, &self.start_value, eased_time )
      } 
      else 
      {
        ( &self.start_value, &self.end_value, eased_time )
      };

      start.interpolate( end, time )
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

    /// Gets the progress of the animation ( 0.0 to 1.0 ).
    pub fn progress( &self ) -> f32 
    {
      if self.state == AnimationState::Pending 
      {
        0.0
      } 
      else 
      {
        (self.elapsed / self.duration).clamp( 0.0, 1.0 )
      }
    }
  }

  impl< T > AnimatableValue for Tween< T > 
  where T : Animatable + 'static
  {
    fn update( &mut self, delta_time : f32 ) 
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
  }

  // === ANIMATABLE IMPLEMENTATIONS ===

  impl Animatable for f32 
  {
    fn interpolate( &self, other : &Self, time : f32 ) -> Self 
    {
      self + ( other - self ) * time
    }
  }

  impl Animatable for f64 
  {
    fn interpolate( &self, other : &Self, time : f32 ) -> Self 
    {
      self + ( other - self ) * f64::from( time )
    }
  }

  impl Animatable for i32 
  {
    fn interpolate( &self, other : &Self, time : f32 ) -> Self 
    {
      ( *self as f32 + ( *other as f32 - *self as f32 ) * time ) as i32
    }
  }

  impl Animatable for ( f32, f32 ) 
  {
    fn interpolate( &self, other : &Self, time : f32 ) -> Self 
    {
      (
        self.0.interpolate( &other.0, time ),
        self.1.interpolate( &other.1, time ),
      )
    }
  }

  impl Animatable for ( i32, i32 ) 
  {
    fn interpolate( &self, other : &Self, time : f32 ) -> Self 
    {
      (
        self.0.interpolate( &other.0, time ),
        self.1.interpolate( &other.1, time ),
      )
    }
  }

  impl Animatable for F32x3
  {
    fn interpolate(&self, other : &Self, time : f32 ) -> Self 
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

  impl Animatable for QuatF32
  {
    fn interpolate( &self, other : &Self, time : f32 ) -> Self 
    {
      self.slerp( other, time )
    }
  } 

  /// Special version of [`Transform`] structure that 
  /// used for skeletal animation
  #[ non_exhaustive ]
  #[ derive( Debug, Clone ) ]
  pub struct Transform
  {
    /// Translation used in node transform interpolation if animated 
    pub translation : Option< F32x3 >,
    /// Rotation used in node transform interpolation if animated 
    pub rotation : Option< QuatF32 >,
    /// Scale used in node transform interpolation if animated 
    pub scale : Option< F32x3 >
  }

  impl Animatable for Transform
  {
    fn interpolate(&self, other : &Self, time : f32 ) -> Self 
    {
      let translation = match ( self.translation, other.translation )
      {
        ( None, other ) => other,
        ( this, None ) => this,
        ( Some( this ), Some( other ) ) => Some( this.interpolate( &other, time ) )
      };

      let rotation = match ( self.rotation, other.rotation )
      {
        ( None, other ) => other,
        ( this, None ) => this,
        ( Some( this ), Some( other ) ) => Some( this.interpolate( &other, time ) )
      };

      let scale = match ( self.scale, other.scale )
      {
        ( None, other ) => other,
        ( this, None ) => this,
        ( Some( this ), Some( other ) ) => Some( this.interpolate( &other, time ) )
      };

      Self
      {
        translation,
        rotation,
        scale
      }
    }
  } 

  /// RGB Color for animations.
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub struct Color 
  {
    /// Red component ( 0.0 to 1.0 )
    pub red : f32,
    /// Green component ( 0.0 to 1.0 )
    pub green : f32,
    /// Blue component ( 0.0 to 1.0 )
    pub blue : f32,
    /// Alpha component ( 0.0 to 1.0 )
    pub alpha : f32
  }

  impl Color 
  {
    /// Creates a new color.
    pub fn new( red : f32, green : f32, blue : f32, alpha : f32 ) -> Self 
    {
      Self 
      {
        red : red.clamp( 0.0, 1.0 ),
        green : green.clamp( 0.0, 1.0 ),
        blue : blue.clamp( 0.0, 1.0 ),
        alpha : alpha.clamp( 0.0, 1.0 ),
      }
    }

    /// Creates an RGB color ( alpha = 1.0 ).
    pub fn rgb( red : f32, green : f32, blue : f32 ) -> Self 
    {
      Self::new( red, green, blue, 1.0 )
    }

    /// Creates a white color.
    pub fn white() -> Self 
    {
      Self::rgb( 1.0, 1.0, 1.0 )
    }

    /// Creates a black color.
    pub fn black() -> Self 
    {
      Self::rgb( 0.0, 0.0, 0.0 )
    }

    /// Creates a transparent color.
    pub fn transparent() -> Self 
    {
      Self::new( 0.0, 0.0, 0.0, 0.0 )
    }
  }

  impl Animatable for Color 
  {
    fn interpolate( &self, other : &Self, time : f32 ) -> Self 
    {
      Self 
      {
        red : self.red.interpolate( &other.red, time ),
        green : self.green.interpolate( &other.green, time ),
        blue : self.blue.interpolate( &other.blue, time ),
        alpha : self.alpha.interpolate( &other.alpha, time ),
      }
    }
  }

  // === ANIMATION BUILDER ===

  /// Builder for creating complex animations with chaining.
  #[ derive( Debug ) ]
  pub struct AnimationBuilder< T : Animatable > 
  {
    start_value : T,
  }

  impl< T : Animatable > AnimationBuilder< T > 
  {
    /// Creates a new animation builder starting from a value.
    pub fn from( start_value : T ) -> Self 
    {
      Self { start_value }
    }

    /// Creates a tween to the target value.
    pub fn to( &self, end_value : T, duration : f32 ) -> TweenBuilder< T > 
    where T : Animatable
    {
      TweenBuilder 
      {
        tween : Tween::new( self.start_value.clone(), end_value, duration, Linear::new() ),
      }
    }

    /// Creates a tween with a specific easing function.
    pub fn to_with_easing
    ( 
      &self, 
      end_value : T, 
      duration : f32, 
      easing : Box< dyn EasingFunction > 
    ) 
    -> TweenBuilder< T > 
    where T : Animatable
    {
      TweenBuilder 
      {
        tween : Tween::new( self.start_value.clone(), end_value, duration, easing )
      }
    }
  }

  /// Builder for configuring tween properties.
  #[ derive( Debug ) ]
  pub struct TweenBuilder< T > 
  where T : Animatable
  {
    tween : Tween< T >,
  }

  impl< T > TweenBuilder< T > 
  where T : Animatable
  {
    /// Sets the easing function.
    pub fn easing( mut self, easing : Box< dyn EasingFunction > ) -> Self 
    {
      self.tween.easing = easing;
      self
    }

    /// Sets a delay before the animation starts.
    pub fn delay( mut self, delay : f32 ) -> Self 
    {
      self.tween = self.tween.with_delay( delay );
      self
    }

    /// Sets the repeat count.
    pub fn repeat( mut self, count : i32 ) -> Self 
    {
      self.tween = self.tween.with_repeat( count );
      self
    }

    /// Enables yoyo mode.
    pub fn yoyo( mut self, yoyo: bool ) -> Self 
    {
      self.tween = self.tween.with_yoyo( yoyo );
      self
    }

    /// Builds the final tween.
    pub fn build( self ) -> Tween< T > 
    {
      self.tween
    }
  }

  // === CONVENIENCE FUNCTIONS ===

  /// Creates an animation builder from a starting value.
  pub fn animate< T : Animatable >( start_value : T ) -> AnimationBuilder< T > 
  {
    AnimationBuilder::from( start_value )
  }

  /// Creates a simple linear tween.
  pub fn tween< T >( start : T, end : T, duration : f32 ) -> Tween< T >
  where T : Animatable
  {
    Tween::new( start, end, duration, Linear::new() )
  }

  /// Creates a tween with easing.
  pub fn tween_with_easing< T : Animatable >
  (
    start : T, 
    end : T, 
    duration : f32, 
    easing : Box< dyn EasingFunction >
  ) 
  -> Tween< T > 
  {
    Tween::new( start, end, duration, easing )
  }

  #[ cfg( test ) ]
  mod tests 
  {
    use super::*;
    use crate::easing::base::Linear;
    use crate::easing::cubic::EaseInOutCubic;

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

    #[ test ]
    fn test_color_interpolation()
    {
      let red = Color::rgb( 1.0, 0.0, 0.0 );
      let blue = Color::rgb( 0.0, 0.0, 1.0 );
      let purple = red.interpolate( &blue, 0.5 );
      assert_eq!( purple.r, 0.5 );
      assert_eq!( purple.g, 0.0 );
      assert_eq!( purple.b, 0.5 );
      assert_eq!( purple.a, 1.0 );
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
      assert_eq!( tween.get_current_value(), 10.0 );
      assert_eq!( tween.current_repeat, 1 );

      // Second loop: 10.0 -> 0.0 (yoyo)
      let val2 = tween.update( 0.5 );
      assert_eq!( val2, 5.0 );
      tween.update( 0.5 );
      assert_eq!( tween.get_current_value(), 0.0 );
      assert!( tween.is_completed() );
    }

    // --- TweenBuilder and Convenience Function Tests ---

    #[ test ]
    fn test_animation_builder_full_chain()
    {
      let tween = animate( 0.0_f32 )
      .to_with_easing( 100.0, 2.0, EaseInOutCubic::new() )
      .delay( 0.5 )
      .repeat( 3 )
      .yoyo( true )
      .build();
      
      assert_eq!( tween.start_value, 0.0 );
      assert_eq!( tween.end_value, 100.0 );
      assert_eq!( tween.duration, 2.0 );
      assert_eq!( tween.delay, 0.5 );
      assert_eq!( tween.repeat_count, 3 );
      assert_eq!( tween.yoyo, true );
    }

    #[ test ]
    fn test_simple_tween_helper()
    {
      let simple_tween = tween( 5.0_f32, 15.0_f32, 2.0 );
      assert_eq!( simple_tween.start_value, 5.0 );
      assert_eq!( simple_tween.end_value, 15.0 );
      assert_eq!( simple_tween.duration, 2.0 );
      
      let mut tween = simple_tween;
      let val = tween.update( 1.0 );
      assert_eq!( val, 10.0 );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    AnimationState,
    Tween,
    Animatable,
    Transform
  };
}