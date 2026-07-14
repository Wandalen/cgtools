/// Internal namespace.
mod private
{
  
}

/// 2D texture utilities.
pub mod d2;
/// Cube texture utilities.
pub mod cube;
/// Compressed texture format capability detection.
#[ cfg( feature = "constants" ) ]
pub mod compressed;

crate::mod_interface!
{
  own use
  {
    d2,
    cube
  };

  #[ cfg( feature = "constants" ) ]
  own use compressed;
}
