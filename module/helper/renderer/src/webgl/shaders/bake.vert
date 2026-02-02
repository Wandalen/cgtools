#version 300 es

layout( location = 0 ) in vec3 a_pos;
layout( location = 1 ) in vec3 a_normal;
layout( location = 2 ) in vec2 a_texcoord;

uniform mat4 u_model;
uniform mat4 u_light_view_projection;

out vec3 v_world_pos;
out vec3 v_normal;
out vec4 v_light_space_pos;

void main()
{
  vec4 world_pos = u_model * vec4( a_pos, 1.0 );
  v_world_pos = world_pos.xyz;
  v_normal = mat3( u_model ) * a_normal;

  v_light_space_pos = u_light_view_projection * world_pos;

  // Position fragment in clip space based on UV coordinates
  // This makes each triangle rasterize to its corresponding lightmap region
  // UV [0,1] maps to NDC [-1,1]
  gl_Position = vec4( a_texcoord * 2.0 - 1.0, 0.0, 1.0 );
}
