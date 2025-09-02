#version 300 es

uniform vec2 u_scale;
uniform vec2 u_camera_pos;
uniform mat3 u_transform;
uniform float u_width;

void main()
{
  const float SQRT_3 = 1.73205080757;
  const vec2[] POINTS = vec2[]
  (
    vec2( SQRT_3 / 2.0, 0.5 ),
    vec2( 0.0, 0.0 ),
    vec2( 1.0, 0.0 ),
    vec2( SQRT_3 / 2.0, -0.5 )
  );
  const float LEN = length( vec2( SQRT_3 / 2.0, 0.5 ) );

  float ratio = u_width / LEN;

  vec2 position = ratio * POINTS[ gl_VertexID ];
  position = u_scale * ( ( u_transform * vec3( position, 1.0 ) ).xy + u_camera_pos );
  gl_Position = vec4( position, 0.0, 1.0 );
}
