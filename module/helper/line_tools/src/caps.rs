mod private
{

  use crate::*;

  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub struct Butt;
  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub struct Round;
  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub struct Square;

  impl< Join > d2::Line< Butt, Join >  
  {
    pub fn cap_geometry() -> [ [ f32; 2 ]; 6 ]
    {
      [
        [ 0.0, -0.5 ],
        [ 1.0, -0.5 ],
        [ 1.0,  0.5 ],
        [ 0.0, -0.5 ],
        [ 1.0,  0.5 ],
        [ 0.0,  0.5 ]
      ]
    }
  }

  pub trait Cap {}

  impl Cap for Butt {}
  impl Cap for Round {} 
  impl Cap for Square {}

}

crate::mod_interface!
{

  own use
  {
    Butt,
    Round,
    Square
  };
  
  exposed use
  {
    Cap
  };
}