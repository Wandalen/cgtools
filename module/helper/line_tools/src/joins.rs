mod private
{
  #[ cfg( feature = "serialization" ) ]
  use serde::{ Serialize, Deserialize };

  /// Represents the different types of line segment joins.
  #[ derive( Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  #[ cfg_attr( feature = "serialization", derive( Serialize, Deserialize ) ) ]
  pub enum Join
  {
    /// A round join, which is a circular arc connecting two line segments.
    /// The `usize` parameter specifies the number of segments used to approximate the curve.
    Round( usize ),
    /// A miter join, where the outer edges of the line segments meet at a sharp point.
    Miter,
    /// A bevel join, where the corner is "cut off" by a straight line, creating a flat edge.
    Bevel
  }

  impl Join
  {
    /// Generates the geometry for the specified join type.
    ///
    /// This method returns a tuple containing the vertices, indices, and the number of
    /// elements for the join's mesh.
    pub fn geometry( &self ) -> ( Vec< f32 >, Vec< u32 >, usize )
    {
      match self 
      {
        Self::Round( segments ) => 
        {
          let ( g, ind ) = round_geometry( *segments );
          let len = g.len();
          ( 
            g.into_iter().map( | v | v as f32 ).collect(), 
            ind.into_iter().flatten().collect(), 
            len 
          )
        },
        Self::Miter =>
        {
          let g = miter_geometry();
          let len = g.len();
          ( g.into_iter().flatten().collect(), Vec::new(), len )
        },
        Self::Bevel => 
        {
          let g = bevel_geometry();
          let len = g.len();
          ( g.into_iter().flatten().collect(), Vec::new(), len )
        }
      }
    }

    pub fn uv( &self ) -> Vec< f32 >
    {
      match self
      {
        Self::Miter =>
        {
          miter_uv().into_iter().flatten().collect()
        },
        _ =>
        {
          Vec::new()
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

  /// Generates the vertex data for a bevel join.
  pub fn bevel_geometry() -> [ [ f32; 3 ]; 3 ]
  {
    [
      [ 1.0, 0.0, 0.0 ],
      [ 0.0, 1.0, 0.0 ],
      [ 0.0, 0.0, 1.0 ],
    ]
  }

  /// Generates the vertex data for a miter join.
  pub fn miter_geometry() -> [ [ f32; 4 ]; 6 ]
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

  pub fn miter_uv() -> [ [ f32; 2 ] ; 6 ]
  {
    [ 
      [ 0.0, 0.0 ], [ 0.0, 0.0 ], [ 1.0, 0.0 ],
      [ 2.0, 1.0 ], [ 1.0, 1.0 ], [ 2.0, 1.0 ] 
    ]
  }

  /// Generates the vertex IDs and triangle indices for a round join.
  pub fn round_geometry( segments : usize ) -> ( Vec< u32 >, Vec< [ u32; 3 ] > )
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