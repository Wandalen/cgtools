mod private
{
  #[ cfg( feature = "serialization" ) ]
  use serde::{ Serialize, Deserialize };
  use minwebgl::{self as gl, IntoArray};

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
    /// This method returns a tuple containing the vertices, indices, uvs, and the number of
    /// elements for the join's mesh.
    pub fn geometry( &self ) -> ( Vec< f32 >, Vec< u32 >, Vec< f32 >, usize )
    {
      match self 
      {
        Self::Round( segments ) => 
        {
          let ( g, ind ) = round_geometry( *segments );
          let uv = Vec::new();
          let len = g.len();
          ( 
            g.into_iter().map( | v | v as f32 ).collect(), 
            ind.into_iter().flatten().collect(), 
            uv,
            len 
          )
        },
        Self::Miter =>
        {
          let ( g, uv ) = miter_geometry_dynamic( 15 );
          let g : Vec< f32 > = g.into_iter().map( | v | v.as_array() ).flatten().collect();
          let ind = Vec::new();
          let len = g.len();
          ( g, ind, uv, len )
        },
        Self::Bevel => 
        {
          let g = bevel_geometry();
          let g : Vec< f32 > = g.into_iter().flatten().collect();
          let ind = Vec::new();
          let uv = Vec::new();
          let len = g.len();
          ( g, ind, uv, len )
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

  pub fn miter_geometry_dynamic( precision : u32 ) -> ( Vec< gl::F32x4 >, Vec< f32 > ) 
  {
    let mut vertex_row_0 = Vec::new();
    let mut vertex_row_1 = Vec::new();
    let mut vertex_row_2 = Vec::new();
    let mut verticies = Vec::new();
    let mut uvs = Vec::new();
    //let mut indices = Vec::new();

    let p0 = gl::F32x4::new( 1.0, 0.0, 0.0, 0.0 );
    let p1 = gl::F32x4::new( 0.0, 1.0, 0.0, 0.0 );
    let p2 = gl::F32x4::new( 0.0, 0.0, 1.0, 0.0 );
    let p3 = gl::F32x4::new( 0.0, 0.0, 0.0, 1.0 );

    let total_vertices = 4 + 3 * ( precision - 1 );
    let shift = 1.0 / precision as f32;
    let mut k = 1.0;
    for i in 0..precision
    {
      vertex_row_0.push( p0 * k );
      vertex_row_1.push( p1 * k );
      vertex_row_2.push( p2 * k );

      k -= shift;
    }

    for i in 0..precision
    {
      let i = i as usize;
      let p01 = vertex_row_0[ i ];
      let p11 = vertex_row_1[ i ];
      let p21 = vertex_row_2[ i ];

      let p12 = if i == precision as usize - 1 { p3 } else { vertex_row_1[ i + 1 ] };

      verticies.push( [ p01, p12, p11 ] );
      verticies.push( [ p21, p11, p12 ] );

      uvs.push( [ 0.0, 1.0, 1.0 ] );
      uvs.push( [ 2.0, 1.0, 1.0 ] );

      if i != precision as usize - 1
      {
        let p02 = vertex_row_0[ i + 1 ];
        let p22 = vertex_row_2[ i + 1 ];

        verticies.push( [ p01, p02, p12 ] );
        verticies.push( [ p21, p12, p22 ] );

        uvs.push( [ 0.0, 0.0, 1.0 ] );
        uvs.push( [ 2.0, 1.0, 2.0 ] );
      }
    }

    gl::info!( "{:?}", verticies );

    let verticies = verticies.into_iter().flatten().collect();
    let uvs = uvs.into_iter().flatten().collect();

    ( verticies, uvs )
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