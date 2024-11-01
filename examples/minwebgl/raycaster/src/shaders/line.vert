#version 300 es

layout( location = 0 ) in vec2 start;
layout( location = 1 ) in vec2 end;
layout( location = 2 ) in vec3 color;

out vec3 v_color;

void main()
{
  v_color = color;
  if ( gl_VertexID == 0 )
  {
    gl_Position = vec4( start, 0.0, 1.0 );
  }
  else
  {
    gl_Position = vec4( end, 0.0, 1.0 );
  }
}
