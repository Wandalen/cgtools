
mod private
{
  /// A constant string containing the source code for the "big triangle" vertex shader.
  pub const VS_TRIANGLE : &str = include_str!( "../shaders/big_triangle.vert" );
}

crate::mod_interface!
{
  /// G-buffer attachment set and framebuffer plumbing for deferred-style passes
  layer gbuffer;
  /// Unreal bloom post-processing
  layer unreal_bloom;
  /// The `Pass` trait and the ping-pong `SwapFramebuffer` for chaining effects
  layer pass;
  /// Tomapping post-processing
  layer tonemapping;
  /// Color grading post-processing
  layer color_grading;
  /// Convert to srgb
  layer to_srgb;
  /// Blend post-processing
  layer blend;
  /// Creates an outline
  layer outline;
  /// Converts shadow texture to colored base color
  layer shadow_to_color;

  own use
  {
    VS_TRIANGLE
  };
}