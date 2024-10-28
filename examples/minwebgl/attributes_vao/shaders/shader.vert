#version 300 es

layout( location=0 ) in vec4 a_position;
layout( location=1 ) in float a_point_size;
layout( location=2 ) in vec4 a_color;

out vec4 v_color;

void main()
{
  v_color = a_color;
  gl_PointSize = a_point_size;
  gl_Position = a_position;
}
