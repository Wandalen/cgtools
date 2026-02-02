mod private
{
  use crate::*;

  impl< E : MatEl > From< [ E; 4 ] > for Quat< E >
  {
    fn from( value: [ E; 4 ] ) -> Self
    {
      Self( Vector::< E, 4 >::from( value ) )
    }
  }

  impl< E : MatEl > From< &[ E ] > for Quat< E >
  {
    fn from( value: &[ E ] ) -> Self
    {
      debug_assert!( value.len() > 4, "Slice should be at least of size 4 to create a Quaternion" );
      let array : [ E; 4 ] = value.try_into().unwrap();
      Self( Vector::< E, 4 >::from( array ) )
    }
  }

  impl< E : MatEl > From< ( E, E, E, E ) > for Quat< E >
  {
    fn from( value: ( E, E, E, E ) ) -> Self
    {
      let array = [ value.0, value.1, value.2, value.3 ];
      Self( Vector::< E, 4 >::from( array ) )
    }
  }

  /// Source: https://www.johndcook.com/blog/2025/05/07/quaternions-and-rotation-matrices/
  impl< E, Descriptor > From< Mat3< E, Descriptor > > for Quat< E >
  where
  E : MatEl + nd::NdFloat,
  Descriptor : mat::Descriptor,
  Mat3< E, Descriptor > : RawSliceMut< Scalar = E > +
  ScalarMut< Scalar = E, Index = Ix2 > +
  ConstLayout< Index = Ix2 > +
  IndexingMut< Scalar = E, Index = Ix2 >
  {
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
