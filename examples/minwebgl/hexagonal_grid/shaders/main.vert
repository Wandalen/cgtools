#version 300 es

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 translation;
uniform vec2 u_zoom;
uniform vec2 u_rotation;

void main()
{
  mat2 rot = mat2
  (
    u_rotation.x, -u_rotation.y,
    u_rotation.y, u_rotation.x
  );
  vec2 pos = u_zoom * ( rot * position + translation );
  gl_Position = vec4( pos.xy, 0.0, 1.0 );
}
