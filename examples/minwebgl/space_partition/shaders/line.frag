#version 300 es
precision highp float;

in vec2 vUv;

out vec4 frag_color;

void main()
{
  vec2 uv = vUv;
  vec3 col = vec3( 1.0, 0.0, 0.0 );
  frag_color = vec4( col, 0.5 );
}
