#version 300 es
precision highp float;

uniform sampler2D sourceTexture;

in vec2 vUv;
out vec4 frag_color;

vec3 LinearToSrgb( const in vec3 color )
{
  vec3 more = pow( color, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 );
  vec3 less = color * 12.92;

  return mix( more, less, vec3( lessThanEqual( color, vec3( 0.0031308 ) ) ) );
}

void main()
{
  vec4 src = texture( sourceTexture, vUv );
#ifdef FORWARD_ALPHA
  // Opt-in (`ToSrgbPass::alpha_forward_set`): forward the coverage alpha from
  // the tone mapping pass (background = 0, geometry = 1) so the canvas this pass
  // writes can be alpha-composited over other content, e.g. AR over a photo.
  // With a 0 clear the input is premultiplied: an MSAA edge pixel holds a * c.
  // The sRGB curve is concave, so encoding a * c directly gives more than a * srgb( c )
  // and can exceed a, which a premultiplied-alpha canvas shows as a bright halo.
  // Encode the straight colour and premultiply again. Clamped to 1 so a pass that
  // pushed rgb above alpha still leaves rgb <= alpha.
  vec3 result = src.a > 0.0
    ? LinearToSrgb( min( src.rgb / src.a, vec3( 1.0 ) ) ) * src.a
    : vec3( 0.0 );
  frag_color = vec4( result, src.a );
#else
  frag_color = vec4( LinearToSrgb( src.rgb ), 1.0 );
#endif
}