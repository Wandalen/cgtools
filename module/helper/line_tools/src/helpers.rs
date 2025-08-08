mod private
{
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
}

crate::mod_interface!
{
  own use
  {
    circle_geometry,
    BODY_GEOMETRY,
    circle_left_half_geometry,
    circle_right_half_geometry
  };
}