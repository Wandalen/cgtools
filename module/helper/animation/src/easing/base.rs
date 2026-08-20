mod private
{
    use core::marker::PhantomData;

    use crate::Animatable;


  /// A trait for all easing functions.
  ///
  /// An easing function takes a value `t` between 0.0 and 1.0 and
  /// transforms it to a new value, usually for animation purposes.
  ///
  pub trait EasingFunction : core::fmt::Debug + clone_dyn_types::CloneDyn
  {
    /// Type that can be interpolated by [`EasingFunction`]
    type AnimatableType : Clone;
    /// Applies the easing function to a given value `t`.
    ///
    /// The input `t` should be a value in the range [ 0.0, 1.0 ].
    fn apply
    (
      &self,
      start : Self::AnimatableType,
      end : Self::AnimatableType,
      time : f64
    )
    -> Self::AnimatableType;
  }

  /// A trait for a builder of new easing function instance.
  pub trait EasingBuilder< T, A >
  where
    T : EasingFunction< AnimatableType = A >,
    A : Animatable,
  {
    /// Builds a `Box` containing an instance of the easing function.
    fn build() -> Box< T >;
  }

  /// A basic linear easing function.
  ///
  /// The value returned is the same as the input `t`.
  #[ non_exhaustive ]
  #[ derive( Debug, Clone ) ]
  pub struct Linear< A >( PhantomData< A > ) where A : Animatable;

  impl< A > EasingFunction for Linear< A >
  where A : Animatable
  {
    type AnimatableType = A;

    fn apply( &self, start : Self::AnimatableType, end : Self::AnimatableType, time : f64 ) -> Self::AnimatableType
    {
      start.interpolate( &end, time )
    }
  }

  impl< A > EasingBuilder< Linear< A >, A > for Linear< A >
  where A : Animatable
  {
    fn build() -> Box< Linear< A > >
    {
      Box::new( Linear( PhantomData ) )
    }
  }

  /// A step-based easing function.
  ///
  /// The output value progresses in discrete steps instead of a smooth gradient.
  #[ non_exhaustive ]
  #[ derive( Debug, Clone ) ]
  pub struct Step< A >
  where
    A : Animatable
  {
    steps : f64,
    _marker : PhantomData< A >
  }

  impl< A > Step< A >
  where
    A : Animatable,
  {
    /// Init [`Step`] easing function
    // Fix(BUG-233)
    // Root cause: `steps` was stored as given, with no floor -- `Step::new( 0.0 )` reached
    // `apply`'s `( time * self.steps ).ceil() / self.steps` with a `0.0` divisor, and `f64`
    // division never panics, so `0.0 / 0.0` silently produced `NaN` instead of erroring.
    // Pitfall: mirrors `Tween::new`'s own `duration.max( 0.001 )` floor -- any `f64` constructor
    // parameter that later becomes a division's divisor needs the same guard, since Rust's
    // float division silently returns `NaN`/`inf` rather than panicking on zero.
    #[must_use]
    pub fn new( steps : f64 ) -> Self
    {
      Self
      {
        steps : steps.max( 0.001 ), // Minimum step count to avoid division by zero
        _marker : PhantomData
      }
    }
  }

  impl< A > EasingFunction for Step< A >
  where
    A : Animatable
  {
    type AnimatableType = A;

    fn apply( &self, start : Self::AnimatableType, end : Self::AnimatableType, time : f64 ) -> Self::AnimatableType
    {
      let time = ( time * self.steps ).ceil() / self.steps;
      start.interpolate( &end, time )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    EasingBuilder,
    EasingFunction,
    Linear,
    Step
  };
}
