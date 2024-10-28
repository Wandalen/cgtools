mod private
{
  //pub use super::d2 as d2; 
}

pub mod d2;

crate::mod_interface!
{
  exposed use 
  {
    d2
  };
}