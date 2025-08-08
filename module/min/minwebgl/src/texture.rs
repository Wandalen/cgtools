/// Internal namespace.
mod private
{
  
}

/// A module containing tools for working with 2D textures.
pub mod d2;
/// A module containing tools for working with cube textures.
pub mod cube;

crate::mod_interface!
{
  own use 
  {
    d2,
    cube
  };
}
