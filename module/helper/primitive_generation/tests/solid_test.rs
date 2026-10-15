//! Integration tests for `primitive_generation`'s procedural solid-mesh
//! generators (`box_mesh`, `cylinder_mesh`, `torus_mesh`, `icosphere`,
//! `src/solid.rs`) -- previously untested despite being `pub fn`. Each test
//! asserts a geometric invariant the generator's own doc comment claims
//! (vertex/index counts, ring radius and height placement, torus surface
//! distance, unit-sphere regularity) rather than pinning exact coordinate
//! values, so the tests stay valid across any future internal
//! re-parametrization that preserves the documented shape.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::{ box_mesh, cylinder_mesh, torus_mesh, icosphere };

  #[ test ]
  fn box_mesh_has_eight_corners_matching_half_extents()
  {
    let ( hx, hy, hz ) = ( 2.0_f32, 3.0_f32, 4.0_f32 );
    let ( positions, indices ) = box_mesh( hx, hy, hz );

    assert_eq!( positions.len(), 8, "a box has exactly 8 corner vertices" );
    assert_eq!( indices.len(), 36, "6 faces * 2 triangles * 3 indices" );

    for &[ x, y, z ] in &positions
    {
      assert!( ( x.abs() - hx ).abs() < 1e-6, "every corner's x must be ±hx; got {x}" );
      assert!( ( y.abs() - hy ).abs() < 1e-6, "every corner's y must be ±hy; got {y}" );
      assert!( ( z.abs() - hz ).abs() < 1e-6, "every corner's z must be ±hz; got {z}" );
    }
    for &i in &indices
    {
      assert!( ( i as usize ) < positions.len(), "index {i} out of range for {} positions", positions.len() );
    }
  }

  #[ test ]
  fn cylinder_mesh_rings_match_requested_radii_and_height()
  {
    let ( radius_top, radius_bottom, height, segments ) = ( 1.5_f32, 2.5_f32, 4.0_f32, 8_usize );
    let ( positions, _ ) = cylinder_mesh( radius_top, radius_bottom, height, segments );
    let half = height * 0.5;

    for &[ x, y, z ] in &positions[ .. segments ]
    {
      assert!( ( y - half ).abs() < 1e-5, "top ring must sit at y = height/2; got {y}" );
      assert!( ( x.hypot( z ) - radius_top ).abs() < 1e-5, "top ring must sit at radius_top; got {}", x.hypot( z ) );
    }
    for &[ x, y, z ] in &positions[ segments .. segments * 2 ]
    {
      assert!( ( y - ( -half ) ).abs() < 1e-5, "bottom ring must sit at y = -height/2; got {y}" );
      assert!( ( x.hypot( z ) - radius_bottom ).abs() < 1e-5, "bottom ring must sit at radius_bottom; got {}", x.hypot( z ) );
    }
  }

  #[ test ]
  fn cylinder_mesh_full_cylinder_has_two_caps()
  {
    let segments = 8;
    let ( positions, indices ) = cylinder_mesh( 1.0, 1.0, 2.0, segments );

    assert_eq!( positions.len(), segments * 2 + 2, "two rings plus two cap centers" );
    assert_eq!( indices.len(), segments * 12, "side quads (6/segment) + two cap fans (3/segment each)" );
    for &i in &indices
    {
      assert!( ( i as usize ) < positions.len(), "index {i} out of range for {} positions", positions.len() );
    }
  }

  #[ test ]
  fn cylinder_mesh_zero_top_radius_yields_cone_with_one_cap()
  {
    let segments = 8;
    let ( positions, indices ) = cylinder_mesh( 0.0, 1.0, 2.0, segments );

    assert_eq!( positions.len(), segments * 2 + 1, "collapsed top ring + bottom ring + one bottom cap center" );
    assert_eq!( indices.len(), segments * 9, "side quads (6/segment) + one cap fan (3/segment)" );
    for &[ x, y, z ] in &positions[ .. segments ]
    {
      assert!
      (
        x.abs() < 1e-6 && z.abs() < 1e-6 && ( y - 1.0 ).abs() < 1e-6,
        "a zero-radius top ring collapses every vertex onto the apex (0, height/2, 0); got [{x}, {y}, {z}]"
      );
    }
  }

  #[ test ]
  fn torus_mesh_vertices_lie_on_the_torus_surface()
  {
    let ( radius, tube, radial_segments, tubular_segments ) = ( 3.0_f32, 0.7_f32, 6_usize, 10_usize );
    let ( positions, indices ) = torus_mesh( radius, tube, radial_segments, tubular_segments );

    assert_eq!( positions.len(), radial_segments * tubular_segments );
    assert_eq!( indices.len(), radial_segments * tubular_segments * 6 );

    for &[ x, y, z ] in &positions
    {
      // Distance from the ring circle of radius `radius` in the XY plane
      // must equal `tube` for every vertex to genuinely lie on the torus
      // surface, not merely within its bounding radii.
      let ring_dist = x.hypot( y ) - radius;
      let surface_dist = ( ring_dist * ring_dist + z * z ).sqrt();
      assert!
      (
        ( surface_dist - tube ).abs() < 1e-4,
        "vertex [{x}, {y}, {z}] is {surface_dist} from the ring circle, expected tube radius {tube}"
      );
    }
    for &i in &indices
    {
      assert!( ( i as usize ) < positions.len(), "index {i} out of range for {} positions", positions.len() );
    }
  }

  #[ test ]
  fn icosphere_is_a_regular_unit_icosahedron()
  {
    let ( positions, indices ) = icosphere();

    assert_eq!( positions.len(), 12, "an icosahedron has exactly 12 vertices" );
    assert_eq!( indices.len(), 60, "20 triangular faces * 3 indices" );

    for &[ x, y, z ] in &positions
    {
      let magnitude = ( x * x + y * y + z * z ).sqrt();
      assert!( ( magnitude - 1.0 ).abs() < 1e-5, "every vertex must lie on the unit sphere; got magnitude {magnitude}" );
    }

    let mut edge_lengths = Vec::new();
    for tri in indices.chunks_exact( 3 )
    {
      let ( ia, ib, ic ) = ( tri[ 0 ] as usize, tri[ 1 ] as usize, tri[ 2 ] as usize );
      assert!
      (
        ia < positions.len() && ib < positions.len() && ic < positions.len(),
        "face index out of range: {tri:?}"
      );
      for &( p, q ) in &[ ( ia, ib ), ( ib, ic ), ( ic, ia ) ]
      {
        let [ px, py, pz ] = positions[ p ];
        let [ qx, qy, qz ] = positions[ q ];
        edge_lengths.push( ( ( px - qx ).powi( 2 ) + ( py - qy ).powi( 2 ) + ( pz - qz ).powi( 2 ) ).sqrt() );
      }
    }

    let first = edge_lengths[ 0 ];
    for len in &edge_lengths
    {
      assert!
      (
        ( len - first ).abs() < 1e-4,
        "a regular icosahedron has exactly one edge length; got {len} vs first edge {first}"
      );
    }
  }

  /// Regression guard for BUG-396: `box_mesh` and `cylinder_mesh` are both star-shaped from the
  /// origin (their own documented center), so an outward-facing triangle's face normal must point
  /// in the same general direction as that triangle's own centroid -- checked via
  /// `dot( cross( edge1, edge2 ), centroid ) > 0` rather than pinning exact normal values, so the
  /// test stays valid across any future re-tessellation that preserves outward winding.
  fn assert_triangle_faces_outward( positions : &[ [ f32; 3 ] ], tri : &[ u32 ], label : &str )
  {
    let ( ia, ib, ic ) = ( tri[ 0 ] as usize, tri[ 1 ] as usize, tri[ 2 ] as usize );
    let [ ax, ay, az ] = positions[ ia ];
    let [ bx, by, bz ] = positions[ ib ];
    let [ cx, cy, cz ] = positions[ ic ];

    let e1 = [ bx - ax, by - ay, bz - az ];
    let e2 = [ cx - ax, cy - ay, cz - az ];
    let normal =
    [
      e1[ 1 ] * e2[ 2 ] - e1[ 2 ] * e2[ 1 ],
      e1[ 2 ] * e2[ 0 ] - e1[ 0 ] * e2[ 2 ],
      e1[ 0 ] * e2[ 1 ] - e1[ 1 ] * e2[ 0 ],
    ];
    let centroid = [ ( ax + bx + cx ) / 3.0, ( ay + by + cy ) / 3.0, ( az + bz + cz ) / 3.0 ];
    let dot = normal[ 0 ] * centroid[ 0 ] + normal[ 1 ] * centroid[ 1 ] + normal[ 2 ] * centroid[ 2 ];

    assert!
    (
      dot > 0.0,
      "{label}: triangle {tri:?} winds inward ( cross(edge1,edge2) = {normal:?} vs centroid \
      {centroid:?}, dot = {dot} ) -- both shapes are centered at and star-shaped from the origin, \
      so an outward-facing normal must point in the same general direction as its own triangle's \
      centroid"
    );
  }

  #[ test ]
  fn box_mesh_triangles_wind_outward()
  {
    let ( positions, indices ) = box_mesh( 2.0, 3.0, 4.0 );
    for ( i, tri ) in indices.chunks_exact( 3 ).enumerate()
    {
      assert_triangle_faces_outward( &positions, tri, &format!( "box_mesh triangle {i}" ) );
    }
  }

  #[ test ]
  fn cylinder_mesh_triangles_wind_outward()
  {
    let ( positions, indices ) = cylinder_mesh( 1.0, 1.0, 2.0, 8 );
    for ( i, tri ) in indices.chunks_exact( 3 ).enumerate()
    {
      assert_triangle_faces_outward( &positions, tri, &format!( "cylinder_mesh triangle {i}" ) );
    }
  }
}
