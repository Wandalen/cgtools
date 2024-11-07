use crate::*;

impl< E > Mat4< E > 
where E : MatEl + nd::NdFloat
{
  pub fn to_array( &self ) -> [ E; 16 ]
  {
    self.raw_slice().try_into().unwrap()
  }    
}