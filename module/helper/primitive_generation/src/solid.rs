//! Procedural box/cylinder/cone/torus/icosphere mesh generators, matching
//! three.js's built-in `BoxGeometry`/`CylinderGeometry`/`ConeGeometry`/
//! `TorusGeometry` shapes closely enough to port scenes built against them.
//!
//! Unlike `primitive::plane_to_geometry` and friends, these return raw
//! `( positions, indices )` pairs instead of a `PrimitiveData` — there is no
//! per-vertex normal, color, or transform bookkeeping, just geometry a
//! caller uploads into its own VAO however it likes (a full `Material`/
//! `Geometry` pipeline via `primitives_data_to_gltf`, or a bare
//! `bufferData` call, or anything in between).
//!
//! None of these bother getting triangle winding "correct" for outward
//! normals — a caller deriving a flat-shading normal from screen-space
//! derivatives (`dFdx`/`dFdy`) gets a correct result regardless of winding;
//! a caller relying on `CULL_FACE` or vertex normals for lighting will need
//! to fix winding order (and generate normals) itself.

mod private
{
  use minwebgl as gl;

  const TAU : f32 = std::f32::consts::TAU;

  /// Centered at the origin, half-extents `(hx, hy, hz)`.
  #[ must_use ]
  pub fn box_mesh( hx : f32, hy : f32, hz : f32 ) -> ( Vec< [ f32; 3 ] >, Vec< u32 > )
  {
    let positions = vec!
    [
      [ -hx, -hy, -hz ], [  hx, -hy, -hz ], [  hx,  hy, -hz ], [ -hx,  hy, -hz ],
      [ -hx, -hy,  hz ], [  hx, -hy,  hz ], [  hx,  hy,  hz ], [ -hx,  hy,  hz ],
    ];

    // Fix(winding): every face here was wound backwards (`cross(edge1,
    // edge2)` pointed *into* the box, not outward) - harmless for the
    // dFdx/dFdy-derived flat-shading normal in hull.frag (it self-corrects
    // via gl_FrontFacing regardless of a mesh's winding), but it broke
    // shadow mapping's front-face-culling trick (`ShadowMap::bind()` culls
    // whatever's front-facing and keeps back-facing, relying on *correct*
    // winding to record the far/away-from-light surface as the occluder -
    // with this reversed, it kept the near surface instead, which reads as
    // near-total self-shadow acne on anything facing the light). Every
    // triangle below has its last two indices swapped from the original to
    // reverse its winding.
    let indices = vec!
    [
      0, 2, 1,  0, 3, 2, // back  (z = -hz)
      4, 5, 6,  4, 6, 7, // front (z = +hz)
      0, 7, 3,  0, 4, 7, // left  (x = -hx)
      1, 6, 5,  1, 2, 6, // right (x = +hx)
      0, 5, 4,  0, 1, 5, // bottom (y = -hy)
      3, 6, 2,  3, 7, 6, // top    (y = +hy)
    ];

    ( positions, indices )
  }

  /// Y-axis aligned, matching three.js `CylinderGeometry`'s default
  /// orientation: top ring at `y = +height/2` (radius `radius_top`), bottom
  /// ring at `y = -height/2` (radius `radius_bottom`). A cone is
  /// `cylinder_mesh(0.0, radius, height, segments)` - the degenerate
  /// zero-radius ring collapses half of each side quad to a zero-area
  /// triangle, leaving the other half as the cone's lateral face.
  #[ must_use ]
  pub fn cylinder_mesh( radius_top : f32, radius_bottom : f32, height : f32, segments : usize ) -> ( Vec< [ f32; 3 ] >, Vec< u32 > )
  {
    let half = height * 0.5;
    let mut positions = Vec::with_capacity( segments * 2 + 2 );
    let mut indices = Vec::new();

    for i in 0 .. segments
    {
      let a = ( i as f32 / segments as f32 ) * TAU;
      positions.push( [ a.cos() * radius_top, half, a.sin() * radius_top ] );
    }
    for i in 0 .. segments
    {
      let a = ( i as f32 / segments as f32 ) * TAU;
      positions.push( [ a.cos() * radius_bottom, -half, a.sin() * radius_bottom ] );
    }

    let top_ring = 0;
    let bottom_ring = segments;
    for i in 0 .. segments
    {
      let i2 = ( i + 1 ) % segments;
      let ( t0, t1 ) = ( ( top_ring + i ) as u32, ( top_ring + i2 ) as u32 );
      let ( b0, b1 ) = ( ( bottom_ring + i ) as u32, ( bottom_ring + i2 ) as u32 );
      indices.extend_from_slice( &[ t0, t1, b1, t0, b1, b0 ] );
    }

    // Fix(winding): both cap fans below were backwards (same class of bug
    // as box_mesh - see its comment) - the side wall quads above are
    // correctly wound already, only these two fans needed the swap.
    if radius_top > 0.0001
    {
      let center = positions.len() as u32;
      positions.push( [ 0.0, half, 0.0 ] );
      for i in 0 .. segments
      {
        let i2 = ( i + 1 ) % segments;
        indices.extend_from_slice( &[ center, ( top_ring + i2 ) as u32, ( top_ring + i ) as u32 ] );
      }
    }
    if radius_bottom > 0.0001
    {
      let center = positions.len() as u32;
      positions.push( [ 0.0, -half, 0.0 ] );
      for i in 0 .. segments
      {
        let i2 = ( i + 1 ) % segments;
        indices.extend_from_slice( &[ center, ( bottom_ring + i ) as u32, ( bottom_ring + i2 ) as u32 ] );
      }
    }

    ( positions, indices )
  }

