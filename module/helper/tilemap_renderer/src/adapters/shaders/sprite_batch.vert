#version 300 es
precision highp float;

// Per-instance (divisor = 1)
layout( location = 0 ) in vec3 i_transform_0;
layout( location = 1 ) in vec3 i_transform_1;
layout( location = 2 ) in vec3 i_transform_2;
layout( location = 3 ) in vec4 i_region; // x, y, w, h in pixels
layout( location = 4 ) in vec4 i_tint;

uniform vec2 u_viewport;
uniform vec2 u_tex_size; // sheet dimensions in pixels
uniform mat3 u_parent;   // batch parent transform

out vec2 v_uv;
out vec4 v_tint;

void main()
{
  // Generate unit quad from gl_VertexID (triangle strip: 0,1,2,3)
  vec2 quad = vec2( float( gl_VertexID & 1 ), float( ( gl_VertexID >> 1 ) & 1 ) );

  // Compute UV from pixel region and sheet size
  v_uv = ( i_region.xy + quad * i_region.zw ) / u_tex_size;
  v_tint = i_tint;

  // Instance transform (row-major in buffer, transpose to column-major)
  mat3 inst = transpose( mat3( i_transform_0, i_transform_1, i_transform_2 ) );

  // Scale quad to sprite pixel size, apply instance then parent transform
  vec3 world = u_parent * inst * vec3( quad * i_region.zw, 1.0 );

  vec2 ndc = ( world.xy / u_viewport ) * 2.0 - 1.0;

  gl_Position = vec4( ndc, 0.0, 1.0 );
}
