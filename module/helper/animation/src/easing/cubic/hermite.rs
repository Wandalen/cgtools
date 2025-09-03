mod private
{
  use crate::easing::base::EasingFunction;

  /// Hermite spline implementation for interpolation
  #[ derive( Debug ) ]
  pub struct CubicHermite
  {
    /// Value start
    pub v1 : f32,
    /// Tangent start
    pub m1 : f32,
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
      v1 : f32,
      m1 : f32,
      v2 : f32,
      m2 : f32
    )
    -> Self
    {
      Self
      {
        v1,
        m1,
        v2,
        m2
      }
    }
  }

  impl EasingFunction for CubicHermite
  {
    type EasingMethod = Hermite;

    fn apply( &self, time : f32 ) -> f32
    {
      let t = time;
      let t2 = t * t;
      let t3 = t2 * t;

      ( 2 * t3 - 3 * t2 + 1 ) * self.v1 +
      ( t3 - 2 * t2 + t ) * self.m1 +
      ( -2 * t3 + 3 * t2 ) * self.v2 +
      ( t3 - t2 ) * self.m2
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