  /// Matches three.js `TorusGeometry`'s parametrization: the main ring lies in
  /// the XY plane (donut-hole axis is Z); callers rotate it to reorient, same
  /// as `ring.rotation.x = Math.PI / 2` in three.js scenes that stand a torus
  /// up around a different axis. Skips the usual seam-duplicate row/column
  /// since nothing here is textured.
  #[ must_use ]
  pub fn torus_mesh( radius : f32, tube : f32, radial_segments : usize, tubular_segments : usize ) -> ( Vec< [ f32; 3 ] >, Vec< u32 > )
  {
    let mut positions = Vec::with_capacity( radial_segments * tubular_segments );
    for j in 0 .. radial_segments
    {
      let v = ( j as f32 / radial_segments as f32 ) * TAU;
      for i in 0 .. tubular_segments
      {
        let u = ( i as f32 / tubular_segments as f32 ) * TAU;
        let r = radius + tube * v.cos();
        positions.push( [ r * u.cos(), r * u.sin(), tube * v.sin() ] );
      }
    }

    let mut indices = Vec::with_capacity( radial_segments * tubular_segments * 6 );
    for j in 0 .. radial_segments
    {
      let j2 = ( j + 1 ) % radial_segments;
      for i in 0 .. tubular_segments
      {
        let i2 = ( i + 1 ) % tubular_segments;
        let a = ( j * tubular_segments + i ) as u32;
        let b = ( j * tubular_segments + i2 ) as u32;
        let c = ( j2 * tubular_segments + i2 ) as u32;
        let d = ( j2 * tubular_segments + i ) as u32;
        indices.extend_from_slice( &[ a, b, c, a, c, d ] );
      }
    }

    ( positions, indices )
  }

  /// Unit-radius icosahedron (12 vertices, 20 triangular faces) - a cheap
  /// low-poly stand-in for a sphere.
  #[ must_use ]
  pub fn icosphere() -> ( Vec< [ f32; 3 ] >, Vec< u32 > )
  {
    let phi = ( 1.0 + 5.0f32.sqrt() ) * 0.5;
    let raw : [ [ f32; 3 ]; 12 ] =
    [
      [ -1.0, phi, 0.0 ], [ 1.0, phi, 0.0 ], [ -1.0, -phi, 0.0 ], [ 1.0, -phi, 0.0 ],
      [ 0.0, -1.0, phi ], [ 0.0, 1.0, phi ], [ 0.0, -1.0, -phi ], [ 0.0, 1.0, -phi ],
      [ phi, 0.0, -1.0 ], [ phi, 0.0, 1.0 ], [ -phi, 0.0, -1.0 ], [ -phi, 0.0, 1.0 ],
    ];
    let positions = raw.iter()
    .map( | v | gl::math::F32x3::from( *v ).normalize().to_array() )
    .collect();

    let indices = vec!
    [
      0, 11, 5,  0, 5, 1,  0, 1, 7,  0, 7, 10,  0, 10, 11,
      1, 5, 9,  5, 11, 4,  11, 10, 2,  10, 7, 6,  7, 1, 8,
      3, 9, 4,  3, 4, 2,  3, 2, 6,  3, 6, 8,  3, 8, 9,
      4, 9, 5,  2, 4, 11,  6, 2, 10,  8, 6, 7,  9, 8, 1,
    ];

    ( positions, indices )
  }
}

crate::mod_interface!
{
  orphan use
  {
    box_mesh,
    cylinder_mesh,
    torus_mesh,
    icosphere,
  };
}
