mod private
{
  use crate::*;
  #[ cfg( feature = "serialization" ) ]
  use serde::{ Serialize, Deserialize };

  /// Represents the different types of line segment joins.
  #[ derive( Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  #[ cfg_attr( feature = "serialization", derive( Serialize, Deserialize ) ) ]
  pub enum Join
  {
    /// A round join, which is a circular arc connecting two line segments.
    /// Accepts the amount of segments for the circle
    Round( usize ),
    /// A miter join, where the outer edges of the line segments meet at a sharp point.
    Miter,
    /// A bevel join, where the corner is "cut off" by a straight line, creating a flat edge.
    Bevel
  }

  impl Default for Join 
  {
    fn default() -> Self 
    {
      Self::Round( 16 )
    }    
  }

  impl Geometry for Join 
  {
    fn geometry( &self ) -> ( Vec< f32 >, Vec< u32 >, Vec< f32 >, usize )
    {
      match self 
      {
        Self::Round( segments ) => 
        {
          let g = helpers::circle_geometry( *segments );
          let ind = Vec::new();
          let uv = Vec::new();
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().flatten().collect();
          ( g, ind, uv, len )
        },
        Self::Miter =>
        {
          let g = helpers::MITER_JOIN_GEOMETRY;
          let ind = Vec::new();
          let uv = Vec::new();
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().flatten().collect();
          ( g, ind, uv, len )
        },
        Self::Bevel => 
        {
          let g = helpers::BEVEL_JOIN_GEOMETRY;
          let ind = Vec::new();
          let uv = Vec::new();
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().flatten().collect();
          ( g, ind, uv, len )
        }
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Join
  };
}