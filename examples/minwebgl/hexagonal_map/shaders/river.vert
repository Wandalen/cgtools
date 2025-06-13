#version 300 es

uniform vec2 u_scale;
uniform vec2 u_camera_pos;
uniform mat3 u_transform;

void main()
{
  const vec2[] VERTICES = vec2[]
  (
    vec2(  1.0, -1.0 ),
    vec2( -1.0, -1.0 ),
    vec2(  1.0,  1.0 ),
    vec2( -1.0,  1.0 )
  );

  vec2 position = VERTICES[ gl_VertexID ];
  position = u_scale * ( ( u_transform * vec3( position, 1.0 ) ).xy + u_camera_pos );
  gl_Position = vec4( position, 0.0, 1.0 );
}
