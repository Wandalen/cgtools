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
  impl< E : MatEl + nd::NdFloat > From< F32x3x3 > for Quat< E >
  {
    fn from( value : F32x3x3 ) -> Self
    {
      let
      [
        r11, r21, r31,
        r12, r22, r32,
        r13, r23, r33
      ]
      = value.to_array();

      let n0 = E::one() + E::from( r11 + r22 + r33 ).unwrap();
      let n1 = E::one() + E::from( r11 - r22 - r33 ).unwrap();
      let n2 = E::one() + E::from( - r11 + r22 - r33 ).unwrap();
      let n3 = E::one() + E::from( - r11 - r22 + r33 ).unwrap();

      let half = E::from( 0.5 ).unwrap();

      let q =
      [
        half * n0.sqrt(),
        half * n1.sqrt() * E::from( ( r32 - r23 ).signum() ).unwrap(),
        half * n2.sqrt() * E::from( ( r13 - r31 ).signum() ).unwrap(),
        half * n3.sqrt() * E::from( ( r21 - r12 ).signum() ).unwrap()
      ];

      Self( Vector::< E, 4 >::from( q ) )
    }
  }


}

crate::mod_interface!
{

}
