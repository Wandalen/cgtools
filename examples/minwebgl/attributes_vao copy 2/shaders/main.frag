#version 300 es
precision mediump float;

uniform sampler2D some_texture;

out vec4 frag_color;
in vec2 vUv;

void main()
{
  
  vec3 color = vec3( 0.0 );
  color = textureLod( some_texture, vUv, 2.0 ).xyz;
  frag_color = vec4( color, 1.0 );
}