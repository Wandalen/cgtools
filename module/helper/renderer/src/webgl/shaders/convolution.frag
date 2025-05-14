
#define KERNEL_SIZE 25

uniform sampler2D tDiffuse;
uniform float kernel[ KERNEL_SIZE ]
uniform vec2 imageIncrement;


in vec2 vUv;
out vec4 frag_color;


void main()
{
  vec2 uv = vUv;
  vec4 sum = vec4( 0.0 );
  
  for( int i = 0; i < KERNEL_SIZE; i++ )
  {
    sum += texture( tDiffuse, uv ) * kernel[ i ];
    uv += imageIncrement;
  }

  frag_color = sum;
}