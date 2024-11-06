mod private
{
  
}

pub mod d2;
pub mod video;

crate::mod_interface!
{
  own use 
  {
    d2,
    video
  };
}