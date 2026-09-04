//@ name: hash22
//@ description: Two-channel hash of a 2D point, each channel in [0, 1).
//@ tags: category:hash
//@ depends_on:
//@ export: fn hash22(p: vec2f) -> vec2f

fn hash22( p : vec2f ) -> vec2f
{
  var p3 = fract( vec3f( p.x, p.y, p.x ) * vec3f( 0.1031, 0.1030, 0.0973 ) );
  p3 += vec3f( dot( p3, p3.yzx + 33.33 ) );
  return fract( ( p3.xx + p3.yz ) * p3.zy );
}
