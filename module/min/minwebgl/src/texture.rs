mod private
{
  
}

pub mod d2;
pub mod cube;

crate::mod_interface!
{
  own use 
  {
    d2,
    cube
  };
}