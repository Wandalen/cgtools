mod private
{
  use crate::*;

  impl F32x2
  {

    /// Unit `x` vector
    pub const X : Self = Self::new( 1.0, 0.0 );

    /// Unit `y` vector
    pub const Y : Self = Self::new( 0.0, 1.0 );

    /// Minus unit `x` vector
    pub const NEG_X : Self = Self::new( -1.0, 0.0 );

    /// Minus unit `y` vector
    pub const NEG_Y : Self = Self::new( 0.0, -1.0 );

    /// All elements are `f32::MIN`
    pub const MIN : Self = Self::splat( f32::MIN );

    /// All elements are `f32::MAX`
    pub const MAX : Self = Self::splat( f32::MAX );

    /// All elemets are `ZERO`
    pub const ZERO : Self = Self::splat( 0.0 );
  }
}

crate::mod_interface!
{

}
