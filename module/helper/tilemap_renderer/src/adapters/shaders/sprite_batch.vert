#version 300 es
precision highp float;

// Per-instance (divisor = 1)
layout( location = 0 ) in vec3 i_transform_0;
layout( location = 1 ) in vec3 i_transform_1;
layout( location = 2 ) in vec3 i_transform_2;
layout( location = 3 ) in vec4 i_region; // x, y, w, h in pixels
layout( location = 4 ) in vec4 i_tint;
layout( location = 5 ) in float i_depth;

uniform vec2 u_viewport;
uniform vec2 u_tex_size; // sheet dimensions in pixels
uniform mat3 u_parent;   // batch parent transform
uniform float u_parent_depth;
uniform float u_max_depth; // RenderConfig::max_depth; defines the usable depth range

out vec2 v_uv;
out vec4 v_tint;

void main()
{
  // Generate unit quad from gl_VertexID (triangle strip: 0,1,2,3)
  vec2 quad = vec2( float( gl_VertexID & 1 ), float( ( gl_VertexID >> 1 ) & 1 ) );

  // Compute UV from pixel region and sheet size.
  // `i_region` carries the sprite rect in pixels (y grows top-down, standard
  // image convention). Textures are uploaded with UNPACK_FLIP_Y_WEBGL=1 so
  // uv.y=1 samples image row 0. Two inversions needed: (a) `( 1 - quad.y )`
  // maps sprite-top (quad.y=1) to region-top (region.y), and (b) the outer
  // `1 - ...` flips V to account for the texture-upload flip. Shared with
  // `sprite.vert` for a consistent single-draw / batch UV pipeline.
  v_uv = vec2
  (
    ( i_region.x + quad.x * i_region.z ) / u_tex_size.x,
    1.0 - ( i_region.y + ( 1.0 - quad.y ) * i_region.w ) / u_tex_size.y
  );
  v_tint = i_tint;

  // Instance transform: each i_transform_N is a column of the column-major matrix from Transform::to_mat3().
  mat3 inst = mat3( i_transform_0, i_transform_1, i_transform_2 );

  // Scale quad to sprite pixel size, apply instance then parent transform
  vec3 world = u_parent * inst * vec3( quad * i_region.zw, 1.0 );

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
