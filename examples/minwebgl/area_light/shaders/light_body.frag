#version 300 es
precision mediump float;

out vec4 frag_color;

uniform vec3 u_color;

vec3 to_srgb( vec3 v ) { return pow( v, vec3( 1.0 / 2.2 ) ); }

void main()
{
  frag_color = vec4( to_srgb( u_color ), 1.0 );
}
