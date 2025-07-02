mod private
{

  use crate::*;

  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub enum Cap
  {
    #[ default ]
    Butt,
    Round( usize ),
    Square
  }

}

crate::mod_interface!
{  
  exposed use
  {
    Cap
  };
}