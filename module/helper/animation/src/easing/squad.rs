mod private
{
  use mingl::QuatF32;
  use crate::easing::base::
  {
    EasingFunction
  };

  /// A quaternion interpolation easing function.
  #[ non_exhaustive ]
  #[ derive( Debug ) ]
  pub struct Squad
  {
    start : QuatF32,
    end : QuatF32,
    in_tangent : QuatF32,
    out_tangent : QuatF32
  }

  impl Squad
  {
    /// Creates a new `Squad` easing function with tangent quaternions.
    pub fn new
    (
      start : QuatF32,
      end : QuatF32,
      in_tangent : QuatF32,
      out_tangent : QuatF32
    ) -> Self
    {
      Self
      {
        start,
        end,
        in_tangent,
        out_tangent
      }
    }
  }

  impl EasingFunction for Squad
  {
    type EasingMethod = Hermite;

    fn apply( &self, time : f32 ) -> f32
    {
      let b_start = self.start.slerp( &self.out_tangent, 1.0 / 3.0 );
      let b_end = self.end.slerp( &self.in_tangent, 1.0 / 3.0 );
      let slerp1 = self.start.slerp( &self.end, time );
      let slerp2 = b_start.slerp( &b_end, time );

      slerp1.slerp( &slerp2, 2.0 * time * ( 1.0 - time ) )
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
