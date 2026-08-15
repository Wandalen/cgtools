//@ name: hash33
//@ description: Three-channel hash of a 3D point, each channel in [0, 1).
//@ tags: category:hash
//@ depends_on:
//@ export: fn hash33(p: vec3f) -> vec3f
//@ export: fn hash33_preview(p: vec2f) -> vec3f

fn hash33( p : vec3f ) -> vec3f
{
  var p3 = fract( p * vec3f( 0.1031, 0.1030, 0.0973 ) );
  p3 += vec3f( dot( p3, p3.yxz + 33.33 ) );
  return fract( ( p3.xxy + p3.yxx ) * p3.zyx );
}

fn hash33_preview( p : vec2f ) -> vec3f
{
  return hash33( vec3f( p, 42.0 ) );
}
