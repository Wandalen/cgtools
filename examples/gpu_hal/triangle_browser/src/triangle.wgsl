// Uniform-colored triangle — the same WGSL `gpu_hal/tests/native_backend_test.rs`'s
// `triangle_render_readback` uses. The WebGPU and native backends consume this
// directly; the WebGL backend's GLSL ES 300 override is generated from it at
// build time by gpu_hal's `webgl_build` kit — see `build.rs`.

struct Color
{
  value : vec4f
}

@group( 0 ) @binding( 0 ) var< uniform > color : Color;

@vertex
fn vs_main( @location( 0 ) position : vec2f ) -> @builtin( position ) vec4f
{
  return vec4f( position, 0.0, 1.0 );
}

@fragment
fn fs_main() -> @location( 0 ) vec4f
{
  return color.value;
}
