mod private
{
  use crate::*;

  #[ derive( Clone, Copy, Debug, Default, PartialEq, PartialOrd ) ]
  pub struct Quat< E >( pub Vector< E, 4 > )
  where E : MatEl;
}

crate::mod_interface!
{
  layer general;
  layer operator;
  layer from;

  exposed use
  {
      Quat
  };
}
