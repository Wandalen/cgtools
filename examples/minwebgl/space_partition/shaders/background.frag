#version 300 es
precision highp float;

in vec2 vUv;

out vec4 frag_color;

void main()
{
  vec2 uv = vUv * 2.0 - 1.0;

  vec3 col = vec3( 1.0 );
  frag_color = vec4( col , 1.0 );
}
