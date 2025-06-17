#version 300 es

uniform vec2 u_scale;
uniform vec2 u_camera_pos;
uniform mat3 u_transform;
uniform float u_width;

void main()
{
  float len = length( vec2( sqrt(3.0) / 2.0, 0.5 ) );
  float ratio = u_width / len;
  // float ratio = 10.0;

  vec2[] points = vec2[]
  (
    ratio * vec2( sqrt( 3.0 ) / 2.0, 0.5 ),
    ratio * vec2( 0.0, 0.0 ),
    ratio * vec2( 1.0, 0.0 ),
    ratio * vec2( sqrt(3.0) / 2.0, -0.5 )
  );

  vec2 position = points[ gl_VertexID ];
  position = u_scale * ( ( u_transform * vec3( position, 1.0 ) ).xy + u_camera_pos );
  gl_Position = vec4( position, 0.0, 1.0 );
}
