mod private
{
  use crate::*;
  #[ cfg( feature = "serialization" ) ]
  use serde::{ Serialize, Deserialize };

  /// Represents the different types of line caps.
  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  #[ cfg_attr( feature = "serialization", derive( Serialize, Deserialize ) ) ]
  pub enum Cap
  {
    /// A butt cap, which is a flat end perpendicular to the line segment's direction.
    /// It's the default cap style.
    #[ default ]
    Butt,
    /// A round cap, which is a semicircular end.
    /// The `usize` parameter specifies the number of segments used to approximate the curve.
    Round( usize ),
    /// A square cap, which extends the line segment by half its width.
    Square
  }

  impl Geometry for Cap
  {
    fn geometry( &self ) -> ( Vec< f32 >, Vec< u32 >, Vec< f32 >, usize ) 
    {
      match self 
      {
        Self::Round( segments ) => 
        {
          let g = helpers::circle_left_half_geometry( *segments );
          let len = g.len();
          let ind = Vec::new();
          let uv = Vec::new();
          let g = g.into_iter().flatten().collect();
          ( g, ind, uv, len )
        },
        Self::Square =>
        {
          let g = helpers::BODY_GEOMETRY;
          let len = g.len();
          let ind = Vec::new();
          let uv = Vec::new();
          let g = g.into_iter().flatten().collect();
          ( g, ind, uv, len )
        },
        Self::Butt => 
        {
          ( Vec::new(), Vec::new(), Vec::new(), 0 )
        }
      }
    }
  }
}

crate::mod_interface!
{  
  orphan use
  {
    Cap
  };
}