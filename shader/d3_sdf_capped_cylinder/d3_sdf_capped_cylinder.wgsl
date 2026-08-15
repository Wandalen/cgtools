//@ name: d3_sdf_capped_cylinder
//@ description: Signed distance from a 3D point to a flat-capped cylinder of half-height h and radius r, axis along y.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_capped_cylinder(p: vec3f, h: f32, r: f32) -> f32
//@ export: fn d3_sdf_capped_cylinder_preview(p: vec2f) -> f32

fn d3_sdf_capped_cylinder( p : vec3f, h : f32, r : f32 ) -> f32
{
  let d = abs( vec2f( length( p.xz ), p.y ) ) - vec2f( r, h );
  return min( max( d.x, d.y ), 0.0 ) + length( max( d, vec2f( 0.0 ) ) );
}

fn d3_sdf_capped_cylinder_preview( p : vec2f ) -> f32
{
  return d3_sdf_capped_cylinder( vec3f( p, 0.0 ), 0.22, 0.16 );
}
