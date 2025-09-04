#version 300 es
precision highp float;

uniform float time;
uniform float totalDistance;

in vec2 vUv;

out vec4 frag_color;

float stripes( float v )
{
  int k = int( v * 10.0 );
  if( k % 2 == 0) { return 0.0; }
  else { return 1.0; }
}

void main()
{
  vec2 uv = vUv;
  float currentDistance = vUv.x * totalDistance;

  uv.x = currentDistance / 800.0;

  vec3 col = vec3( 112.21, 201.45, 94.35 ) / 255.0;
  float stripe = stripes( uv.x  * 10.0 - time );
  //frag_color = vec4( col * stripe, stripe * 0.5 );
}
