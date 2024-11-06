mod private
{
  
}

pub mod d2;
pub mod sprite;

crate::mod_interface!
{
  own use 
  {
    d2,
    sprite
  };
}