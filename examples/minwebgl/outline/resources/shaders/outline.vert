#version 300 es
// Interpolated texture coordinate from the vertex shader for the current pixel.
out vec2 v_tex_coord;
// Interpolated direction for sampling skybox texture
out vec3 v_dir;

uniform mat4 u_inv_projection;
uniform mat4 u_inv_view;

void main()
{
  float x = float( gl_VertexID / 2 );
  float y = float( gl_VertexID % 2 );

  v_tex_coord = vec2( x, y ) * 2.0;

  gl_Position = vec4( x * 4.0 - 1.0, y * 4.0 - 1.0, 1.0, 1.0 );

  vec4 clip = vec4( v_tex_coord * 2.0 - 1.0, 1.0, 1.0 );
  vec4 view = u_inv_projection * clip;

  v_dir = ( u_inv_view * view ).xyz;
}
