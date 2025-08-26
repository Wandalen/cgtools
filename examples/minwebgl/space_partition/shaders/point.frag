#version 300 es
precision highp float;

in vec3 vColor;

out vec4 frag_color;

void main()
{
  vec2 uv = gl_PointCoord * 2.0 - 1.0;
  vec3 col = vec3( 0.0 );
  col = vec3( smoothstep( 0.0, 1.0, length( uv ) ) );

  frag_color = vec4( col, 1.0 );
}
