#version 300 es

// The grid plane is authored directly in world space (no separate model
// matrix), so the vertex position doubles as the world position.
layout ( location = 0 ) in vec3 a_position;

uniform mat4 u_view_proj;

out vec3 v_world_pos;

void main()
{
  v_world_pos = a_position;
  gl_Position = u_view_proj * vec4( a_position, 1.0 );
}
