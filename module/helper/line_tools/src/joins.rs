mod private
{
  use crate::*;
  use ndarray_cg as math;
  use math::F32x2;

  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub struct Miter;
  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub struct Round;
  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub struct Bevel;


  impl Round 
  {
    pub fn geometry( segments : usize ) -> Vec< F32x2 >
    {
      round_geometry( segments )
    }
  }

  impl Miter 
  {
    pub fn geometry() -> [ [ f32; 3 ]; 6 ]
    {
      [
        [ 0.0, 0.0, 0.0 ],
        [ 1.0, 0.0, 0.0 ],
        [ 0.0, 1.0, 0.0 ],
        [ 0.0, 0.0, 0.0 ],
        [ 0.0, 1.0, 0.0 ],
        [ 0.0, 0.0, 1.0 ]
      ]
    }
  }

  pub fn round_geometry( segments : usize ) -> Vec< F32x2 >
  {
    let mut positions = Vec::with_capacity( segments );
    for wedge in 0..=segments
    {
      let theta = 2.0 * std::f32::consts::PI * wedge as f32 / segments as f32;
      let ( s, c ) = theta.sin_cos();
      positions.push( F32x2::from( [ 0.5 * c, 0.5 * s ] ) )
    }

    positions
  }

  pub trait Join {}

  impl Join for Miter {}
  impl Join for Round {} 
  impl Join for Bevel {}

}

crate::mod_interface!
{

  own use
  {
    Miter,
    Round,
    Bevel,
    round_geometry
  };

}