#version 300 es

layout( location = 0 ) in vec3 a_position;
layout( location = 1 ) in vec3 a_translation;
layout( location = 2 ) in float a_radius;
layout( location = 3 ) in vec3 a_color;

uniform mat4 u_mvp;

flat out mediump vec3 v_light_position;
flat out mediump float v_light_radius;
flat out mediump vec3 v_light_color;

void main()
{
  v_light_position = a_translation;
  v_light_radius = a_radius;
  v_light_color = a_color;
  gl_Position = u_mvp * vec4( a_position * a_radius + a_translation, 1.0 );
}
