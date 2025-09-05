mod private
{
  use minwebgl as gl;

  /// Geometry for a rectangular body segment, represented by two triangles.
  pub const BODY_GEOMETRY : [ [ f32; 2 ]; 6 ] =
  [
    [ 0.0, -0.5 ],
    [ 1.0, -0.5 ],
    [ 1.0,  0.5 ],
    [ 0.0, -0.5 ],
    [ 1.0,  0.5 ],
    [ 0.0,  0.5 ]
  ];

  /// Geometry for a simple miter, consisting of 2 triangles
  pub const MITER_JOIN_GEOMETRY : [ [ f32; 3 ]; 6 ] = 
  [
    [ 0.0, 0.0, 0.0 ],
    [ 1.0, 0.0, 0.0 ],
    [ 0.0, 1.0, 0.0 ],
    [ 0.0, 0.0, 0.0 ],
    [ 0.0, 1.0, 0.0 ],
    [ 0.0, 0.0, 1.0 ]
  ];
  
  /// Geometry for a simple bevel, consiting of 1 triangle
  pub const BEVEL_JOIN_GEOMETRY : [ [ f32; 2 ]; 3 ] = 
  [
    [ 0.0, 0.0 ],
    [ 1.0, 0.0 ],
    [ 0.0, 1.0 ],
  ];
  

  /// Generates the geometry for a circle using a `TRIANGLE_FAN` draw mode.
  pub fn circle_geometry( segments : usize ) -> Vec< [ f32; 2 ] >
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

   /// Generates the geometry for the left half of a circle using `TRIANGLES` draw mode.
  pub fn circle_left_half_geometry( segments : usize ) -> Vec< [ f32; 2 ] >
  {
    let mut positions = Vec::with_capacity( segments * 3 );
    for wedge in 0..segments
    {
      let theta1  = std::f32::consts::PI / 2.0 + std::f32::consts::PI * wedge as f32 / segments as f32;
      let theta2  = std::f32::consts::PI / 2.0 + std::f32::consts::PI * ( wedge + 1 ) as f32 / segments as f32;
      let ( s1, c1 ) = theta1.sin_cos();
      let ( s2, c2 ) = theta2.sin_cos();
      positions.push( [ 0.0, 0.0 ] );
      positions.push( [ 0.5 * c1, 0.5 * s1 ] );
      positions.push( [ 0.5 * c2, 0.5 * s2 ] );
    }

    positions
  }

  /// Generates the geometry for the right half of a circle using `TRIANGLES` draw mode.
  pub fn circle_right_half_geometry( segments : usize ) -> Vec< [ f32; 2 ] >
  {
    let mut positions = Vec::with_capacity( segments * 3 );
    for wedge in 0..segments
    {
      let theta1  = 3.0 * std::f32::consts::PI / 2.0 + std::f32::consts::PI * wedge as f32 / segments as f32;
      let theta2  = 3.0 * std::f32::consts::PI / 2.0 + std::f32::consts::PI * ( wedge + 1 ) as f32 / segments as f32;
      let ( s1, c1 ) = theta1.sin_cos();
      let ( s2, c2 ) = theta2.sin_cos();
      positions.push( [ 0.0, 0.0 ] );
      positions.push( [ 0.5 * c1, 0.5 * s1 ] );
      positions.push( [ 0.5 * c2, 0.5 * s2 ] );
    }

    positions
  }

  /// Creates geometry for an arc that can be dynamically changed in the vertex shader
  pub fn simple_arc_geometry( segments : usize ) -> ( Vec< [ f32; 2 ] >, Vec< u32 > )
  {
    let segments = segments as u32;
    let ind = ( 0..=segments + 1 ).collect();
    let mut vertices = Vec::new();
    vertices.push( [ 0.0, 0.0 ] );

    for i in 1..=( segments + 1 )
    {
      vertices.push( [ ( i as f32 - 1.0 ) / segments as f32, 1.0 ] );
    }

    ( vertices, ind )
  }

  /// Generates the vertex data for a triangulated circular arc with the slight offset at the center of the arc, to ensure smooth looking uv transition. 
  /// The arc is meant to be built in the vertex shader.
  /// 
  /// Returns : ( vertices, uv in y direction )
  pub fn triangulated_arc_geometry( row_precision : usize, column_precision : usize ) -> ( Vec< gl::F32x2 >, Vec< f32 > ) 
  {
    let mut vertex_row_list = Vec::with_capacity( row_precision );
    let mut verticies = Vec::new();
    let mut uvs = Vec::new();

    let center_offset = 0.005;

    // Create vertices
    for i in 0..( row_precision + 1 )
    {
      let rm = ( 1.0 - ( i as f32 / row_precision as f32 ) ).max( center_offset );
      let mut column_list = Vec::with_capacity( column_precision );

      for k in 0..( column_precision + 1 )
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

  /// Generates the vertex data for a triangulated bevel with the slight offset at the base of the bevel, to ensure smooth looking uv transition. 
  /// The arc is meant to be built in the vertex shader.
  /// 
  /// Returns : ( vertices, uv in y direction )
  pub fn triangulated_bevel_geometry( row_precision : usize, column_precision : usize ) -> ( Vec< gl::F32x2 >, Vec< f32 > ) 
  {
    let mut vertex_row_list = Vec::with_capacity( row_precision );
    let mut verticies = Vec::new();
    let mut uvs = Vec::new();

    let p0 = gl::F32x2::new( 1.0, 0.0 );
    let p1 = gl::F32x2::new( 0.0, 1.0 );

    let center_offset = 0.005;

    // Create vertices
    for i in 0..( row_precision + 1 )
    {
      let rm = ( 1.0 - ( i as f32 / row_precision as f32 ) ).max( center_offset );
      let mut column_list = Vec::with_capacity( column_precision );
      let rp0 = p0 * rm;
      let rp1 = p1 * rm;

      for k in 0..( column_precision + 1 )
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

  /// Generates the vertex data for a triangulated miter with the slight offset at the base of the miter, to ensure smooth looking uv transition. 
  /// The arc is meant to be built in the vertex shader.
  /// 
  /// Returns : ( vertices, uv in y direction )
  pub fn triangulated_miter_geometry( row_precision : usize, column_precision : usize ) -> ( Vec< gl::F32x3 >, Vec< f32 > ) 
  {
    let mut vertex_row_list = Vec::with_capacity( row_precision );
    let mut verticies = Vec::new();
    let mut uvs = Vec::new();

    let p0 = gl::F32x3::new( 1.0, 0.0, 0.0 );
    let p1 = gl::F32x3::new( 0.0, 1.0, 0.0 );
    let p2 = gl::F32x3::new( 0.0, 0.0, 1.0 );

    let center_offset = 0.005;

    // Create vertices
    for i in 0..( row_precision + 1 )
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
      for k in 0..( column_precision + 1 )
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
  own use
  {
    BODY_GEOMETRY,
    BEVEL_JOIN_GEOMETRY,
    MITER_JOIN_GEOMETRY,

    circle_geometry,
    circle_left_half_geometry,
    circle_right_half_geometry,

    simple_arc_geometry,

    triangulated_arc_geometry,
    triangulated_bevel_geometry,
    triangulated_miter_geometry
  };
}