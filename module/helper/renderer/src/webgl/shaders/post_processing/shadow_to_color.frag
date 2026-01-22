#version 300 es
precision highp float;

uniform sampler2D sourceTexture;
uniform vec3 color;

in vec2 vUv;
out vec4 frag_color;

void main()
{
  // Read shadow value from red channel (1 = max shadow, 0 = no shadow)
  float shadow_value = texture( sourceTexture, vUv ).r;

  // Apply formula: (1 - shadow_value) * color
  vec3 result = ( 1.0 - shadow_value ) * color;

  frag_color = vec4( result, 1.0 );
}
