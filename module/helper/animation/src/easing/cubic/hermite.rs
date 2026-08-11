mod private
{
  use crate::easing::base::EasingFunction;
  use mingl::{ MatEl, Vector, Mul, NdFloat };

  /// Hermite spline implementation for interpolation
  #[ non_exhaustive ]
  #[ derive( Debug, Clone ) ]
  pub struct CubicHermite< T >
  {
    /// Tangent start
    pub m1 : T,
    /// Tangent end
    pub m2 : T
  }

  impl< E > CubicHermite< Vec< E > >
  where E : MatEl + core::default::Default + core::marker::Copy
  {
    /// [`CubicHermite`] constructor
    ///
    /// # Panics
    /// Panics if `m1` and `m2` have different lengths — the two tangent vectors must describe
    /// the same number of components.
    // Fix(TASK-041): silently resized both tangents down to the shorter length instead of
    // surfacing the mismatch, discarding trailing tangent data with no signal to the caller.
    // Root cause: `.resize()` used as an ad hoc normalization instead of validating the
    // precondition that both tangents share one dimensionality.
    // Pitfall: `EasingFunction::apply` returns `Self::AnimatableType` (no `Result`) for every
    // implementor, so this can't become a `Result` without changing the shared trait — a loud
    // panic on malformed caller input is the correct fix at this call site.
    pub fn new
    (
      m1 : Vec< E >,
      m2 : Vec< E >
    )
    -> Self
    {
      assert_eq!
      (
        m1.len(), m2.len(),
        "CubicHermite::new: m1 and m2 must have the same length ( got {} and {} )", m1.len(), m2.len()
      );

      Self
      {
        m1,
        m2
      }
    }
  }

  impl< E, const N : usize > CubicHermite< Vector< E, N > >
  where E : MatEl + core::default::Default + core::marker::Copy
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

  impl< E > EasingFunction for CubicHermite< Vec< E > >
  where
    E : MatEl +
    core::default::Default +
    core::marker::Copy +
    NdFloat
  {
    type AnimatableType = Vec< E >;

    /// # Panics
    /// Panics if `start`, `end`, and the tangents (`m1`, `m2`, guaranteed equal-length by
    /// [`CubicHermite::new`]) don't all share the same length.
    // Fix(TASK-041): silently truncated `start`/`end` down to the shortest of 3 independent
    // lengths instead of surfacing the mismatch, discarding trailing components with no signal.
    // Root cause: same ad hoc `.resize()`-as-normalization pattern as the constructor.
    // Pitfall: see constructor's Pitfall — the shared `EasingFunction` trait signature has no
    // `Result`, so a loud panic on malformed input is the correct fix here too.
    fn apply( &self, start : Vec< E >, end : Vec< E >, time : f64 ) -> Vec< E >
    {
      assert_eq!
      (
        start.len(), end.len(),
        "CubicHermite::apply: start and end must have the same length ( got {} and {} )", start.len(), end.len()
      );
      assert_eq!
      (
        start.len(), self.m1.len(),
        "CubicHermite::apply: start/end length must match tangent length ( got {} and {} )", start.len(), self.m1.len()
      );

      let len = start.len();

      let t2 = time * time;
      let t3 = t2 * time;

      let mut result = vec![];

      for i in 0..len
      {
        result.push
        (
          E::from( 2.0 * t3 - 3.0 * t2 + 1.0 ).unwrap() * start[ i ] +
          E::from( t3 - 2.0 * t2 + time ).unwrap() * self.m1[ i ] +
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
