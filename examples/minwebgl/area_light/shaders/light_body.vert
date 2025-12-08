#version 300 es

// already transformed with model matrix
layout( location = 0 ) in vec3 a_position;

uniform mat4 u_view_projection;

void main()
{
  gl_Position = u_view_projection * vec4( a_position, 1.0 );
}
