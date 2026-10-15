//@ name: value_noise3
//@ description: Trilinear-interpolated value noise sampled at a 3D point, in [0, 1).
//@ tags: category:noise
//@ depends_on: hash13
//@ export: fn value_noise3(p: vec3f, seed: f32) -> f32
//@ export: fn value_noise3_preview(p: vec2f, z: f32, seed: f32) -> f32
//@ param: z argument f32 range(0.0, 10.0)
//@ param: seed argument f32 range(-50.0, 50.0)

// seed offsets the integer lattice coordinate fed into each corner's hash
// -- vec3f + f32 broadcasts per-component in WGSL, same rule already
// proven for vec2f + f32 elsewhere in this codebase. seed = 0 ( this
// range's midpoint ) reproduces the original, unseeded pattern exactly.
fn value_noise3( p : vec3f, seed : f32 ) -> f32
{
  let i = floor( p );
  let f = fract( p );
  let u = f * f * ( 3.0 - 2.0 * f );
  let c000 = hash13( i + seed );
  let c100 = hash13( i + vec3f( 1.0, 0.0, 0.0 ) + seed );
  let c010 = hash13( i + vec3f( 0.0, 1.0, 0.0 ) + seed );
  let c110 = hash13( i + vec3f( 1.0, 1.0, 0.0 ) + seed );
  let c001 = hash13( i + vec3f( 0.0, 0.0, 1.0 ) + seed );
  let c101 = hash13( i + vec3f( 1.0, 0.0, 1.0 ) + seed );
  let c011 = hash13( i + vec3f( 0.0, 1.0, 1.0 ) + seed );
  let c111 = hash13( i + vec3f( 1.0, 1.0, 1.0 ) + seed );
  let z0 = mix( mix( c000, c100, u.x ), mix( c010, c110, u.x ), u.y );
  let z1 = mix( mix( c001, c101, u.x ), mix( c011, c111, u.x ), u.y );
  return mix( z0, z1, u.z );
}

fn value_noise3_preview( p : vec2f, z : f32, seed : f32 ) -> f32
{
  return value_noise3( vec3f( p, z ), seed );
}
