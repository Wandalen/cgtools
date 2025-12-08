mod private
{
    use std::marker::PhantomData;

    use crate::Animatable;


  /// A trait for all easing functions.
  ///
  /// An easing function takes a value `t` between 0.0 and 1.0 and
  /// transforms it to a new value, usually for animation purposes.
  ///
  pub trait EasingFunction : core::fmt::Debug
  {
    /// Type that can be interpolated by [`EasingFunction`]
    type AnimatableType;
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
    /// Creates a new `Box` containing an instance of the easing function.
    fn new() -> Box< T >;
  }

  /// Implements the `EasingBuilder` trait for a specified easing function.
  ///
  /// This macro generates a new public struct that acts as a builder for
  /// a specific easing function, allowing you to create a boxed instance
  /// of the function.
  #[ macro_export ]
  macro_rules! impl_easing_function
  {
    ( $builder_ty:ident, $function_ty:ty, $value:expr ) =>
    {
      /// A builder for the `EasingFunction` of type [`$function_ty`].
      ///
      /// This struct provides a way to create a boxed instance of the
      /// associated easing function.
      #[ non_exhaustive ]
      pub struct $builder_ty< A >( PhantomData< A > );

      impl< A > EasingBuilder< $function_ty, A > for $builder_ty< A >
      where A : Animatable
      {
        /// Creates a new `Box` containing an instance of the easing function.
        fn new() -> Box< $function_ty >
        {
          Box::new( $value )
        }
      }
    };
  }

  /// A basic linear easing function.
  ///
  /// The value returned is the same as the input `t`.
  #[ non_exhaustive ]
  #[ derive( Debug ) ]
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
    fn new() -> Box< Linear< A > >
    {
      Box::new( Linear( PhantomData ) )
    }
  }

  /// A step-based easing function.
  ///
  /// The output value progresses in discrete steps instead of a smooth gradient.
  #[ non_exhaustive ]
  #[ derive( Debug ) ]
  pub struct Step< A >
  where
    A : Animatable,
  {
    steps : f64,
    _marker : PhantomData< A >
  }

  impl< A > Step< A >
  where
    A : Animatable,
  {
    /// Init [`Step`] easing function
    pub fn new( steps : f64 ) -> Self
    {
      Self
      {
        steps,
        _marker : PhantomData
      }
    }
  }

  impl< A > EasingFunction for Step< A >
  where
    A : Animatable,
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
