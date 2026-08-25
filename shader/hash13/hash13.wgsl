//@ name: hash13
//@ description: Single-value hash of a 3D point into [0, 1).
//@ tags: category:hash
//@ depends_on:
//@ export: fn hash13(p: vec3f) -> f32
//@ export: fn hash13_preview(p: vec2f, z: f32) -> f32
//@ param: z argument f32 range(0.0, 100.0)

fn hash13( p : vec3f ) -> f32
{
  var p3 = fract( p * 0.1031 );
  p3 += vec3f( dot( p3, p3.zyx + 31.32 ) );
  return fract( ( p3.x + p3.y ) * p3.z );
}

fn hash13_preview( p : vec2f, z : f32 ) -> f32
{
  return hash13( vec3f( p, z ) );
}
