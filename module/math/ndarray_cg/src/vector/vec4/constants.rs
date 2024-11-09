mod private
{
  use crate::*;

  impl F32x4 
  {
    /// Unit `x` vector
    pub const X : Self = Self::new( 1.0, 0.0, 0.0, 0.0 );

    /// Unit `y` vector
    pub const Y : Self = Self::new( 0.0, 1.0, 0.0, 0.0 );
    
    /// Unit `z` vector
    pub const Z : Self = Self::new( 0.0, 0.0, 1.0, 0.0 );

    /// Unit `w` vector
    pub const W : Self = Self::new( 0.0, 0.0, 0.0, 1.0);

    /// All elements are `f32::MIN`
    pub const MIN : Self = Self::splat( f32::MIN );

    /// All elements are `f32::MAX`
    pub const MAX : Self = Self::splat( f32::MAX );

    /// All elemets are `ZERO`
    pub const ZERO : Self = Self::splat( 0.0 );

    /// Creates a new vector
    #[inline(always)]
    pub const fn new( x : f32, y : f32, z : f32, w : f32 ) -> Self
    {
        Vector::< f32, 4 >( [ x, y, z, w ] )
    } 

    /// Creates a vector from a single value : [ v ; N ]
    #[inline(always)]
    pub const fn splat( v : f32 ) -> Self
    {
        Vector::< f32, 4 >( [ v; 4 ] )
    }
  }
  
}

crate::mod_interface!
{
  
}
