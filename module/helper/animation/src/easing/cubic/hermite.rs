mod private
{
  use crate::easing::base::EasingFunction;
  use mingl::{ MatEl, Vector, Mul, NdFloat };

  /// Hermite spline implementation for interpolation
  #[ non_exhaustive ]
  #[ derive( Debug, Clone ) ]
  pub struct CubicHermite< E, const N : usize >
  where E : MatEl + core::default::Default + core::marker::Copy
  {
    /// Tangent start
    pub m1 : Vector< E, N >,
    /// Tangent end
    pub m2 : Vector< E, N >
  }

  impl< E, const N : usize > CubicHermite< E, N >
  where E : MatEl + core::default::Default + core::marker::Copy
  {
    /// [`CubicHermite`] constructor
    pub fn new
    (
      m1 : Vector<E, N >,
      m2 : Vector<E, N >
    )
    -> Self
    {
      Self
      {
        m1,
        m2
      }
    }
  }

  impl< E, const N : usize > EasingFunction for CubicHermite< E, N >
  where
    E : MatEl +
    core::default::Default +
    core::marker::Copy +
    Mul< Vector< E, N >, Output = Vector< E, N > > +
    NdFloat
  {
    type AnimatableType = Vector< E, N >;

    fn apply( &self, start : Vector< E, N >, end : Vector< E, N >, time : f64 ) -> Vector< E, N >
    {
      let t2 = time * time;
      let t3 = t2 * time;

      Vector::splat( E::from( 2.0 * t3 - 3.0 * t2 + 1.0 ).unwrap() ) * start +
      Vector::splat( E::from( t3 - 2.0 * t2 + time ).unwrap() ) * self.m1 +
      Vector::splat( E::from( -2.0 * t3 + 3.0 * t2 ).unwrap() ) * end +
      Vector::splat( E::from( t3 - t2 ).unwrap() ) * self.m2
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    CubicHermite,

  };
}
