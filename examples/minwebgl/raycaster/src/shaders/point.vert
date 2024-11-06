#version 300 es

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in float point_size;
layout( location = 2 ) in vec3 color;

out vec3 v_color;

void main()
{
  v_color = color;
  gl_PointSize = point_size;
  gl_Position = vec4( position, 0.0, 1.0 );
}
