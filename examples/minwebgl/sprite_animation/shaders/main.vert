#version 300 es

layout( location = 0 ) in vec4 a_position;
layout( location = 1 ) in vec2 a_tex_coord;
layout( location = 2 ) in float a_depth;

out vec2 v_tex_coord;
out float v_depth;

void main()
{
  v_depth = a_depth;
  v_tex_coord = a_tex_coord;
  gl_Position = a_position;
}