//! Animation and tweening system for smooth entity movement in tile-based games.
//!
//! This module provides comprehensive animation capabilities for creating smooth,
//! visually appealing movement and transformations in tile-based games. It supports
//! various easing functions, animation composition, and frame-based updates.
//!
//! # Animation System
//!
//! The animation system is built around tweening (interpolation) between values
//! over time. It supports animating positions, rotations, scales, colors, and
//! custom properties with different easing functions.
//!
//! ## Core Concepts
//!
//! - **Tween**: Interpolates between start and end values over duration
//! - **Easing**: Mathematical functions that control animation timing
//! - **Animation**: Collection of tweens that can run sequentially or in parallel
//! - **Timeline**: Manages multiple animations with precise timing control
//!
//! ## Supported Value Types
//!
//! - Position coordinates (any coordinate system)
//! - Floating point values (scale, rotation, opacity)
//! - Colors (RGB, RGBA)
//! - Custom interpolatable values
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::animation::*;
//! use tiles_tools::coordinates::square::{Coordinate, FourConnected};
//!
//! // Create a position animation
//! let start = Coordinate::<FourConnected>::new(0, 0);
//! let end = Coordinate::<FourConnected>::new(10, 5);
//! let tween = Tween::new(start, end, 2.0, EasingFunction::EaseInOutCubic);
//!
//! // Create an animation timeline
//! let mut timeline = Timeline::new();
//! timeline.add_tween("move", tween);
//!
//! // Update animation over time
//! timeline.update(0.5); // 0.5 seconds elapsed
//! let current_pos = timeline.get_value::<Coordinate<FourConnected>>("move");
//! ```

mod private
{
  use crate::sequencer::AnimatableValue;
  use std::collections::HashMap;

  /// Represents different easing functions for smooth animations.
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub enum EasingFunction
  {
    /// Linear interpolation (constant speed)
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in-out (slow start and end)
    EaseInOut,
    /// Quadratic ease in
    EaseInQuad,
    /// Quadratic ease out
    EaseOutQuad,
    /// Quadratic ease in-out
    EaseInOutQuad,
    /// Cubic ease in
    EaseInCubic,
    /// Cubic ease out
    EaseOutCubic,
    /// Cubic ease in-out
    EaseInOutCubic,
    /// Bounce ease out
    BounceOut,
    /// Elastic ease out
    ElasticOut,
    /// Back ease in (overshoot)
    BackIn,
    /// Back ease out (overshoot)
    BackOut,
  }

  impl EasingFunction
  {
    /// Applies the easing function to a normalized time value (0.0 to 1.0).
    pub fn apply( &self, t : f32 ) -> f32
    {
      let t = t.clamp(0.0, 1.0);
      
      match self
      {
        EasingFunction::Linear => t,
        
        EasingFunction::EaseIn => t * t,
        EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        EasingFunction::EaseInOut => {
          if t < 0.5 {
            2.0 * t * t
          } else {
            -1.0 + (4.0 - 2.0 * t) * t
          }
        }
        
        EasingFunction::EaseInQuad => t * t,
        EasingFunction::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
        EasingFunction::EaseInOutQuad => {
          if t < 0.5 {
            2.0 * t * t
          } else {
            -1.0 + (4.0 - 2.0 * t) * t
          }
        }
        
        EasingFunction::EaseInCubic => t * t * t,
        EasingFunction::EaseOutCubic =>
        {
          let t1 = t - 1.0;
          1.0 + t1 * t1 * t1
        }
        EasingFunction::EaseInOutCubic =>
        {
          if t < 0.5
          {
            4.0 * t * t * t
          }
          else
          {
            let t1 = 2.0 * t - 2.0;
            1.0 + t1 * t1 * t1 / 2.0
          }
        }
        
        EasingFunction::BounceOut =>
        {
          if t < 1.0 / 2.75 {
            7.5625 * t * t
          } else if t < 2.0 / 2.75 {
            let t1 = t - 1.5 / 2.75;
            7.5625 * t1 * t1 + 0.75
          } else if t < 2.5 / 2.75 {
            let t1 = t - 2.25 / 2.75;
            7.5625 * t1 * t1 + 0.9375
          } else {
            let t1 = t - 2.625 / 2.75;
            7.5625 * t1 * t1 + 0.984375
          }
        }
        
        EasingFunction::ElasticOut => {
          if t == 0.0 || t == 1.0 {
            t
          } else {
            let p = 0.3;
            let s = p / 4.0;
            2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin() + 1.0
          }
        }
        
        EasingFunction::BackIn => {
          let c1 = 1.70158;
          let c3 = c1 + 1.0;
          c3 * t * t * t - c1 * t * t
        }
        
        EasingFunction::BackOut => {
          let c1 = 1.70158;
          let c3 = c1 + 1.0;
          let t1 = t - 1.0;
          1.0 + c3 * t1 * t1 * t1 + c1 * t1 * t1
        }
      }
    }
  }

