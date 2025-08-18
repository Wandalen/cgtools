#version 300 es
precision highp float;

uniform vec3 u_color;
uniform float time;
out vec4 frag_color;

in vec2 vUv;

vec3 stripes()
{
  int k = int( vUv.x * 10.0 );
  if( k % 2 == 0) { return vec3( 0.0 ); }
  else { return vec3( 1.0 ); }
}

void main()
{
  vec3 col = vec3( 112.21, 201.45, 94.35 ) / 255.0;

  col = stripes();
  
  col = vec3( fract( vUv.x - time ) );
  //col = vec3( vUv.x  );
  frag_color = vec4( col, 1.5 );
}
