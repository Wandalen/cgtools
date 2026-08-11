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

      let half = E::from( 0.5 ).unwrap();

      let q =
      [
        half * n0.sqrt(),
        half * n1.sqrt() * ( r32 - r23 ).signum(),
        half * n2.sqrt() * ( r13 - r31 ).signum(),
        half * n3.sqrt() * ( r21 - r12 ).signum()
      ];

      Self( Vector::< E, 4 >::from( q ) )
    }
  }


}

crate::mod_interface!
{

}
