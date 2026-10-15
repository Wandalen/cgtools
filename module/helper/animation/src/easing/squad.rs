mod private
{
  use mingl::{ NdFloat, Quat, MatEl };
  use crate::easing::base::
  {
    EasingFunction
  };

  /// A quaternion interpolation easing function.
  #[ non_exhaustive ]
  #[ derive( Debug, Clone ) ]
  pub struct Squad< E >
  where
    E : MatEl + core::fmt::Debug + core::marker::Copy + core::default::Default
  {
    in_tangent : Quat< E >,
    out_tangent : Quat< E >
  }

  impl< E > Squad< E >
  where
    E : MatEl + core::fmt::Debug + core::marker::Copy + core::default::Default
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

  /// Sources:
  ///  - `https://math.stackexchange.com/questions/2650188/super-confused-by-squad-algorithm-for-quaternion-interpolation`
  ///  - `https://github.com/phuicy/ROBOOP/blob/8bee84036b82362a74c7c5a73fa9aa2ab5cb54f8/source/quaternion.cpp#L722`
  ///  - `https://web.mit.edu/2.998/www/QuaternionReport1.pdf` ( Section 6.2.1, Page 51 )
  impl< E > EasingFunction for Squad< E >
  where
    E : MatEl + core::fmt::Debug + core::marker::Copy + core::default::Default + NdFloat
  {
    type AnimatableType = Quat< E >;

    // Fix(BUG-149)
    // Root cause: `apply` inserted an extraneous 1/3-blend step (`b_start`/`b_end`, blending
    // `start`/`end` toward the tangents) before the second slerp, instead of slerping
    // `out_tangent`/`in_tangent` directly against each other -- the correct SQUAD formula
    // (Shoemake's Definition 17, confirmed independently via this file's own cited ROBOOP and
    // MIT-thesis sources) is `Slerp( Slerp(start,end,t), Slerp(out_tangent,in_tangent,t),
    // 2t(1-t) )`, using the pre-computed tangent quaternions directly, with no further blending
    // toward the endpoints.
    // Pitfall: the outer `2t(1-t)` coefficient is exactly `0` at `time == 0.0` and `time == 1.0`,
    // so `apply` returns precisely `start`/`end` at both boundaries under BOTH the buggy and
    // fixed formula -- a boundary-only test (the style already used elsewhere in this crate) can
    // never catch this class of defect; only a pinned mid-curve value can.
    fn apply( &self, start : Quat< E >, end : Quat< E >, time : f64 ) -> Quat< E >
    {
      let time_e = E::from( time ).unwrap();
      let slerp1 = start.slerp( &end, time_e );
      let slerp2 = self.out_tangent.slerp( &self.in_tangent, time_e );

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
