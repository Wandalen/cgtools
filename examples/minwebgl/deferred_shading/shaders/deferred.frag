#version 300 es
// This shader is for deferred lighting
// It calculates lighting of 50 point lights

precision mediump float;

in vec3 v_normal;

layout( location = 0 ) out vec4 frag_color;

void main()
{
  vec4 pos = gl_FragCoord;
  frag_color = vec4( 1.0, 1.0, 0.0, 1.0 );
}
