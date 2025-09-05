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
    /// Accepts level of triangualtion in the horizontal and vertical directions
    Round( usize, usize ),
    /// A miter join, where the outer edges of the line segments meet at a sharp point.
    /// Accepts level of triangualtion in the horizontal and vertical directions
    Miter( usize, usize ),
    /// A bevel join, where the corner is "cut off" by a straight line, creating a flat edge.
    /// Accepts level of triangualtion in the horizontal and vertical directions
    Bevel( usize, usize )
  }

  impl Default for Join 
  {
    fn default() -> Self 
    {
      Self::Round( 16, 8 )
    }    
  }

  impl Geometry for Join 
  {
    fn geometry( &self ) -> ( Vec< f32 >, Vec< u32 >, Vec< f32 >, usize )
    {
      match self 
      {
        Self::Round( row_precision, column_precision ) => 
        {
          let ( g, uv ) = helpers::triangulated_arc_geometry( *row_precision, *column_precision );
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().map( | v | v.to_array() ).flatten().collect();
          let ind = Vec::new();
          ( g, ind, uv, len )
        },
        Self::Miter( row_precision, column_precision ) =>
        {
          let ( g, uv ) = helpers::triangulated_miter_geometry( *row_precision, *column_precision );
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().map( | v | v.to_array() ).flatten().collect();
          let ind = Vec::new();
          ( g, ind, uv, len )
        },
        Self::Bevel( row_precision, column_precision ) => 
        {
          let ( g, uv ) = helpers::triangulated_bevel_geometry( *row_precision, *column_precision );
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().map( | v | v.to_array() ).flatten().collect();
          let ind = Vec::new();
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