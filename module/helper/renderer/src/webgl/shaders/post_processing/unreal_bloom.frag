

uniform sampler2D blurTexture0;
uniform sampler2D blurTexture1;
uniform sampler2D blurTexture2;
uniform sampler2D blurTexture3;
uniform sampler2D blurTexture4;
uniform float bloomStrength;
uniform float bloomRadius;
uniform float bloomFactors[ NUM_MIPS ];
uniform vec3 bloomTintColors[ NUM_MIPS ];

in vec2 vUv;
out vec4 frag_color;

float lerpBloomFactor( const in float factor ) 
{
  float mirrorFactor = 1.2 - factor;
  return mix( factor, mirrorFactor, bloomRadius );
}


void main()
{
  vec3 result = vec3( 0.0 );

  vec3 color1 = lerpBloomFactor( bloomFactors[ 0 ] ) * bloomTintColors[ 0 ] * texture( blurTexture0, vUv );
  vec3 color2 = lerpBloomFactor( bloomFactors[ 1 ] ) * bloomTintColors[ 1 ] * texture( blurTexture1, vUv );
  vec3 color3 = lerpBloomFactor( bloomFactors[ 2 ] ) * bloomTintColors[ 2 ] * texture( blurTexture2, vUv );
  vec3 color4 = lerpBloomFactor( bloomFactors[ 3 ] ) * bloomTintColors[ 3 ] * texture( blurTexture3, vUv );
  vec3 color5 = lerpBloomFactor( bloomFactors[ 4 ] ) * bloomTintColors[ 4 ] * texture( blurTexture4, vUv );
 
  result =  color1 + color2 + color3 + color4 + color5;
  result *= bloomStrength

  frag_color = vec4( result, 1.0 );
}