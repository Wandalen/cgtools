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
}

crate::mod_interface!
{
  
}
