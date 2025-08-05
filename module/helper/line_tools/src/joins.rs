mod private
{
  use crate::*;
  use ndarray_cg as math;
  use math::F32x2;
  use serde::{ Serialize, Deserialize };

  #[ derive( Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize ) ]
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

    // Returns vertices, indices, and the amount of elements
    pub fn geometry_merged( &self ) -> ( Vec< f32 >, Vec< u32 >, usize )
    {
      match self 
      {
        Self::Round( segments ) => 
        {
          let ( g, ind ) = round_geometry_merged( *segments );
          let len = g.len();
          ( 
            g.into_iter().map( | v | v as f32 ).collect(), 
            ind.into_iter().flatten().collect(), 
            len 
          )
        },
        Self::Miter =>
        {
          let g = miter_geometry_merged();
          let len = g.len();
          ( g.into_iter().flatten().collect(), Vec::new(), len )
        },
        Self::Bevel => 
        {
          let g = bevel_geometry_merged();
          let len = g.len();
          ( g.into_iter().flatten().collect(), Vec::new(), len )
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

  pub fn bevel_geometry_merged() -> [ [ f32; 3 ]; 3 ]
  {
    [
      [ 1.0, 0.0, 0.0 ],
      [ 0.0, 1.0, 0.0 ],
      [ 0.0, 0.0, 1.0 ],
    ]
  }

  pub fn miter_geometry_merged() -> [ [ f32; 4 ]; 6 ]
  {
    [
      [ 0.0, 0.0, 0.0, 1.0 ],
      [ 1.0, 0.0, 0.0, 0.0 ],
      [ 0.0, 1.0, 0.0, 0.0 ],
      [ 0.0, 0.0, 0.0, 1.0 ],
      [ 0.0, 1.0, 0.0, 0.0 ],
      [ 0.0, 0.0, 1.0, 0.0 ]
    ]
  }

  pub fn round_geometry_merged( segments : usize ) -> ( Vec< u32 >, Vec< [ u32; 3 ] > )
  {
    let mut ids = Vec::with_capacity( segments );
    let mut cells = Vec::with_capacity( segments );

    for i in 0..( segments + 2 )
    {
      let i = i as u32;
      ids.push( i );
    }

    for i in 0..segments
    {
      let i = i as u32;
      cells.push( [ 0, i + 1, i + 2 ] );
    }

    ( ids, cells )
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