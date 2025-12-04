#version 300 es
precision highp float;

out vec4 frag_color;

void main()
{
  float depth = gl_FragCoord.z;
  frag_color = vec4( depth, 0.0, 0.0, 1.0 );
  frag_color = depth;
}
