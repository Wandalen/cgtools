#version 300 es

// GLSL 300 es twin of `tonemap.wgsl` `vs_main`, consumed by the gpu_hal WebGL2
// backend: one oversized triangle from `gl_VertexID`, no vertex buffers.
// `tonemap.wgsl` is the canonical source ( ADR-001 §5 ) — edit it first and
// mirror changes here.

void main()
{
  float x = float( gl_VertexID / 2 );
  float y = float( gl_VertexID % 2 );

  gl_Position = vec4( x * 4.0 - 1.0, y * 4.0 - 1.0, 0.0, 1.0 );
}
