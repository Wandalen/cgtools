
#define KERNEL_SIZE 11

uniform sampler2D tSource;
uniform float kernel[ KERNEL_SIZE ]
uniform vec2 invSize;
uniform vec2 blurDir;
uniform int radius;

in vec2 vUv;
out vec4 frag_color;


void main()
{
  vec3 result = vec3( 0.0 );
  float weightSum = kernel[ 0 ];
  result += tSource( vUV ).rgb * kernel[ 0 ];
  for( int i = 1; i < radius; i++)
  {
    result += tSource( vUv + invSize * float( i ) * blurDir ).rgb * kernel[ i ];
    result += tSource( vUv - invSize * float( i ) * blurDir ).rgb * kernel[ i ];
    weightSum += 2.0 * kernel[ i ];
  }

  frag_color = vec4( result / weightSum, 1.0 );
}