#version 300 es

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;

uniform mat4 mvp;
uniform mat4 model;

out vec3 v_normal;
out vec3 v_world_pos;

void main()
{
  v_normal = normal;
  v_world_pos = vec3( model * vec4( position, 1.0 ) );
  gl_Position = mvp * vec4( position, 1.0 );
}
