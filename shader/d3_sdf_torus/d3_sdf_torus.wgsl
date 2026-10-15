//@ name: d3_sdf_torus
//@ description: Signed distance from a 3D point to a torus with major and tube radii t, ring in the xz-plane.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_torus(p: vec3f, t: vec2f) -> f32
//@ export: fn d3_sdf_torus_preview(p: vec2f, major_radius: f32, tube_radius: f32, z_slice: f32) -> f32
//@ param: major_radius argument f32 range(0.1, 0.4)
//@ param: tube_radius argument f32 range(0.02, 0.15)
//@ param: z_slice argument f32 range(-0.3, 0.3)

fn d3_sdf_torus( p : vec3f, t : vec2f ) -> f32
{
  // t.x = major (ring) radius, t.y = tube radius. Ring lies in the xz-plane.
  let q = vec2f( length( p.xz ) - t.x, p.y );
  return length( q ) - t.y;
}

fn d3_sdf_torus_preview( p : vec2f, major_radius : f32, tube_radius : f32, z_slice : f32 ) -> f32
{
  return d3_sdf_torus( vec3f( p, z_slice ), vec2f( major_radius, tube_radius ) );
}
