mod private
{
  #[ cfg( feature = "serialization" ) ]
  use serde::{ Serialize, Deserialize };
  use minwebgl::{ self as gl, IntoArray };

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

  impl Join
  {
    /// Generates the geometry for the specified join type.
    ///
    /// This method returns a tuple containing the vertices, indices, uvs, and the number of
    /// elements for the join's mesh.
    #[must_use]
    pub fn geometry( &self ) -> ( Vec< f32 >, Vec< u32 >, Vec< f32 >, usize )
    {
      match self 
      {
        Self::Round( row_precision, column_precision ) => 
        {
          let ( g, uv ) = round_geometry( *row_precision, *column_precision );
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().flat_map(| v | v.as_array()).collect();
          let ind = Vec::new();
          ( g, ind, uv, len )
        },
        Self::Miter( row_precision, column_precision ) =>
        {
          let ( g, uv ) = miter_geometry( *row_precision, *column_precision );
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().flat_map(| v | v.as_array()).collect();
          let ind = Vec::new();
          ( g, ind, uv, len )
        },
        Self::Bevel( row_precision, column_precision ) => 
        {
          let ( g, uv ) = bevel_geometry( *row_precision, *column_precision );
          let len = g.len();
          let g : Vec< f32 > = g.into_iter().flat_map(| v | v.as_array()).collect();
          let ind = Vec::new();
          ( g, ind, uv, len )
        }
      }
    }
  }

  impl Default for Join 
  {
    fn default() -> Self 
    {
      Self::Round( 16, 8 )
    }    
  }

  /// Generates the vertex data for a round join.
  #[must_use]
  pub fn round_geometry( row_precision : usize, column_precision : usize ) -> ( Vec< gl::F32x2 >, Vec< f32 > )
  {
    // Fix(BUG-491): `row_precision`/`column_precision` are used both as loop bounds and as
    // division divisors (`i as f32 / row_precision as f32`, `k as f32 / column_precision as
    // f32`) with no floor. `column_precision == 0` computes a genuine NaN internally (unrescued
    // `0.0 / 0.0`); it never reached the returned geometry only because every read loop that
    // populates `verticies`/`uvs` is bounded by the exclusive range `0..column_precision`
    // (empty for `column_precision == 0`), so the observable defect was silently empty output,
    // not NaN -- see BUG-491's report for the full empirical trace.
    // Root cause: same missing-floor shape already fixed via `.max( 1 )` in
    // `caps.rs::round_cap_geometry` (BUG-236) and `helpers.rs::circle_geometry` (BUG-237), not
    // yet applied here.
    // Pitfall: a loop-bound coincidence that happens to prevent a downstream NaN from escaping
    // is not a substitute for flooring the value at its source -- it only changes the failure
    // mode (NaN vs. silently empty output) and breaks the moment an unrelated future edit
    // widens that read loop.
    let row_precision = row_precision.max( 1 );
    let column_precision = column_precision.max( 1 );

    let mut vertex_row_list = Vec::with_capacity( row_precision );
    let mut verticies = Vec::new();
    let mut uvs = Vec::new();

    let center_offset = 0.005;

    // Create vertices
    for i in 0..=row_precision
    {
      let rm = ( 1.0 - ( i as f32 / row_precision as f32 ) ).max( center_offset );
      let mut column_list = Vec::with_capacity( column_precision );

      for k in 0..=column_precision
      {
        let cm = k as f32 / column_precision as f32;
        column_list.push( gl::F32x2::new( cm, rm  ) );
      }

      vertex_row_list.push( column_list );
    }

    // Create triangles
    for i in 0..( vertex_row_list.len() - 1 )
    {
      let row1 = &vertex_row_list[ i ];
      let row2 = &vertex_row_list[ i + 1 ];

      // Left triangle
      for j in 0..column_precision
      {
        let c11 = row1[ j ];
        let c12 = row1[ j + 1 ];

        let c21 = row2[ j ];
        let c22 = row2[ j + 1 ];

        verticies.push( [ c11, c21, c22 ] );
        verticies.push( [ c11, c22, c12 ] );

        let uv1 = j as f32 / column_precision as f32;
        let uv2 = ( j + 1 ) as f32 / column_precision as f32;

        uvs.push( [ uv1, uv1, uv2 ] );
        uvs.push( [ uv1, uv2, uv2 ] );
      }
    }

    //// Create the last row of triangles
    let last_row = &vertex_row_list[ vertex_row_list.len() - 1 ];
    for j in 0..column_precision
    {
      let c11 = last_row[ j ];
      let c12 = last_row[ j + 1  ];

      verticies.push( [ c11, gl::F32x2::ZERO, c12 ] );

      let uv1 = j as f32 / column_precision as f32;
      let uv2 = ( j + 1 ) as f32 / column_precision as f32;

      uvs.push( [ uv1, 0.5, uv2 ] );
    }

    let verticies = verticies.into_iter().flatten().collect();
    let uvs = uvs.into_iter().flatten().collect();

    ( verticies, uvs )
  }

