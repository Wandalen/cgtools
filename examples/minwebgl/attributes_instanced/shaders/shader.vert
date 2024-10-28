#version 300 es

layout( location=0 ) in vec4 a_position;
layout( location=1 ) in vec4 a_color;
layout( location=2 ) in vec2 a_offset;

out vec4 v_color;

void main()
{
  v_color = a_color;
  vec4 pos = a_position;
  pos.xy += a_offset;
  gl_Position = pos;
}
