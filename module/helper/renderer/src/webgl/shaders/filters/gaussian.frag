precision mediump float;

uniform sampler2D sourceTexture;
uniform float kernel[ KERNEL_RADIUS ]
uniform vec2 invSize;
uniform vec2 blurDir;

in vec2 vUv;
out vec4 frag_color;


void main()
{
  vec3 result = vec3( 0.0 );
  float weightSum = kernel[ 0 ];
  result += texture( sourceTexture, vUV ).rgb * kernel[ 0 ];
  for( int i = 1; i < radius; i++)
  {
    result += texture( sourceTexture, vUv + invSize * float( i ) * blurDir ).rgb * kernel[ i ];
    result += texture( sourceTexture, vUv - invSize * float( i ) * blurDir ).rgb * kernel[ i ];
    // Maybe the weight sum should be the sum texel, instead of kernel coefficients
    weightSum += 2.0 * kernel[ i ];
  }

  frag_color = vec4( result / weightSum, 1.0 );
}