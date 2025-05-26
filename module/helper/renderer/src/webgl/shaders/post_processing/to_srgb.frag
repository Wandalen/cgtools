

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
  vec3 result = LinearToSrgb( texture( sourceTexture, vUv ).rgb );
  frag_color = vec4( result, 1.0 );
}