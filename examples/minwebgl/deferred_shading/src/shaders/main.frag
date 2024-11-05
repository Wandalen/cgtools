#version 300 es
precision mediump float;

in vec3 v_normal;
in vec3 v_world_pos;

out vec4 frag_color;

void main()
{
  frag_color = vec4( 1.0, 0.0, 0.0, 1.0 );
}
