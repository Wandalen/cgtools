#version 300 es
precision highp float;

in vec2 vUv;

out vec4 frag_color;

void main()
{
  vec2 uv = vUv * 2.0 - 1.0;

  vec3 col = vec3( 104.41, 28.19, 28.19 ) / 255.0;
  col *= exp( - length( uv ) );
  col = vec3( 1.0 * 0.5, 0.0, 0.0 );
  col = vec3( 1.0 );
  frag_color = vec4( col , 1.0 );
}
