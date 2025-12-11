#version 300 es
precision highp float;

in vec2 vUv;

uniform float radius;

out vec4 frag_color;

void main()
{
  vec2 uv = vUv * 2.0 - 1.0;
  vec3 col = vec3( 0.0 );
  col = vec3( 1.0, 0.0, 0.0 );

  float k = mix( 1.0, 0.0, smoothstep( 0.9, 1.0, length( uv ) ) );

  frag_color = vec4( col * k, 0.2 * k );
}
