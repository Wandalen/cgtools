mod private
{
  use crate::{Quat, MatEl, Vector, TryInto, Mat3, nd, mat, RawSliceMut, ScalarMut, Ix2, ConstLayout, IndexingMut};

  impl< E : MatEl > From< [ E; 4 ] > for Quat< E >
  {
    #[ inline ]
    fn from( value: [ E; 4 ] ) -> Self
    {
      Self( Vector::< E, 4 >::from( value ) )
    }
  }

  impl< E : MatEl > From< &[ E ] > for Quat< E >
  {
    // Fix(TASK-014): removed the `debug_assert!( value.len() > 4, .. )` line entirely.
    // Root cause: the condition used `> 4` (strictly greater than 4) instead of `>= 4`,
    // so a valid, correctly-sized 4-element slice failed the assertion in every debug
    // build. The check was also fully redundant: the very next line,
    // `value.try_into().unwrap()`, already panics unconditionally (in every build
    // profile, not just debug) when `value.len() != 4`.
    // Pitfall: a `debug_assert!` duplicating a check that another, always-on code path
    // already performs can silently drift out of sync with it (here: `> 4` vs the real
    // `== 4` requirement) without being noticed, since release builds never evaluate the
    // drifted condition.
    #[ inline ]
    fn from( value: &[ E ] ) -> Self
    {
      let array : [ E; 4 ] = value.try_into().unwrap();
      Self( Vector::< E, 4 >::from( array ) )
    }
  }

  impl< E : MatEl > From< ( E, E, E, E ) > for Quat< E >
  {
    #[ inline ]
    fn from( value: ( E, E, E, E ) ) -> Self
    {
      let array = [ value.0, value.1, value.2, value.3 ];
      Self( Vector::< E, 4 >::from( array ) )
    }
  }

  /// Source: <https://www.johndcook.com/blog/2025/05/07/quaternions-and-rotation-matrices/>
  impl< E, Descriptor > From< Mat3< E, Descriptor > > for Quat< E >
  where
  E : MatEl + nd::NdFloat,
  Descriptor : mat::Descriptor,
  Mat3< E, Descriptor > : RawSliceMut< Scalar = E > +
  ScalarMut< Scalar = E, Index = Ix2 > +
  ConstLayout< Index = Ix2 > +
  IndexingMut< Scalar = E, Index = Ix2 >
  {
    #[ inline ]
    fn from( value : Mat3< E, Descriptor > ) -> Self
    {
      let r11 = *value.scalar_ref( Ix2( 0, 0 ) );
      let r12 = *value.scalar_ref( Ix2( 0, 1 ) );
      let r13 = *value.scalar_ref( Ix2( 0, 2 ) );

      let r21 = *value.scalar_ref( Ix2( 1, 0 ) );
      let r22 = *value.scalar_ref( Ix2( 1, 1 ) );
      let r23 = *value.scalar_ref( Ix2( 1, 2 ) );

      let r31 = *value.scalar_ref( Ix2( 2, 0 ) );
      let r32 = *value.scalar_ref( Ix2( 2, 1 ) );
      let r33 = *value.scalar_ref( Ix2( 2, 2 ) );

      let n0 = E::one() + r11 + r22 + r33;
      let n1 = E::one() + r11 - r22 - r33;
      let n2 = E::one() - r11 + r22 - r33;
      let n3 = E::one() - r11 - r22 + r33;

      // Fix(BUG-447): clamp each `n0..n3` to `>= 0` before `.sqrt()`.
      // Root cause: each `n*` is algebraically `1 +/- r11 +/- r22 +/- r33`, which is only
      // guaranteed non-negative when `value` is an *exactly* orthonormal rotation matrix.
      // Floating-point rounding (or an approximately-but-not-exactly orthonormal caller-supplied
      // matrix, e.g. one accumulated from repeated transform composition) can drive one term
      // marginally negative, and `.sqrt()` of a negative input silently returns `NaN` -- which
      // then propagates into every component of the resulting quaternion via the `half * n*.sqrt()`
      // products below.
      // Pitfall: an algebraic identity that guarantees non-negativity only for *exact* inputs
      // (here: an exactly orthonormal matrix) does not carry that guarantee into finite-precision
      // floating point -- always clamp before `.sqrt()`/`.acos()`/`.asin()` when the domain
      // constraint is a mathematical property of exact inputs, not a syntactic property of the
      // formula itself; see the identical pattern at BUG-272 (`Quat::to_euler_xyz`'s `asin`) and
      // BUG-446 (`vector::angle`'s `acos`).
      let n0 = n0.max( E::zero() );
      let n1 = n1.max( E::zero() );
      let n2 = n2.max( E::zero() );
      let n3 = n3.max( E::zero() );

      let half = E::from( 0.5 ).unwrap();

      // Fix(BUG-119): reordered the array from `[n0,n1,n2,n3]` to `[n1,n2,n3,n0]`-based slots.
      // Root cause: `n0` is the trace-derived term (proportional to `w²`), while `n1`/`n2`/`n3`
      // are proportional to `x²`/`y²`/`z²` respectively (standard Shepperd's-method algebra) —
      // but this crate's `Quat` stores components in `[x,y,z,w]` order (confirmed by
      // `from_angle_x`/`from_angle_z` and the reverse conversion `Mat3::from_quat`, both of
      // which put the axis component first and the scalar/cosine term last). Building the
      // array as `[n0,n1,n2,n3]` and storing it directly therefore wrote `w` into the `x`
      // slot, `x` into the `y` slot, `y` into the `z` slot, and `z` into the `w` slot — a
      // cyclic shift, not a random scramble, which made it easy to miss by inspection.
      // Pitfall: when a derivation names its intermediate terms `n0..n3` in the order they're
      // *computed* (trace term first, purely for algebraic convenience), that order can
      // silently diverge from the order the target type actually *stores* its components in
      // — always map each intermediate back to its named component before assembling the
      // final array, rather than assuming computation order matches storage order.
      let q =
      [
        half * n1.sqrt() * ( r32 - r23 ).signum(),
        half * n2.sqrt() * ( r13 - r31 ).signum(),
        half * n3.sqrt() * ( r21 - r12 ).signum(),
        half * n0.sqrt()
      ];

      Self( Vector::< E, 4 >::from( q ) )
    }
  }


}

crate::mod_interface!
{

}
