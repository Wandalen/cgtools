mod private
{
  use crate::*;
  use ndarray_cg as math;
  use math::F32x2;

  #[ derive( Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub enum Join
  {
    Round( usize ),
    Miter,
    Bevel
  }

  impl Join
  {
    pub fn geometry( &self ) -> ( Vec< f32 >, usize )
    {
      match self 
      {
        Self::Round( segments ) => 
        {
          let g = round_geometry( *segments );
          let len = g.len();
          ( g.into_iter().flatten().collect(), len )
        },
        Self::Miter =>
        {
          let g = miter_geometry();
          let len = g.len();
          ( g.into_iter().flatten().collect(), len )
        },
        _ => 
        {
          ( Vec::new(), 0 )
        }
      }
    }
  }

  impl Default for Join 
  {
    fn default() -> Self 
    {
      Self::Round( 16 )
    }    
  }

  pub fn miter_geometry() -> [ [ f32; 3 ]; 6 ]
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

  pub fn round_geometry( segments : usize ) -> Vec< [ f32; 2 ] >
  {
    let mut positions = Vec::with_capacity( segments );
    for wedge in 0..=segments
    {
      let theta = 2.0 * std::f32::consts::PI * wedge as f32 / segments as f32;
      let ( s, c ) = theta.sin_cos();
      positions.push( [ 0.5 * c, 0.5 * s ] )
    }

    positions
  }



}

crate::mod_interface!
{

  own use
  {
    round_geometry,
    miter_geometry,
  };

  exposed use
  {
    Join
  };

}