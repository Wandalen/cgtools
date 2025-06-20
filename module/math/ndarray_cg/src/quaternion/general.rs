mod private
{
  use crate::{ mat::DescriptorOrderColumnMajor, * };

  impl< E > Quat< E >
  where 
    E : MatEl
  {
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    pub fn w( &self ) -> E
    {
      self.0[ 3 ]
    }
  }

  impl< E > Quat< E >
  where 
    E : MatEl + nd::NdFloat
  {
    /// Transform the quaterion into a column major 3x3 rotation matrix
    pub fn to_matrix( &self ) -> Mat3< E, DescriptorOrderColumnMajor >
    where 
      Mat3< E, DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
    {
      Mat3::< E, DescriptorOrderColumnMajor >::from_quat( *self )
    }
  }
}

crate::mod_interface!
{
  
}
