#version 300 es

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;
layout( location = 2 ) in vec2 uv;

uniform mat4x4 project_view_matrix;

out vec3 vNormal;
out vec2 vUv;

void main()
{
  vNormal = normal;
  vUv = uv;

  gl_Position = project_view_matrix * vec4( position * 3.0, 1.0 );
}
