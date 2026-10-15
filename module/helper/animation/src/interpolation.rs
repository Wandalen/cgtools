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
  use crate::easing::base::EasingFunction;
  use minwebgl as gl;
  use gl::
  {
    NdFloat,
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
        duration : self.duration,
        elapsed : self.elapsed,
        easing : clone_dyn_types::clone_into_box( &*self.easing ),
        state : self.state,
        delay : self.delay,
        remain : self.remain,
        repeat_count : self.repeat_count,
        current_repeat : self.current_repeat,
        yoyo : self.yoyo
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
    #[ must_use ]
    pub fn with_delay( mut self, delay : f64 ) -> Self
    {
      self.delay = delay.max( 0.0 );
      self.remain = self.delay;
      self
    }

    /// Sets an animation duration
    #[ must_use ]
    pub fn with_duration( mut self, duration : f64 ) -> Self
    {
      // Fix(BUG-142)
      // Root cause: clamped to `0.0` instead of the same `0.001` floor `new` uses ("Minimum
      // duration to avoid division by zero", above) -- `with_duration(0.0)` reintroduced exactly
      // the `self.elapsed / self.duration` == `0.0 / 0.0` == NaN case `new`'s own clamp exists to
      // prevent, propagating NaN out of `value_get`/`progress` on the very first `update`.
      // Pitfall: a builder method re-deriving a sibling constructor's own documented invariant
      // ("avoid division by zero") must copy the sibling's actual clamp value, not just its
      // clamp's polarity (`.max`) -- `0.0` still satisfies "non-negative" while reintroducing the
      // exact division-by-zero the invariant was written to prevent.
      self.duration = duration.max( 0.001 );
      self
    }

    /// Sets the number of times to repeat the animation.
    #[ must_use ]
    pub fn with_repeat( mut self, count : i32 ) -> Self
    {
      self.repeat_count = count;
      self
    }

    /// Enables yoyo mode ( reverse direction on repeat ).
    #[ must_use ]
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
    // Fix(TASK-015): the post-wrap elapsed time was clamped with .min(0.0), but
    // `elapsed - duration * floor(elapsed/duration)` is the floor-division remainder, which is
    // mathematically always >= 0.0 — so .min(0.0) forced elapsed back to exactly 0.0 on every
    // repeat instead of preserving the real leftover time, dropping the fractional progress made
    // into the new loop.
    // Root cause: `.min(0.0)` written where `.max(0.0)` was intended (guarding against
    // floating-point drift producing a tiny negative remainder), inverting the clamp direction.
    // Pitfall: existing tests only drive `update()` with deltas that are exact multiples of
    // `duration`, where the remainder is exactly 0.0 either way — the bug is invisible unless the
    // elapsed time crosses a repeat boundary mid-frame.
    fn repeat_handle( &mut self )
    {
      let elapsed_repeats = ( self.elapsed / self.duration ).floor();
      if self.repeat_count == -1
      {
        // Infinite repeat
        // `elapsed_repeats` counts whole durations crossed within one frame's delta time —
        // bounded in practice by plausible delta_time magnitudes; reaching i32::MAX would need
        // thousands of repeats to elapse within a single `update()` call.
        let repeats : i32 = elapsed_repeats as i32;
        self.current_repeat += repeats;
        self.elapsed = ( self.elapsed - ( self.duration * elapsed_repeats ) ).max( 0.0 );
        self.state = AnimationState::Running;
      }
      else if self.repeat_count > 0 && self.current_repeat < self.repeat_count
      {
        // Finite repeat
        // See the infinite-repeat branch above for why this narrowing is bounded in practice.
        let repeats : i32 = elapsed_repeats as i32;
        // Fix(BUG-232)
        // Root cause: a single `update()` call whose `delta_time` spans more than one repeat
        // boundary (a frame stall, a backgrounded tab, a deliberate fast-forward) crossed
        // `elapsed_repeats` boundaries in one shot; the old code added all of them to
        // `current_repeat` unconditionally, letting it overshoot past `repeat_count` while still
        // leaving `state` at `Running` -- exactly the crossing that should have completed the
        // Tween instead ran one (or more) extra, unrequested loops.
        // Pitfall: processing N boundary crossings in one call must behave identically to
        // processing them one at a time -- the moment a crossing would occur at
        // `current_repeat == repeat_count`, that crossing completes the Tween immediately and
        // discards every further crossing in the same batch, rather than letting the batch
        // silently carry `current_repeat` past `repeat_count`.
        let remaining = self.repeat_count - self.current_repeat;
        if repeats > remaining
        {
          self.current_repeat = self.repeat_count;
          self.state = AnimationState::Completed;
          self.elapsed = self.duration;
        }
        else
        {
          self.current_repeat += repeats;
          self.elapsed = ( self.elapsed - ( self.duration * elapsed_repeats ) ).max( 0.0 );
          self.state = AnimationState::Running;
        }
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

    // Fix(BUG-352)
    // Root cause: gated only on `state == Running`, so calling `pause()` while a Tween was still
    // mid-delay ( `state == Pending`, `with_delay(...)`'s countdown not yet finished ) was a
    // silent no-op -- `update`'s own match never early-returns for `Pending` ( only for `Paused`/
    // `Completed`, see above ), so a later `update` kept ticking the delay ( and, once it
    // expired, the animation itself ) forward exactly as if `pause()` had never been called.
    // Pitfall: `Completed` is deliberately still excluded here -- pausing an already-finished
    // Tween must not make `is_completed()` ( `state == Completed` ) start reporting `false`.
    fn pause( &mut self )
    {
      if matches!( self.state, AnimationState::Running | AnimationState::Pending )
      {
        self.state = AnimationState::Paused;
      }
    }

    // Fix(BUG-352)
    // Root cause: widening `pause()` ( above ) to also freeze a mid-delay `Pending` Tween makes
    // `Paused` reachable while `self.remain` ( the countdown `with_delay` set up, consulted by
    // `update`'s own `Pending` arm above ) is still > 0.0 -- unconditionally resuming straight to
    // `Running` skipped that leftover delay entirely, since `update`'s `Running` branch ticks
    // `self.elapsed` ( the animated value itself ) forward immediately and never re-checks
    // `remain`.
    // Pitfall: `self.remain` is already exactly the countdown `update`'s own `Pending` arm
    // consults -- reuse it here rather than assuming every pause happened only after the delay
    // had fully elapsed.
    fn resume( &mut self )
    {
      if self.state == AnimationState::Paused
      {
        self.state = if self.remain > 0.0 { AnimationState::Pending } else { AnimationState::Running };
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
        // Fix(BUG-140)
        // Root cause: subtracted `self.delay` from `self.elapsed`, but `update` only ever adds
        // to `elapsed` AFTER the delay countdown (`remain`) has been fully consumed -- `elapsed`
        // is already delay-exclusive by construction (mirrors `value_get`'s own
        // `self.elapsed / self.duration`, which performs no such subtraction). Subtracting
        // `delay` a second time undercounted progress, and a fully-completed delayed tween
        // (`elapsed == duration`) never reported `1.0`.
        // Pitfall: identical-looking `( time - delay_get() ) / duration_get()` formulas exist
        // elsewhere (e.g. `Sequencer::progress()`) where `time`/`elapsed` DO include the delay by
        // construction -- the correct formula depends on which "elapsed" convention the specific
        // type actually uses, not on the formula's shape alone.
        ( self.elapsed / self.duration ).clamp( 0.0, 1.0 )
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
      self.iter().all( Tween::is_completed )
    }

    fn pause( &mut self )
    {
      for tween in self.iter_mut() { tween.pause(); }
    }

    fn resume( &mut self )
    {
      for tween in self.iter_mut() { tween.resume(); }
    }

    fn reset( &mut self )
    {
      for tween in self.iter_mut() { tween.reset(); }
    }

    // Fix(TASK-015): duration_get computed min_start via .max() (seeded 0.0), returning the
    // latest delay instead of the earliest, and delay_get seeded its .min() reduction at 0.0
    // instead of f64::MAX, so it always returned 0.0 whenever every real delay was positive.
    // Root cause: min-reduction pattern copy-pasted from a max-reduction without adjusting the
    // seed value or comparison direction.
    // Pitfall: a min-reduction seeded at a real domain value like 0.0 silently returns that seed
    // whenever every element is >= it, so arrays containing a zero-delay tween mask the bug —
    // it only surfaces once every element is strictly positive.
    //
    // Fix(BUG-501)
    // Root cause: for `N == 0`, both reduction loops above never execute, so `min_start`
    // stays at its `f64::MAX` seed and `max_end` stays at its `0.0` seed -- `duration_get`
    // then returns `0.0 - f64::MAX == -f64::MAX`, a nonsensical negative-infinity-scale
    // duration for an empty tween group.
    // Pitfall: a min/max-reduction seeded for the non-empty case has no valid seed relationship
    // for the empty case -- `max_end - min_start` assumes `min_start <= max_end`, which the
    // unreached-loop seeds (`f64::MAX`, `0.0`) violate in the opposite direction from what an
    // "empty" answer should even look like (a huge negative number, not zero).
    fn duration_get( &self ) -> f64
    {
      if self.is_empty()
      {
        return 0.0;
      }

      let mut min_start = f64::MAX;
      for tween in self
      {
        min_start = tween.delay.min( min_start );
      }

      let mut max_end = 0.0;
      for tween in self
      {
        max_end = ( tween.delay + tween.duration ).max( max_end );
      }

      max_end - min_start
    }

    // Fix(BUG-501)
    // Root cause: for `N == 0`, the reduction loop never executes, so `min_delay` stays at
    // its `f64::MAX` seed and is returned as-is -- a group with zero tweens reports a delay
    // of `f64::MAX` instead of the "nothing to delay" answer of `0.0`.
    // Pitfall: same class of defect as `duration_get` above -- a reduction seed chosen to be
    // "beaten" by any real element is never beaten when there are no elements, and gets
    // returned unchanged as if it were a legitimate result.
    fn delay_get( &self ) -> f64
    {
      if self.is_empty()
      {
        return 0.0;
      }

      let mut min_delay = f64::MAX;
      for tween in self
      {
        min_delay = tween.delay.min( min_delay );
      }

      min_delay
    }

    // Fix(BUG-143)
    // Root cause: reconstructed "elapsed since the group's own start" from `self[ 0 ]` alone,
    // via `self[ 0 ].time() - self.delay_get()` -- two defects in one formula. (1) it omitted
    // `self[ 0 ].delay` entirely, so whenever element 0's own delay differs from the group's
    // earliest delay (`delay_get()`), the result is wrong from the very first tick, not just
    // near completion. (2) `self[ 0 ]` is an arbitrary, possibly non-representative element --
    // once IT individually completes, its own `time()` (an already delay-exclusive elapsed,
    // frozen at its own `duration` on completion per `Tween::update`'s early-return for
    // `Completed`) stops advancing even while OTHER, longer-running array members keep
    // animating toward the group's real completion (`duration_get()`, correctly the max
    // `delay + duration` across every element). A fully-completed array (`is_completed()` ==
    // true for every element) could therefore report `progress() < 1.0` forever, violating the
    // trait's own "0.0 to 1.0" contract at exactly the boundary condition `Tween::progress()`
    // itself was required to hit precisely (BUG-140).
    // Pitfall: `duration_get()`/`delay_get()` already correctly aggregate over every element
    // (min delay, max end) -- `progress()`'s numerator must reconstruct elapsed time from the
    // SAME element that determines the group's own end (`max_end`), not an arbitrary fixed
    // index, or the numerator and denominator describe two different notions of "the group."
    fn progress( &self ) -> f64
    {
      let last = self.iter().max_by
      (
        | a, b | ( a.delay + a.duration ).partial_cmp( &( b.delay + b.duration ) ).expect( "Animation keyframes can't be NaN" )
      ).expect( "N must be greater than 0" );

      if last.state == AnimationState::Pending
      {
        0.0
      }
      else
      {
        ( ( last.delay + last.time() - self.delay_get() ) / self.duration_get() ).clamp( 0.0, 1.0 )
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
      // `time` is the normalized [0, 1] interpolation factor; narrowing to f32 loses precision
      // but stays representable and visually indistinguishable at animation-frame granularity.
      let time = time as f32;
      self + ( other - self ) * time
    }
  }

  impl Animatable for f64
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      self + ( other - self ) * time
    }
  }

  impl Animatable for i32
  {
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      // Intentionally truncates the fractional part of the blended value to sample a discrete
      // integer; magnitude stays bounded by `self`/`other` for `time` within the intended [0, 1].
      
      ( f64::from( *self ) + ( f64::from( *other ) - f64::from( *self ) ) * time ) as i32
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
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      let mut copy = *self;
      copy.iter_mut().zip( other.iter() )
      .for_each( | ( elem, other_elem ) | *elem = elem.interpolate( other_elem, time ) );

      copy
    }
  }

  impl< E > Animatable for Vec< E >
  where E : MatEl + Animatable
  {
    // Fix(BUG-148)
    // Root cause: `self.iter().zip( other.iter() )` silently truncates to the shorter of the two
    // Vecs whenever their lengths differ, instead of surfacing the mismatch -- the exact same
    // defect shape `CubicHermite::new`/`apply` (`easing/cubic/hermite.rs`) already guard against
    // via `assert_eq!`, which this sibling `Animatable` impl had never been brought into line
    // with.
    // Pitfall: `Animatable::interpolate`'s own boundary contract (every scalar impl computes
    // `self + ( other - self ) * time`, so `time == 0.0` must equal `self` and `time == 1.0` must
    // equal `other`) is silently violated for the longer side's trailing elements whenever
    // lengths differ -- a loud panic on malformed input is correct here, not a recoverable error,
    // since `Animatable::interpolate` returns `Self` directly with no `Result` in the trait.
    fn interpolate( &self, other : &Self, time : f64 ) -> Self
    {
      assert_eq!
      (
        self.len(), other.len(),
        "Vec::interpolate: self and other must have the same length ( got {} and {} )", self.len(), other.len()
      );

      self.iter().zip( other.iter() )
      .map
      (
        | ( elem, other_elem ) |
        elem.interpolate( other_elem, time )
      )
      .collect::< Vec< _ > >()
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
    Tween
  };
}
