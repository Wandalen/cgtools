mod private
{
  /// Trait for types that can be animated ( interpolated ).
  pub trait Animatable : Clone + core::fmt::Debug
  {
    /// Interpolates between two values at time t ( 0.0 to 1.0 ).
    fn interpolate( &self, other : &Self, t : f64 ) -> Self;
  }

  /// Trait for type-erased animatable values in Sequencer.
  pub trait AnimatablePlayer : core::fmt::Debug + clone_dyn_types::CloneDyn
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
    /// Returns a type-erased mutable reference to the underlying value.
    fn as_any_mut( &mut self ) -> &mut dyn core::any::Any;
    /// Returns animation duration
    fn duration_get( &self ) -> f64;
    /// Returns animation delay
    fn delay_get( &self ) -> f64;
    /// Gets the progress of the animated value ( 0.0 to 1.0 ).
    fn progress( &self ) -> f64;
  }
}

crate::mod_interface!
{
  orphan use
  {
    Animatable,
    AnimatablePlayer
  };
}
