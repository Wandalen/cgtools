#version 300 es

// GLSL 300 es twin of `main.wgsl` `vs_main`, consumed by the gpu_hal WebGL2
// backend. `main.wgsl` is the canonical source ( ADR-001 §5 ) — edit it first
// and mirror changes here. Uniform block names follow the HAL binding-name
// convention `ub_{group}_{binding}`; attribute locations match
// `Geometry::vertex_layouts`.

layout( location = 0 ) in vec3 a_position;
layout( location = 1 ) in vec3 a_normal;
layout( location = 2 ) in vec2 a_uv_0;
layout( location = 3 ) in vec4 a_color_0;

layout( std140 ) uniform ub_0_0
{
  mat4 view_matrix;
  mat4 projection_matrix;
  // xyz — world-space camera position; w — exposure ( applied as exp2 ).
  vec4 position_exposure;
} camera;

layout( std140 ) uniform ub_2_0
{
  mat4 world_matrix;
  // mat3 in std140: three vec4-aligned columns, matching `ModelRaw`.
  mat3 normal_matrix;
} model;

out vec3 v_world_position;
out vec3 v_normal;
out vec2 v_uv_0;
out vec4 v_color_0;

void main()
{
  vec4 world_position = model.world_matrix * vec4( a_position, 1.0 );
  v_world_position = world_position.xyz;
  v_normal = normalize( model.normal_matrix * a_normal );
  v_uv_0 = a_uv_0;
  v_color_0 = a_color_0;
  gl_Position = camera.projection_matrix * camera.view_matrix * world_position;
}
