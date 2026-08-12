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
  vec3 result = LinearToSrgb( src.rgb );
  //result = texture( sourceTexture, vUv ).rgb;
  // Forward the coverage alpha from the tone mapping pass (background=0,
  // geometry=1) instead of hardcoding opaque, so the canvas this pass writes
  // to the default framebuffer can be alpha-composited by the caller.
  frag_color = vec4( result, src.a );
}