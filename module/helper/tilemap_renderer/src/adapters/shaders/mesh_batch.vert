#version 300 es
precision highp float;

// Per-vertex (from geometry VAO)
layout( location = 0 ) in vec2 a_position;
layout( location = 1 ) in vec2 a_uv;

// Per-instance (divisor = 1)
layout( location = 2 ) in vec3 i_transform_0;
layout( location = 3 ) in vec3 i_transform_1;
layout( location = 4 ) in vec3 i_transform_2;
layout( location = 5 ) in float i_depth;

uniform vec2 u_viewport;
uniform mat3 u_parent; // batch parent transform
uniform float u_parent_depth;
uniform float u_max_depth; // RenderConfig::max_depth; defines the usable depth range

out vec2 v_uv;

void main()
{
  v_uv = a_uv;

  // Instance transform: each i_transform_N is a column of the column-major matrix from Transform::to_mat3().
  mat3 inst = mat3( i_transform_0, i_transform_1, i_transform_2 );
  vec3 world = u_parent * inst * vec3( a_position, 1.0 );

  vec2 ndc = ( world.xy / u_viewport ) * 2.0 - 1.0;

  // Negate so higher user depth → smaller clip-space z → wins LEQUAL depth test.
  // Each of `u_parent_depth` / `i_depth` is individually in
  // [-u_max_depth, u_max_depth] by contract; their sum is the caller's
  // responsibility to keep within the same range. Divide by u_max_depth so the
  // in-contract sum maps into clip-space [-1, 1]. Out-of-contract sums are
  // clipped by the GPU — the caller will see geometry disappear, which is
  // easier to diagnose than the silent z-fighting a clamp would introduce.
  gl_Position = vec4( ndc, -( u_parent_depth + i_depth ) / u_max_depth, 1.0 );
}