  /// Generates the vertex data for a bevel join.
  #[must_use]
  pub fn bevel_geometry( row_precision : usize, column_precision : usize ) -> ( Vec< gl::F32x2 >, Vec< f32 > )
  {
    // Fix(BUG-491): same missing floor as `round_geometry` above -- see that comment for the
    // full explanation.
    let row_precision = row_precision.max( 1 );
    let column_precision = column_precision.max( 1 );

    let mut vertex_row_list = Vec::with_capacity( row_precision );
    let mut verticies = Vec::new();
    let mut uvs = Vec::new();

    let p0 = gl::F32x2::new( 1.0, 0.0 );
    let p1 = gl::F32x2::new( 0.0, 1.0 );

    let center_offset = 0.005;

    // Create vertices
    for i in 0..=row_precision
    {
      let rm = ( 1.0 - ( i as f32 / row_precision as f32 ) ).max( center_offset );
      let mut column_list = Vec::with_capacity( column_precision );
      let rp0 = p0 * rm;
      let rp1 = p1 * rm;

      for k in 0..=column_precision
      {
        let cm = k as f32 / column_precision as f32;
        let p = rp0 * ( 1.0 - cm ) + rp1 * cm;
        column_list.push( p );
      }

      vertex_row_list.push( column_list );
    }

    // Create triangles
    for i in 0..( vertex_row_list.len() - 1 )
    {
      let row1 = &vertex_row_list[ i ];
      let row2 = &vertex_row_list[ i + 1 ];

      // Left triangle
      for j in 0..column_precision
      {
        let c11 = row1[ j ];
        let c12 = row1[ j + 1 ];

        let c21 = row2[ j ];
        let c22 = row2[ j + 1 ];

        verticies.push( [ c11, c21, c22 ] );
        verticies.push( [ c11, c22, c12 ] );

        let uv1 = j as f32 / column_precision as f32;
        let uv2 = ( j + 1 ) as f32 / column_precision as f32;

        uvs.push( [ uv1, uv1, uv2 ] );
        uvs.push( [ uv1, uv2, uv2 ] );
      }
    }

    //// Create the last row of triangles
    let last_row = &vertex_row_list[ vertex_row_list.len() - 1 ];
    for j in 0..column_precision
    {
      let c11 = last_row[ j ];
      let c12 = last_row[ j + 1  ];

      verticies.push( [ c11, gl::F32x2::ZERO, c12 ] );

      let uv1 = j as f32 / column_precision as f32;
      let uv2 = ( j + 1 ) as f32 / column_precision as f32;

      uvs.push( [ uv1, 0.5, uv2 ] );
    }

    let verticies = verticies.into_iter().flatten().collect();
    let uvs = uvs.into_iter().flatten().collect();

    ( verticies, uvs )
  }

