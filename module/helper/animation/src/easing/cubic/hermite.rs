mod private
{
  use crate::easing::base::EasingFunction;
  use mingl::{ MatEl, Vector, Mul, NdFloat };

  /// Hermite spline implementation for interpolation
  #[ derive( Debug, Clone ) ]
  pub struct CubicHermite< T >
  {
    /// Tangent start
    pub m1 : T,
    /// Tangent end
    pub m2 : T
  }

  impl< E > CubicHermite< Vec< E > >
  where E : MatEl + std::default::Default + std::marker::Copy
  {
    /// [`CubicHermite`] constructor
    pub fn new
    (
      mut m1 : Vec< E >,
      mut m2 : Vec< E >
    )
    -> Self
    {
      let len = m1.len().min( m2.len() );
      m1.resize( len, E::default() );
      m2.resize( len, E::default() );

      Self
      {
        m1,
        m2
      }
    }
  }

  impl< E, const N : usize > CubicHermite< Vector< E, N > >
  where E : MatEl + std::default::Default + std::marker::Copy
  {
    /// [`CubicHermite`] constructor
    pub fn new
    (
      m1 : Vector< E, N >,
      m2 : Vector< E, N >
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

  impl< E, const N : usize > EasingFunction for CubicHermite< Vector< E, N > >
  where
    E : MatEl +
    std::default::Default +
    std::marker::Copy +
    Mul< Vector< E, N >, Output = Vector< E, N > > +
    NdFloat
  {
    type AnimatableType = Vector< E, N >;

    fn apply( &self, start : Vector< E, N >, end : Vector< E, N >, time : f64 ) -> Vector< E, N >
    {
      let t = time;
      let t2 = t * t;
      let t3 = t2 * t;

      Vector::splat( E::from( 2.0 * t3 - 3.0 * t2 + 1.0 ).unwrap() ) * start +
      Vector::splat( E::from( t3 - 2.0 * t2 + t ).unwrap() ) * self.m1 +
      Vector::splat( E::from( -2.0 * t3 + 3.0 * t2 ).unwrap() ) * end +
      Vector::splat( E::from( t3 - t2 ).unwrap() ) * self.m2
    }
  }

  impl< E > EasingFunction for CubicHermite< Vec< E > >
  where
    E : MatEl +
    std::default::Default +
    std::marker::Copy +
    NdFloat
  {
    type AnimatableType = Vec< E >;

    fn apply( &self, mut start : Vec< E >, mut end : Vec< E >, time : f64 ) -> Vec< E >
    {
      let len = start.len().min( end.len() ).min( self.m1.len() );
      start.resize( len, E::default() );
      end.resize( len, E::default() );

      let t = time;
      let t2 = t * t;
      let t3 = t2 * t;

      let mut result = vec![];

      for i in 0..len
      {
        result.push
        (
          E::from( 2.0 * t3 - 3.0 * t2 + 1.0 ).unwrap() * start[ i ] +
          E::from( t3 - 2.0 * t2 + t ).unwrap() * self.m1[ i ] +
          E::from( -2.0 * t3 + 3.0 * t2 ).unwrap() * end[ i ] +
          E::from( t3 - t2 ).unwrap() * self.m2[ i ]
        );
      }

      result
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
