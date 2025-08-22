mod private
{ 

  /// A trait for all easing functions.
  /// 
  /// An easing function takes a value `t` between 0.0 and 1.0 and
  /// transforms it to a new value, usually for animation purposes.
  /// 
  // #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub trait EasingFunction : core::fmt::Debug
  {
    /// Applies the easing function to a given value `t`.
    /// 
    /// The input `t` should be a value in the range [ 0.0, 1.0 ].
    fn apply( &self, time : f32 ) -> f32;
  }


  
  /// A trait for a builder of new easing function instance.
  pub trait EasingBuilder< T >
  where T : EasingFunction
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
      pub struct $builder_ty;

      impl EasingBuilder< $function_ty > for $builder_ty
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
  pub struct Linear;

  impl EasingFunction for Linear 
  {
    fn apply( &self, time : f32 ) -> f32
    {
      time
    }
  }

  impl EasingBuilder< Linear > for Linear
  {
    fn new() -> Box< Linear >
    {
      Box::new( Linear )
    }
  }

  /// A step-based easing function.
  /// 
  /// The output value progresses in discrete steps instead of a smooth gradient.
  #[ non_exhaustive ]
  #[ derive( Debug ) ]
  pub struct Step
  {
    steps : f32
  }

  impl Step
  {
    /// Init [`Step`] easing function
    pub fn new( steps : f32 ) -> Self
    {
      Self
      {
        steps
      }
    }
  }

  impl EasingFunction for Step 
  {
    fn apply( &self, time : f32 ) -> f32
    {
      ( time * self.steps ).floor() / self.steps
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