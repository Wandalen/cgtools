#version 300 es

precision mediump float;

out vec4 out_color;

void main()
{
  vec2 coord = gl_PointCoord - vec2( 0.5 );
  float distance_squared = dot( coord, coord );

  if ( distance_squared > 0.25 )
  {
    discard;
  }

  out_color = vec4( 1.0 );
}
