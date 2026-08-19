//! ## Root Cause
//! `plane_vao`'s 4-vertex `TRIANGLE_STRIP` data gives vertex 3 ( position x=1, z=-1 ) the
//! texcoord `( 1.0, 0.0 )` — an exact duplicate of vertex 2's row — instead of the
//! `( 1.0, 1.0 )` its own corner requires to complete the bilinear UV grid the other 3
//! vertices establish ( `uv.x = (1 - z) / 2`, `uv.y = (x + 1) / 2` ).
//!
//! ## Why Not Caught
//! `plane_material` currently fills both the base-color and ARM textures with a single
//! constant 1x1 texel, and `wrap_clamp`/`filter_nearest` make every UV sample that same
//! texel regardless of value — so the wrong UV produced no visible defect with today's
//! placeholder textures. The crate has no lib target or native test target to unit-test
//! `plane_vao`'s vertex data directly, only this structural source parse.
//!
//! ## Fix Applied (BUG-321)
//! Corrected vertex 3's texcoord from `( 1.0, 0.0 )` to `( 1.0, 1.0 )` in
//! `examples/minwebgl/area_light/src/plane.rs`'s `plane_vertices` array.
//!
//! ## Prevention
//! Parses the `plane_vertices` array out of `plane.rs`, extracts each vertex's
//! `( x, z, u, v )` (skipping the constant normal columns), and asserts every vertex's
//! UV matches the grid formula `u = (1 - z) / 2`, `v = (x + 1) / 2` — a general property
//! check that would catch a regression on any of the 4 vertices, not just vertex 3.
//!
//! ## Pitfall
//! Don't "fix" this by changing vertices 0/1/2 — they already satisfy the grid formula;
//! only vertex 3 was wrong.

const PLANE_RS : &str = include_str!( "../src/plane.rs" );

/// Parses the 4 `( x, y, z, nx, ny, nz, u, v )` rows out of `plane_vertices`'s array literal.
fn plane_vertex_rows() -> Vec< [ f32 ; 8 ] >
{
  let start = PLANE_RS.find( "let plane_vertices" ).expect( "plane_vertices not found in plane.rs" );
  // The `[`/`]` in the `&[ f32 ]` type annotation precede the array literal's own `[`, so
  // skip past the `=` assignment operator first before looking for the opening bracket —
  // otherwise this matches the type annotation's bracket instead of the value's.
  let assign = PLANE_RS[ start.. ].find( '=' ).map( | i | start + i ).expect( "assignment `=` not found after plane_vertices" );
  let array_start = PLANE_RS[ assign.. ].find( '[' ).map( | i | assign + i + 1 ).expect( "array open bracket not found" );
  let array_end = PLANE_RS[ array_start.. ].find( "];" ).map( | i | array_start + i ).expect( "array close bracket not found" );
  let body = &PLANE_RS[ array_start..array_end ];

  body.lines()
  .filter_map
  (
    | line |
    {
      let line = line.split( "//" ).next().unwrap_or( "" ).trim();
      if line.is_empty() { return None; }
      // Split on anything that isn't part of a float literal, so this survives either the
      // plain `x, y, z, ...` row format or a `Vertex { position : [ x, y, z ], ... }` struct
      // literal — field names and punctuation never contain digit/'.'/'-' characters.
      let nums : Vec< f32 > = line
      .split( | c : char | !( c.is_ascii_digit() || c == '.' || c == '-' ) )
      .filter( | s | !s.is_empty() )
      .map( | s | s.parse::< f32 >().unwrap_or_else( | e | panic!( "failed to parse {s:?}: {e}" ) ) )
      .collect();
      if nums.len() != 8 { return None; }
      Some( [ nums[ 0 ], nums[ 1 ], nums[ 2 ], nums[ 3 ], nums[ 4 ], nums[ 5 ], nums[ 6 ], nums[ 7 ] ] )
    }
  )
  .collect()
}

#[ test ]
fn plane_vertices_form_a_consistent_bilinear_uv_grid()
{
  let rows = plane_vertex_rows();
  assert_eq!( rows.len(), 4, "expected exactly 4 vertex rows, found {}: {PLANE_RS}", rows.len() );

  for ( i, row ) in rows.iter().enumerate()
  {
    let [ x, _y, z, .., u, v ] = *row;
    let expected_u = ( 1.0 - z ) / 2.0;
    let expected_v = f32::midpoint( x, 1.0 );
    assert!(
      ( u - expected_u ).abs() < 1e-6,
      "vertex {i}: u={u} does not match grid formula (1-z)/2={expected_u} for z={z}: {row:?}"
    );
    assert!(
      ( v - expected_v ).abs() < 1e-6,
      "vertex {i}: v={v} does not match grid formula (x+1)/2={expected_v} for x={x}: {row:?}"
    );
  }
}

#[ test ]
fn vertex_2_and_vertex_3_no_longer_share_a_texcoord()
{
  let rows = plane_vertex_rows();
  let v2_uv = ( rows[ 2 ][ 6 ], rows[ 2 ][ 7 ] );
  let v3_uv = ( rows[ 3 ][ 6 ], rows[ 3 ][ 7 ] );
  assert_ne!( v2_uv, v3_uv, "vertex 2 and vertex 3 must have distinct texcoords (different z but same x): {rows:?}" );
}