  /// Trait for types that can be animated (interpolated).
  pub trait Animatable: Clone + std::fmt::Debug {
    /// Interpolates between two values at time t (0.0 to 1.0).
    fn interpolate(&self, other: &Self, t: f32) -> Self;
  }

  /// Animation state for tracking tween progress.
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub enum AnimationState {
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
  #[derive(Debug, Clone)]
  pub struct Tween<T> {
    /// Starting value
    start_value: T,
    /// Target value
    end_value: T,
    /// Animation duration in seconds
    duration: f32,
    /// Current elapsed time
    elapsed: f32,
    /// Easing function to use
    easing: EasingFunction,
    /// Current animation state
    state: AnimationState,
    /// Delay before animation starts
    delay: f32,
    /// Number of times to repeat (0 = no repeat, -1 = infinite)
    repeat_count: i32,
    /// Current repeat iteration
    current_repeat: i32,
    /// Whether to reverse on repeat (ping-pong)
    yoyo: bool,
  }

  impl<T: Animatable> Tween<T> {
    /// Creates a new tween animation.
    pub fn new(start: T, end: T, duration: f32, easing: EasingFunction) -> Self {
      Self {
        start_value: start,
        end_value: end,
        duration: duration.max(0.001), // Minimum duration to avoid division by zero
        elapsed: 0.0,
        easing,
        state: AnimationState::Pending,
        delay: 0.0,
        repeat_count: 0,
        current_repeat: 0,
        yoyo: false,
      }
    }

    /// Sets a delay before the animation starts.
    pub fn with_delay(mut self, delay: f32) -> Self {
      self.delay = delay.max(0.0);
      self
    }

    /// Sets the number of times to repeat the animation.
    pub fn with_repeat(mut self, count: i32) -> Self {
      self.repeat_count = count;
      self
    }

    /// Enables yoyo mode (reverse direction on repeat).
    pub fn with_yoyo(mut self, yoyo: bool) -> Self {
      self.yoyo = yoyo;
      self
    }

    /// Updates the tween with the elapsed time and returns current value.
    pub fn update(&mut self, delta_time: f32) -> T {
      let mut remaining_time = delta_time;

      match self.state {
        AnimationState::Pending => {
          if self.delay > 0.0 {
            let delay_consumed = remaining_time.min(self.delay);
            self.delay -= delay_consumed;
            remaining_time -= delay_consumed;
            
            if self.delay <= 0.0 {
              self.state = AnimationState::Running;
            } else {
              return self.start_value.clone();
            }
          } else {
            self.state = AnimationState::Running;
          }
        }
        AnimationState::Paused | AnimationState::Completed => {
          return self.get_current_value();
        }
        AnimationState::Running => {}
      }

      // Apply remaining time to animation
      if remaining_time > 0.0 && self.state == AnimationState::Running {
        self.elapsed += remaining_time;

        if self.elapsed >= self.duration {
          // Animation completed this frame
          if self.repeat_count != 0 {
            self.handle_repeat();
          } else {
            self.state = AnimationState::Completed;
            self.elapsed = self.duration;
          }
        }
      }

      self.get_current_value()
    }

    /// Gets the current interpolated value without updating time.
    pub fn get_current_value(&self) -> T {
      if self.state == AnimationState::Pending {
        return self.start_value.clone();
      }

      let normalized_time = (self.elapsed / self.duration).clamp(0.0, 1.0);
      let eased_time = self.easing.apply(normalized_time);

      // Handle yoyo mode
      let (start, end, t) = if self.yoyo && self.current_repeat % 2 == 1 {
        (&self.end_value, &self.start_value, eased_time)
      } else {
        (&self.start_value, &self.end_value, eased_time)
      };

      start.interpolate(end, t)
    }

    /// Handles animation repeat logic.
    fn handle_repeat(&mut self) {
      if self.repeat_count > 0 {
        self.current_repeat += 1;
        if self.current_repeat > self.repeat_count {
          self.state = AnimationState::Completed;
          return;
        }
      } else if self.repeat_count == -1 {
        // Infinite repeat
        self.current_repeat += 1;
      }

      self.elapsed = 0.0;
      self.state = AnimationState::Running;
    }

    /// Pauses the animation.
    pub fn pause(&mut self) {
      if self.state == AnimationState::Running {
        self.state = AnimationState::Paused;
      }
    }

    /// Resumes a paused animation.
    pub fn resume(&mut self) {
      if self.state == AnimationState::Paused {
        self.state = AnimationState::Running;
      }
    }

    /// Resets the animation to its starting state.
    pub fn reset(&mut self) {
      self.elapsed = 0.0;
      self.current_repeat = 0;
      self.state = if self.delay > 0.0 {
        AnimationState::Pending
      } else {
        AnimationState::Running
      };
    }

    /// Checks if the animation is completed.
    pub fn is_completed(&self) -> bool {
      self.state == AnimationState::Completed
    }

    /// Gets the current animation state.
    pub fn state(&self) -> AnimationState {
      self.state
    }

    /// Gets the progress of the animation (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
      if self.state == AnimationState::Pending {
        0.0
      } else {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
      }
    }
  }

