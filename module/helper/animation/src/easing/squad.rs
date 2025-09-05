mod private
{
  use mingl::{ NdFloat, Quat, MatEl };
  use crate::easing::base::
  {
    EasingFunction
  };

  /// A quaternion interpolation easing function.
  #[ non_exhaustive ]
  #[ derive( Debug ) ]
  pub struct Squad< E >
  where
    E : MatEl + core::fmt::Debug + std::marker::Copy + std::default::Default
  {
    in_tangent : Quat< E >,
    out_tangent : Quat< E >
  }

  impl< E > Squad< E >
  where
    E : MatEl + core::fmt::Debug + std::marker::Copy + std::default::Default
  {
    /// Creates a new `Squad` easing function with tangent quaternions.
    pub fn new
    (
      in_tangent : Quat< E >,
      out_tangent : Quat< E >
    ) -> Self
    {
      Self
      {
        in_tangent,
        out_tangent
      }
    }
  }

  impl< E > EasingFunction for Squad< E >
  where
    E : MatEl + core::fmt::Debug + std::marker::Copy + std::default::Default + NdFloat
  {
    type AnimatableType = Quat< E >;

    fn apply( &self, start : Quat< E >, end : Quat< E >, time : f32 ) -> Quat< E >
    {
      let t = E::from( time ).unwrap();
      let b_start = start.slerp( &self.out_tangent, E::from( 1.0 / 3.0 ).unwrap() );
      let b_end = end.slerp( &self.in_tangent, E::from( 1.0 / 3.0 ).unwrap() );
      let slerp1 = start.slerp( &end, t );
      let slerp2 = b_start.slerp( &b_end, t );

      slerp1.slerp( &slerp2, E::from( 2.0 * time * ( 1.0 - time ) ).unwrap() )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Squad
  };
}
