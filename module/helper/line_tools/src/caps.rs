mod private
{
  use serde::{ Serialize, Deserialize };

  /// Represents the different types of line caps.
  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize ) ]
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

  impl Cap 
  {
    /// Generates the geometry for the specified cap type.
    ///
    /// This method returns a tuple containing the vertices, indices, and the number of vertices
    /// for the cap's mesh.
    pub fn geometry( &self ) -> ( Vec< f32 >, Vec< u32 >, usize )
    {
      match self 
      {
        Self::Round( segments ) => 
        {
          let ( g, ind ) = round_cap_geometry( *segments );
          let len = g.len();
          ( 
            g.into_iter().flatten().collect(), 
            ind.into_iter().flatten().collect(),  
            len 
          )
        },
        Self::Square =>
        {
          let ( g, ind ) = square_cap_geometry();
          let len = g.len();
          ( g.into_iter().flatten().collect(), ind.into_iter().collect(), len )
        },
        Self::Butt => 
        {
          ( Vec::new(), Vec::new(), 0 )
        }
      }
    }
  }

  pub fn round_cap_geometry( segments : usize ) -> ( Vec< [ f32; 2 ] >, Vec< [ u32; 3 ] > )
  {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    positions.push( [ 0.0; 2 ] );
    for i in 0..( segments + 1 )
    {
      let theta = std::f32::consts::PI * 0.5 + i as f32 / segments as f32 * std::f32::consts::PI;
      let ( y, x ) = theta.sin_cos();
      positions.push( [ 0.5 * x, 0.5 * y ] );
    }

    for i in 0..segments
    {
      let i = i as u32;
      indices.push( [ 0, i + 1, i + 2 ] );
    }

    ( positions, indices )
  }

  pub fn square_cap_geometry() -> ([ [ f32; 2 ]; 4 ] , [ u32; 6 ] )
  {
    let positions = 
    [
      [ 0.0, 0.5 ],
      [ 0.0, -0.5 ],
      [ 0.5, -0.5 ],
      [ 0.5, 0.5 ]
    ];

    let indices = 
    [
      0, 1, 2,
      0, 2, 3
    ];

    ( positions, indices )
  }

}

crate::mod_interface!
{  
  exposed use
  {
    Cap
  };
}