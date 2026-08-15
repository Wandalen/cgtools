//@ name: value_noise3
//@ description: Trilinear-interpolated value noise sampled at a 3D point, in [0, 1).
//@ tags: category:noise
//@ depends_on: hash13
//@ export: fn value_noise3(p: vec3f) -> f32
//@ export: fn value_noise3_preview(p: vec2f) -> f32

fn value_noise3( p : vec3f ) -> f32
{
  let i = floor( p );
  let f = fract( p );
  let u = f * f * ( 3.0 - 2.0 * f );
  let c000 = hash13( i );
  let c100 = hash13( i + vec3f( 1.0, 0.0, 0.0 ) );
  let c010 = hash13( i + vec3f( 0.0, 1.0, 0.0 ) );
  let c110 = hash13( i + vec3f( 1.0, 1.0, 0.0 ) );
  let c001 = hash13( i + vec3f( 0.0, 0.0, 1.0 ) );
  let c101 = hash13( i + vec3f( 1.0, 0.0, 1.0 ) );
  let c011 = hash13( i + vec3f( 0.0, 1.0, 1.0 ) );
  let c111 = hash13( i + vec3f( 1.0, 1.0, 1.0 ) );
  let z0 = mix( mix( c000, c100, u.x ), mix( c010, c110, u.x ), u.y );
  let z1 = mix( mix( c001, c101, u.x ), mix( c011, c111, u.x ), u.y );
  return mix( z0, z1, u.z );
}

fn value_noise3_preview( p : vec2f ) -> f32
{
  return value_noise3( vec3f( p, 1.7 ) );
}
