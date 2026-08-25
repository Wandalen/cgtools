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
  #[must_use]
  pub fn circle_geometry( segments : usize ) -> Vec< [ f32; 2 ] >
  {
    // Fix(BUG-237): `segments == 0` made the loop below divide by zero (`wedge as f32 / segments as f32`
    // with `segments = 0`), pushing a NaN vertex into the returned geometry instead of erroring.
    // Root cause: no floor on `segments` before it's used as a division's divisor -- same shape
    // as BUG-236's `round_cap_geometry`, found in the same scouting pass.
    // Pitfall: `f32` division never panics on a zero divisor -- it silently returns NaN -- so this
    // has to be guarded at the entry point, mirroring `Tween::new`/`Step::new`'s established
    // `.max( .. )` convention for the identical defect shape (BUG-142/BUG-233).
    let segments = segments.max( 1 );
    let mut positions = Vec::with_capacity( segments );
    for wedge in 0..=segments
    {
      let theta = 2.0 * std::f32::consts::PI * wedge as f32 / segments as f32;
      let ( s, c ) = theta.sin_cos();
      positions.push( [ 0.5 * c, 0.5 * s ] );
    }

    positions
  }

   /// Generates the geometry for the left half of a circle using `TRIANGLES` draw mode.
  #[must_use]
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
  #[must_use]
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

  /// Return positions and uvs for a rectangle, used in 3d line
  #[must_use]
  pub fn four_piece_rectangle_geometry() -> ( [ [ f32; 2 ]; 8 ], [ u32; 18 ],  [ [ f32; 2 ]; 8 ] )
  {
    let positions = 
    [
      [ -1.0, 2.0 ], [ 1.0, 2.0 ],
      [ -1.0, 1.0 ], [ 1.0, 1.0 ],
      [ -1.0, 0.0 ], [ 1.0, 0.0 ],
      [ -1.0, -1.0 ], [ 1.0, -1.0 ],
    ];

    let uvs = 
    [
      [ -1.0, 2.0 ], [ 1.0, 2.0 ],
      [ -1.0, 1.0 ], [ 1.0, 1.0 ],
      [ -1.0, -1.0 ], [ 1.0, -1.0 ],
      [ -1.0, -2.0 ], [ 1.0, -2.0 ],
    ];

    let indices = 
    [
      0, 2, 1, 
      2, 3, 1, 
      2, 4, 3, 
      4, 5, 3, 
      4, 6, 5, 
      6, 7, 5
    ];

    ( positions, indices, uvs )
  }
}

crate::mod_interface!
{
  own use
  {
    circle_geometry,
    BODY_GEOMETRY,
    circle_left_half_geometry,
    circle_right_half_geometry,

    four_piece_rectangle_geometry
  };
}