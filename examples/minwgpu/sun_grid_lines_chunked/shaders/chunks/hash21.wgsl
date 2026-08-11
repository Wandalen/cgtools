//@ name: hash21
//@ description: Single-value hash of a 2D point into [0, 1).
//@ depends_on:
//@ export: fn hash21(p: vec2f) -> f32

fn hash21( p : vec2f ) -> f32
{
  var p3 = fract( vec3f( p.x, p.y, p.x ) * 0.1031 );
  p3 += vec3f( dot( p3, p3.yzx + 33.33 ) );
  return fract( ( p3.x + p3.y ) * p3.z );
}
