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
          let g = helpers::circle_geometry( *segments );
          let len = g.len();
          ( g.into_iter().flatten().collect(), len )
        },
        Self::Miter =>
        {
          let g = miter_geometry();
          let len = g.len();
          ( g.into_iter().flatten().collect(), len )
        },
        Self::Bevel => 
        {
          let g = bevel_geometry();
          let len = g.len();
          ( g.into_iter().flatten().collect(), len )
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

  pub fn bevel_geometry() -> [ [ f32; 2 ]; 3 ]
  {
    [
      [ 0.0, 0.0 ],
      [ 1.0, 0.0 ],
      [ 0.0, 1.0 ],
    ]
  }

}

crate::mod_interface!
{
  own use crate::helpers::circle_geometry;

  own use
  {
    miter_geometry,
    bevel_geometry,
  };

  exposed use
  {
    Join
  };

}