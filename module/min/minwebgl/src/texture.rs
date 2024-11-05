mod private
{
  
}

pub mod d2;
pub mod sprite;

crate::mod_interface!
{
  exposed use 
  {
    d2,
    sprite
  };
}