  impl<T: Animatable + 'static> AnimatableValue for Tween<T> {
    fn update(&mut self, delta_time: f32) {
      self.update(delta_time);
    }

    fn is_completed(&self) -> bool {
      self.is_completed()
    }

    fn pause(&mut self) {
      self.pause();
    }

    fn resume(&mut self) {
      self.resume();
    }

    fn reset(&mut self) {
      self.reset();
    }

    fn as_any(&self) -> &dyn std::any::Any {
      self
    }
  }

  // === ANIMATABLE IMPLEMENTATIONS ===

  impl Animatable for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
      self + (other - self) * t
    }
  }

  impl Animatable for f64 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
      self + (other - self) * (t as f64)
    }
  }

  impl Animatable for i32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
      (*self as f32 + (*other as f32 - *self as f32) * t) as i32
    }
  }

  impl Animatable for (f32, f32) {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
      (
        self.0.interpolate(&other.0, t),
        self.1.interpolate(&other.1, t),
      )
    }
  }

  impl Animatable for (i32, i32) {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
      (
        self.0.interpolate(&other.0, t),
        self.1.interpolate(&other.1, t),
      )
    }
  }

  /// RGB Color for animations.
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub struct Color {
    /// Red component (0.0 to 1.0)
    pub r: f32,
    /// Green component (0.0 to 1.0)
    pub g: f32,
    /// Blue component (0.0 to 1.0)
    pub b: f32,
    /// Alpha component (0.0 to 1.0)
    pub a: f32,
  }

  impl Color {
    /// Creates a new color.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
      Self {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: a.clamp(0.0, 1.0),
      }
    }

    /// Creates an RGB color (alpha = 1.0).
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
      Self::new(r, g, b, 1.0)
    }

    /// Creates a white color.
    pub fn white() -> Self {
      Self::rgb(1.0, 1.0, 1.0)
    }

    /// Creates a black color.
    pub fn black() -> Self {
      Self::rgb(0.0, 0.0, 0.0)
    }

    /// Creates a transparent color.
    pub fn transparent() -> Self {
      Self::new(0.0, 0.0, 0.0, 0.0)
    }
  }

  impl Animatable for Color {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
      Self {
        r: self.r.interpolate(&other.r, t),
        g: self.g.interpolate(&other.g, t),
        b: self.b.interpolate(&other.b, t),
        a: self.a.interpolate(&other.a, t),
      }
    }
  }

  impl Animatable for crate::coordinates::pixel::Pixel {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
      let x = self.x().interpolate(&other.x(), t);
      let y = self.y().interpolate(&other.y(), t);
      Self::new(x, y)
    }
  }

  // === ANIMATION BUILDER ===

  /// Builder for creating complex animations with chaining.
  #[derive(Debug)]
  pub struct AnimationBuilder<T: Animatable> {
    start_value: T,
  }

  impl<T: Animatable> AnimationBuilder<T> {
    /// Creates a new animation builder starting from a value.
    pub fn from(start_value: T) -> Self {
      Self { start_value }
    }

    /// Creates a tween to the target value.
    pub fn to(&self, end_value: T, duration: f32) -> TweenBuilder<T> {
      TweenBuilder {
        tween: Tween::new(self.start_value.clone(), end_value, duration, EasingFunction::Linear),
      }
    }

    /// Creates a tween with a specific easing function.
    pub fn to_with_easing(&self, end_value: T, duration: f32, easing: EasingFunction) -> TweenBuilder<T> {
      TweenBuilder {
        tween: Tween::new(self.start_value.clone(), end_value, duration, easing),
      }
    }
  }

  /// Builder for configuring tween properties.
  #[derive(Debug)]
  pub struct TweenBuilder<T: Animatable> {
    tween: Tween<T>,
  }

  impl<T: Animatable> TweenBuilder<T> {
    /// Sets the easing function.
    pub fn easing(mut self, easing: EasingFunction) -> Self {
      self.tween.easing = easing;
      self
    }

    /// Sets a delay before the animation starts.
    pub fn delay(mut self, delay: f32) -> Self {
      self.tween = self.tween.with_delay(delay);
      self
    }

    /// Sets the repeat count.
    pub fn repeat(mut self, count: i32) -> Self {
      self.tween = self.tween.with_repeat(count);
      self
    }

    /// Enables yoyo mode.
    pub fn yoyo(mut self, yoyo: bool) -> Self {
      self.tween = self.tween.with_yoyo(yoyo);
      self
    }

    /// Builds the final tween.
    pub fn build(self) -> Tween<T> {
      self.tween
    }
  }

  // === CONVENIENCE FUNCTIONS ===

  /// Creates an animation builder from a starting value.
  pub fn animate<T: Animatable>(start_value: T) -> AnimationBuilder<T> {
    AnimationBuilder::from(start_value)
  }

  /// Creates a simple linear tween.
  pub fn tween<T: Animatable>(start: T, end: T, duration: f32) -> Tween<T> {
    Tween::new(start, end, duration, EasingFunction::Linear)
  }

  /// Creates a tween with easing.
  pub fn tween_with_easing<T: Animatable>(
    start: T, 
    end: T, 
    duration: f32, 
    easing: EasingFunction
  ) -> Tween<T> {
    Tween::new(start, end, duration, easing)
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    use crate::coordinates::square::{Coordinate as SquareCoord, FourConnected};

    #[test]
    fn test_easing_functions() {
      assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
      assert_eq!(EasingFunction::EaseIn.apply(0.0), 0.0);
      assert_eq!(EasingFunction::EaseIn.apply(1.0), 1.0);
      
      // EaseInQuad should be slower than linear at the start
      assert!(EasingFunction::EaseInQuad.apply(0.2) < EasingFunction::Linear.apply(0.2));
      
      // EaseOutQuad should be faster than linear at the start
      assert!(EasingFunction::EaseOutQuad.apply(0.2) > EasingFunction::Linear.apply(0.2));
    }

    #[test]
    fn test_f32_interpolation() {
      let start = 10.0_f32;
      let end = 20.0_f32;
      
      assert_eq!(start.interpolate(&end, 0.0), 10.0);
      assert_eq!(start.interpolate(&end, 1.0), 20.0);
      assert_eq!(start.interpolate(&end, 0.5), 15.0);
    }

    #[test]
    fn test_color_interpolation() {
      let red = Color::rgb(1.0, 0.0, 0.0);
      let blue = Color::rgb(0.0, 0.0, 1.0);
      
      let purple = red.interpolate(&blue, 0.5);
      assert_eq!(purple.r, 0.5);
      assert_eq!(purple.g, 0.0);
      assert_eq!(purple.b, 0.5);
      assert_eq!(purple.a, 1.0);
    }

    #[test]
    fn test_coordinate_interpolation() {
      let start = SquareCoord::<FourConnected>::new(0, 0);
      let end = SquareCoord::<FourConnected>::new(10, 20);
      
      let mid = start.interpolate(&end, 0.5);
      assert_eq!(mid.x, 5);
      assert_eq!(mid.y, 10);
    }

    #[test]
    fn test_tween_basic_animation() {
      let mut tween = Tween::new(0.0_f32, 10.0_f32, 1.0, EasingFunction::Linear);
      
      assert_eq!(tween.state(), AnimationState::Pending);
      
      let value1 = tween.update(0.5);
      assert_eq!(tween.state(), AnimationState::Running);
      assert_eq!(value1, 5.0);
      
      let value2 = tween.update(0.5);
      assert_eq!(tween.state(), AnimationState::Completed);
      assert_eq!(value2, 10.0);
      
      assert!(tween.is_completed());
    }

    #[test]
    fn test_tween_with_delay() {
      let mut tween = Tween::new(0.0_f32, 10.0_f32, 1.0, EasingFunction::Linear)
        .with_delay(0.5);
      
      // During delay, should return start value
      let value1 = tween.update(0.3);
      assert_eq!(value1, 0.0);
      assert_eq!(tween.state(), AnimationState::Pending);
      
      // After delay, should start animating
      let value2 = tween.update(0.3);
      assert_eq!(tween.state(), AnimationState::Running);
      assert!((value2 - 1.0).abs() < 0.001); // 0.1s into 1s animation = 10% ≈ 1.0
    }

    #[test]
    fn test_tween_repeat() {
      let mut tween = Tween::new(0.0_f32, 10.0_f32, 0.5, EasingFunction::Linear)
        .with_repeat(2);
      
      // First iteration
      let _value1 = tween.update(0.5);
      assert!(!tween.is_completed());
      
      // Second iteration
      let _value2 = tween.update(0.5);
      assert!(!tween.is_completed());
      
      // Third iteration (final repeat)
      let _value3 = tween.update(0.5);
      assert!(tween.is_completed());
    }

    #[test]
    fn test_tween_yoyo() {
      let mut tween = Tween::new(0.0_f32, 10.0_f32, 1.0, EasingFunction::Linear)
        .with_repeat(1)
        .with_yoyo(true);
      
      // First iteration: 0 -> 10
      let _value1 = tween.update(1.0);
      assert!(!tween.is_completed());
      
      // Second iteration: 10 -> 0 (yoyo)
      let value2 = tween.update(0.5);
      assert_eq!(value2, 5.0); // Halfway back from 10 to 0
      
      tween.update(0.5);
      assert!(tween.is_completed());
    }

    #[test]
    fn test_tween_pause_resume() {
      let mut tween = Tween::new(0.0_f32, 10.0_f32, 1.0, EasingFunction::Linear);
      
      tween.update(0.5);
      assert_eq!(tween.state(), AnimationState::Running);
      
      tween.pause();
      assert_eq!(tween.state(), AnimationState::Paused);
      
      // Updating while paused shouldn't change the value
      let paused_value = tween.update(0.5);
      assert_eq!(paused_value, 5.0);
      assert_eq!(tween.state(), AnimationState::Paused);
      
      tween.resume();
      assert_eq!(tween.state(), AnimationState::Running);
      
      tween.update(0.5);
      assert!(tween.is_completed());
    }

    #[test]
    fn test_timeline_basic() {
      let mut timeline = Timeline::new();
      
      let position_tween = tween(
        SquareCoord::<FourConnected>::new(0, 0),
        SquareCoord::<FourConnected>::new(10, 10),
        1.0
      );
      
      let scale_tween = tween(1.0_f32, 2.0_f32, 1.0);
      
      timeline.add_tween("position", position_tween);
      timeline.add_tween("scale", scale_tween);
      
      assert_eq!(timeline.animation_count(), 2);
      assert!(!timeline.is_completed());
      
      timeline.update(0.5);
      
      let pos = timeline.get_value::<SquareCoord<FourConnected>>("position").unwrap();
      assert_eq!(pos.x, 5);
      assert_eq!(pos.y, 5);
      
      let scale = timeline.get_value::<f32>("scale").unwrap();
      assert_eq!(scale, 1.5);
      
      timeline.update(0.5);
      assert!(timeline.is_completed());
    }

    #[test]
    fn test_animation_builder() {
      let tween = animate(0.0_f32)
        .to(10.0, 1.0)
        .easing(EasingFunction::EaseInOutCubic)
        .delay(0.1)
        .repeat(2)
        .yoyo(true)
        .build();
      
      assert_eq!(tween.start_value, 0.0);
      assert_eq!(tween.end_value, 10.0);
      assert_eq!(tween.duration, 1.0);
      assert_eq!(tween.easing, EasingFunction::EaseInOutCubic);
      assert_eq!(tween.delay, 0.1);
      assert_eq!(tween.repeat_count, 2);
      assert!(tween.yoyo);
    }

    #[test]
    fn test_convenience_functions() {
      let simple_tween = tween(5.0_f32, 15.0_f32, 2.0);
      assert_eq!(simple_tween.start_value, 5.0);
      assert_eq!(simple_tween.end_value, 15.0);
      assert_eq!(simple_tween.duration, 2.0);
      assert_eq!(simple_tween.easing, EasingFunction::Linear);
      
      let easing_tween = tween_with_easing(
        0.0_f32, 
        100.0_f32, 
        1.5, 
        EasingFunction::BounceOut
      );
      assert_eq!(easing_tween.easing, EasingFunction::BounceOut);
    }

    #[test]
    fn test_color_animations() {
      let mut color_tween = tween(
        Color::black(),
        Color::white(),
        1.0
      );
      
      let mid_color = color_tween.update(0.5);
      assert_eq!(mid_color.r, 0.5);
      assert_eq!(mid_color.g, 0.5);
      assert_eq!(mid_color.b, 0.5);
      assert_eq!(mid_color.a, 1.0);
    }

    #[test]
    fn test_complex_easing() {
      // Test bounce easing has the expected characteristics
      let bounce_start = EasingFunction::BounceOut.apply(0.0);
      let bounce_end = EasingFunction::BounceOut.apply(1.0);
      let bounce_mid = EasingFunction::BounceOut.apply(0.5);
      
      assert_eq!(bounce_start, 0.0);
      assert_eq!(bounce_end, 1.0);
      assert!(bounce_mid > 0.0 && bounce_mid < 1.0);
      
      // Test elastic easing
      let elastic_start = EasingFunction::ElasticOut.apply(0.0);
      let elastic_end = EasingFunction::ElasticOut.apply(1.0);
      
      assert_eq!(elastic_start, 0.0);
      assert_eq!(elastic_end, 1.0);
    }

    #[test]
    fn test_timeline_pause_resume() {
      let mut timeline = Timeline::new();
      timeline.add_tween("test", tween(0.0_f32, 10.0_f32, 1.0));
      
      timeline.update(0.5);
      timeline.pause();
      
      // Should not update while paused
      timeline.update(0.5);
      let value = timeline.get_value::<f32>("test").unwrap();
      assert_eq!(value, 5.0);
      
      timeline.resume();
      timeline.update(0.5);
      assert!(timeline.is_completed());
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    
  };
}