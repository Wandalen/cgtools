//@ name: srgb
//@ description: Linear-to-sRGB and sRGB-to-linear color conversions, piecewise-exact.
//@ tags: category:color
//@ depends_on:
//@ export: fn linear_to_srgb(color: vec3f) -> vec3f
//@ export: fn srgb_to_linear(color: vec3f) -> vec3f
//@ export: fn srgb_preview(p: vec2f) -> vec3f

fn linear_to_srgb( color : vec3f ) -> vec3f
{
  // Exact piecewise IEC 61966-2-1 curve : linear segment below the
  // threshold, 1/2.4 power segment above.
  let more = pow( color, vec3f( 1.0 / 2.4 ) ) * 1.055 - vec3f( 0.055 );
  let less = color * 12.92;
  return select( more, less, color <= vec3f( 0.0031308 ) );
}

fn srgb_to_linear( color : vec3f ) -> vec3f
{
  let more = pow( ( color + vec3f( 0.055 ) ) / 1.055, vec3f( 2.4 ) );
  let less = color / 12.92;
  return select( more, less, color <= vec3f( 0.04045 ) );
}

fn srgb_preview( p : vec2f ) -> vec3f
{
  let lin = vec3f( p.x );
  if ( p.y > 0.0 )
  {
    return lin;
  }
  return linear_to_srgb( lin );
}
