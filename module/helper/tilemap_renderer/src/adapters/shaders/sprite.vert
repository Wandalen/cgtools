#version 300 es
precision highp float;

// Uniforms
uniform mat3 u_transform;    // model transform (column-major 3x3)
uniform vec2 u_viewport;     // viewport size in pixels
uniform vec4 u_region;       // sprite region: x, y, w, h in pixels (same convention as sprite_batch.vert)
uniform vec2 u_tex_size;     // sheet dimensions in pixels
uniform float u_depth;       // Transform::depth; range [-u_max_depth, u_max_depth]
uniform float u_max_depth;   // RenderConfig::max_depth; defines the usable depth range

out vec2 v_uv;

void main()
{
  // Generate unit quad from gl_VertexID (triangle strip: 0,1,2,3)
  vec2 quad = vec2( float( gl_VertexID & 1 ), float( ( gl_VertexID >> 1 ) & 1 ) );

  // Compute UV from pixel region and sheet size (matches sprite_batch.vert).
  v_uv = ( u_region.xy + quad * u_region.zw ) / u_tex_size;

  // Scale unit quad to sprite's pixel size (region.zw), then apply transform.
  vec3 world = u_transform * vec3( quad * u_region.zw, 1.0 );

  // Convert to clip space: pixel coords → -1..1 (Y-up)
  vec2 ndc = ( world.xy / u_viewport ) * 2.0 - 1.0;

  // Negate so higher user depth → smaller clip-space z → wins LEQUAL depth test.
  // Divide by u_max_depth to map [-max_depth, max_depth] → [-1, 1].
  // Out-of-contract depths fall outside [-1, 1] and are clipped by the GPU —
  // that visible failure is preferable to a clamp that would silently collapse
  // ordering among overflow values.
  gl_Position = vec4( ndc, -u_depth / u_max_depth, 1.0 );
}
