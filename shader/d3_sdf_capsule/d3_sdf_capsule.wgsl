//@ name: d3_sdf_capsule
//@ description: Signed distance from a 3D point to a capsule (swept sphere) between two endpoints of radius r.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_capsule(p: vec3f, a: vec3f, b: vec3f, r: f32) -> f32
//@ export: fn d3_sdf_capsule_preview(p: vec2f, z_slice: f32, a_x: f32, a_y: f32, a_z: f32, b_x: f32, b_y: f32, b_z: f32, radius: f32) -> f32
//@ param: z_slice argument f32 range(-0.3, 0.3)
//@ param: a_x argument f32 range(-0.3, 0.3)
//@ param: a_y argument f32 range(-0.3, 0.3)
//@ param: a_z argument f32 range(-0.3, 0.3)
//@ param: b_x argument f32 range(-0.3, 0.3)
//@ param: b_y argument f32 range(-0.3, 0.3)
//@ param: b_z argument f32 range(-0.3, 0.3)
//@ param: radius argument f32 range(0.02, 0.2)

fn d3_sdf_capsule( p : vec3f, a : vec3f, b : vec3f, r : f32 ) -> f32
{
  let pa = p - a;
  let ba = b - a;
  let h = clamp( dot( pa, ba ) / dot( ba, ba ), 0.0, 1.0 );
  return length( pa - ba * h ) - r;
}

fn d3_sdf_capsule_preview( p : vec2f, z_slice : f32, a_x : f32, a_y : f32, a_z : f32, b_x : f32, b_y : f32, b_z : f32, radius : f32 ) -> f32
{
  return d3_sdf_capsule( vec3f( p, z_slice ), vec3f( a_x, a_y, a_z ), vec3f( b_x, b_y, b_z ), radius );
}
