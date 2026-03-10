#version 300 es
precision highp float;

// Per-vertex
layout( location = 0 ) in vec2 a_position; // unit quad 0..1
layout( location = 1 ) in vec2 a_uv;

// Uniforms
uniform mat3 u_transform;   // model transform (column-major 3x3)
uniform vec2 u_viewport;    // viewport size in pixels
uniform vec4 u_uv_rect;     // sprite region: x, y, w, h in UV space
uniform vec2 u_sprite_size;  // natural size of sprite region in pixels

out vec2 v_uv;
out vec2 v_pos;

void main()
{
  // Map UV from full quad to sprite sub-region
  v_uv = u_uv_rect.xy + a_uv * u_uv_rect.zw;

  // Scale unit quad to sprite's natural pixel size, then apply transform
  vec3 world = u_transform * vec3( a_position * u_sprite_size, 1.0 );

  // Convert to clip space: pixel coords → -1..1 (Y-up)
  vec2 ndc = ( world.xy / u_viewport ) * 2.0 - 1.0;

  v_pos = world.xy;
  gl_Position = vec4( ndc, 0.0, 1.0 );
}
