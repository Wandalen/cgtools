#version 300 es
#pragma vscode_glsllint_stage : vert

layout( location=0 ) in vec4 a_position;
layout( location=1 ) in vec4 a_color;

uniform mat4x4 project_view_matrix;

out vec4 vColor;

void main()
{
  vColor = a_color;
  gl_Position = project_view_matrix * a_position;
}


