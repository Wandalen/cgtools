#version 300 es

layout( location = 0 ) in vec4 position;

uniform mat4x4 projection_matrix;
uniform float angle;

void main()
{
  mat4x4 rotate_x = mat4x4
  (
   1.0,  0.0,           0.0,           0.0,
   0.0,  cos( angle ),  sin( angle ),  0.0,
   0.0, -sin( angle ),  cos( angle ),  0.0,
   0.0,  0.0,           0.0,           1.0
  );

  mat4x4 rotate_y = mat4x4
  (
   cos( angle ),  0.0, -sin( angle ),  0.0,
   0.0,           1.0,  0.0,           0.0,
   sin( angle ),  0.0,  cos( angle ),  0.0,
   0.0,           0.0,  0.0,           1.0
  );

  mat4x4 translate = mat4x4
  (
   1.0,  0.0,  0.0,  0.0,
   0.0,  1.0,  0.0,  0.0,
   0.0,  0.0,  1.0,  0.0,
   0.0,  0.0, -6.0,  1.0
  );

  mat4x4 transform = translate * rotate_x * rotate_y;

  gl_PointSize = 15.0;
  gl_Position = projection_matrix * transform * position;
}
