#version 300 es
precision highp float;

// Uniforms
uniform mat3 u_transform;    // model transform (column-major 3x3)
uniform vec2 u_viewport;     // viewport size in pixels
uniform vec4 u_uv_rect;      // sprite region: x, y, w, h in UV space (0..1)
uniform vec2 u_sprite_size;  // natural size of sprite region in pixels
uniform float u_depth;       // NDC z; higher Transform::depth → drawn on top (see webgl.rs depth notes)

out vec2 v_uv;

void main()
{
  // Generate unit quad from gl_VertexID (triangle strip: 0,1,2,3)
  vec2 quad = vec2( float( gl_VertexID & 1 ), float( ( gl_VertexID >> 1 ) & 1 ) );

  // Map quad corner to sprite sub-region UV
  v_uv = u_uv_rect.xy + quad * u_uv_rect.zw;

  // Scale unit quad to sprite's natural pixel size, then apply transform
  vec3 world = u_transform * vec3( quad * u_sprite_size, 1.0 );

  // Convert to clip space: pixel coords → -1..1 (Y-up)
  vec2 ndc = ( world.xy / u_viewport ) * 2.0 - 1.0;

  // Negate so higher user depth → smaller clip-space z → wins LEQUAL depth test.
  gl_Position = vec4( ndc, -u_depth, 1.0 );
}
