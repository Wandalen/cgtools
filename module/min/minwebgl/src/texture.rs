mod private
{
  
}

/// 2D texture utilities.
pub mod d2;
/// Cube texture utilities.
pub mod cube;

crate::mod_interface!
{
  own use 
  {
    d2,
    cube
  };
}