  /// Generates the vertex data for a miter join.
  #[must_use]
  pub fn miter_geometry( row_precision : usize, column_precision : usize ) -> ( Vec< gl::F32x3 >, Vec< f32 > )
  {
    // Fix(BUG-491): same missing floor as `round_geometry` above -- see that comment for the
    // full explanation.
    let row_precision = row_precision.max( 1 );
    let column_precision = column_precision.max( 1 );

    let mut vertex_row_list = Vec::with_capacity( row_precision );
    let mut verticies = Vec::new();
    let mut uvs = Vec::new();

    let p0 = gl::F32x3::new( 1.0, 0.0, 0.0 );
    let p1 = gl::F32x3::new( 0.0, 1.0, 0.0 );
    let p2 = gl::F32x3::new( 0.0, 0.0, 1.0 );

    let center_offset = 0.005;

    // Create vertices
    for i in 0..=row_precision
    {
      let rm = ( 1.0 - ( i as f32 / row_precision as f32 ) ).max( center_offset );
      let mut column_list = Vec::with_capacity( column_precision );
      let rp0 = p0 * rm;
      let rp1 = p1 * rm;
      let rp2 = p2 * rm;

      // Left triangle
      for k in 0..column_precision
      {
        let cm = k as f32 / column_precision as f32;
        let p = rp0 * ( 1.0 - cm ) + rp1 * cm;
        column_list.push( p );
      }

      // Right triangle
      for k in 0..=column_precision
      {
        let cm = k as f32 / column_precision as f32;
        let p = rp1 * ( 1.0 - cm ) + rp2 * cm;
        column_list.push( p );
      }

      vertex_row_list.push( column_list );
    }


    // Create triangles
    for i in 0..( vertex_row_list.len() - 1 )
    {
      let row1 = &vertex_row_list[ i ];
      let row2 = &vertex_row_list[ i + 1 ];

      // Left triangle
      for j in 0..column_precision
      {
        let c11 = row1[ j ];
        let c12 = row1[ j + 1 ];

        let c21 = row2[ j ];
        let c22 = row2[ j + 1 ];

        verticies.push( [ c11, c21, c22 ] );
        verticies.push( [ c11, c22, c12 ] );

        let uv1 = 0.5 * j as f32 / column_precision as f32;
        let uv2 = 0.5 * ( j + 1 ) as f32 / column_precision as f32;

        uvs.push( [ uv1, uv1, uv2 ] );
        uvs.push( [ uv1, uv2, uv2 ] );
      }

      // Right triangle
      for j in 0..column_precision
      {
        let j_old = j;
        let j = j + column_precision;

        let c11 = row1[ j ];
        let c12 = row1[ j + 1 ];

        let c21 = row2[ j ];
        let c22 = row2[ j + 1 ];

        verticies.push( [ c11, c21, c22 ] );
        verticies.push( [ c11, c22, c12 ] );

        let uv1 = 0.5 + 0.5 * j_old as f32 / column_precision as f32;
        let uv2 = 0.5 + 0.5 * ( j_old + 1 ) as f32 / column_precision as f32;

        uvs.push( [ uv1, uv1, uv2 ] );
        uvs.push( [ uv1, uv2, uv2 ] );
      }

    }

    //// Create the last row of triangles
    let last_row = &vertex_row_list[ vertex_row_list.len() - 1 ];
    //Left triangle
    for j in 0..column_precision
    {
      let c11 = last_row[ j ];
      let c12 = last_row[ j + 1 ];

      verticies.push( [ c11, gl::F32x3::ZERO, c12 ] );

      let uv1 = 0.5 * j as f32 / column_precision as f32;
      let uv2 = 0.5 * ( j + 1 ) as f32 / column_precision as f32;

      uvs.push( [ uv1, 0.5, uv2 ] );
    }

    // Right triangle
    for j in 0..column_precision
    {
      let j_old = j;
      let j = j + column_precision;

      let c11 = last_row[ j ];
      let c12 = last_row[ j + 1 ];

      verticies.push( [ c11, gl::F32x3::ZERO, c12 ] );

      let uv1 = 0.5 + 0.5 * j_old as f32 / column_precision as f32;
      let uv2 = 0.5 + 0.5 * ( j_old + 1 ) as f32 / column_precision as f32;

      uvs.push( [ uv1, 0.5, uv2 ] );
    }


    let verticies = verticies.into_iter().flatten().collect();
    let uvs = uvs.into_iter().flatten().collect();

    ( verticies, uvs )
  }

}

crate::mod_interface!
{
  own use crate::helpers::circle_geometry;

  own use
  {
    miter_geometry,
    bevel_geometry,
    round_geometry
  };

  exposed use
  {
    Join
  };

}