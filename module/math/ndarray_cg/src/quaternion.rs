mod private
{
  use crate::*;

  #[ derive( Clone, Copy, Debug, Default, PartialEq, PartialOrd ) ]
  pub struct Quat< E >( pub Vector< E, 4 > )
  where E : MatEl;

  pub type QuatF32 = Quat< f32 >;
  pub type QuatF64 = Quat< f64 >;
}

crate::mod_interface!
{
  layer general;
  layer operator;
  layer from;
  layer arithmetics;

  exposed use
  {
    Quat,
    QuatF32,
    QuatF64
  };
}
