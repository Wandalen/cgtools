mod private
{
  // use crate::easing::base::
  // {
  //   EasingFunction,
  //   EasingBuilder
  // };

  /// Hermite spline implementation for interpolation
  #[ derive( Debug ) ]
  pub struct CubicHermite
  {
    /// Time start
    pub t1 : f32,
    /// Value start
    pub v1 : f32,
    /// Tangent start
    pub m1 : f32,
    /// Time end
    pub t2 : f32,
    /// Value end
    pub v2 : f32,
    /// Tangent end
    pub m2 : f32
  }

  impl CubicHermite
  {
    /// [`CubicHermite`] constructor
    pub fn new
    (
      t1 : f32,
      v1 : f32,
      m1 : f32,
      t2 : f32,
      v2 : f32,
      m2 : f32
    ) -> Self
    {
      Self
      {
        t1,
        v1,
        m1,
        t2,
        v2,
        m2
      }
    }
  }

  // impl EasingFunction for CubicHermite
  // {
  //   fn apply( &self, time : f32 ) -> f32
  //   {
  //     let t = time;
  //     let t2 = t * t;
  //     let t3 = t2 * t;

  //     return ( 2 * t3 - 3 * t2 + 1 ) * self.p1 +
  //     ( t3 - 2 * t2 + t ) * self.m1 +
  //     ( -2 * t3 + 3 * t2 ) * self.p2 +
  //     (t3 - t2) * self.m2;
  //   }
  // }
}

crate::mod_interface!
{
  orphan use
  {
    CubicHermite,

  };
}
