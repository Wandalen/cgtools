mod private
{ 
  // #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub trait EasingFunction
  {
    pub fn apply( &self, t : f32 ) -> f32;
  }

  pub trait EasingBuilder< T >
  where T : EasingFunction
  {
    fn new() -> T;
  }

  macro_rules! impl_easing_function
  {
    ( $builder_ty:ty, $function_ty:ty, $value:expr ) =>
    {
      struct $builder_ty;

      impl EasingBuilder< $function_ty > for $builder_ty
      {
        fn new() -> $builder_ty
        {
          $value
        }
      }
    };
  }

  pub struct Linear;

  impl EasingFunction for Linear 
  {
    pub fn apply( &self, t : f32 ) -> f32
    {
      t
    }
  }

  pub struct Step
  {
    steps : f32
  };

  impl EasingFunction for Step 
  {
    pub fn apply( &self, t : f32 ) -> f32
    {
      ( t * self.steps ).floor() / self.steps
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    impl_easing_function,
    EasingBuilder,
    EasingFunction,
    Linear,
    Step
  };
}