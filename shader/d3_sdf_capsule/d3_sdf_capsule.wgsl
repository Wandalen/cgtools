//@ name: d3_sdf_capsule
//@ description: Signed distance from a 3D point to a capsule (swept sphere) between two endpoints of radius r.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_capsule(p: vec3f, a: vec3f, b: vec3f, r: f32) -> f32
//@ export: fn d3_sdf_capsule_preview(p: vec2f) -> f32

fn d3_sdf_capsule( p : vec3f, a : vec3f, b : vec3f, r : f32 ) -> f32
{
  let pa = p - a;
  let ba = b - a;
  let h = clamp( dot( pa, ba ) / dot( ba, ba ), 0.0, 1.0 );
  return length( pa - ba * h ) - r;
}

fn d3_sdf_capsule_preview( p : vec2f ) -> f32
{
  return d3_sdf_capsule( vec3f( p, 0.0 ), vec3f( -0.15, -0.12, 0.0 ), vec3f( 0.15, 0.12, 0.0 ), 0.09 );
}
