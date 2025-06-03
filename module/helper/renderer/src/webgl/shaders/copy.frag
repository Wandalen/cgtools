#version 300 es
precision highp float;

uniform sampler2D sourceTexture;

in vec2 vUv;
out vec4 frag_color;

void main()
{
  frag_color = texture( sourceTexture, vUv );
}