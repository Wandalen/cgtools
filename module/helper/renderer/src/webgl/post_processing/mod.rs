
mod private
{
  /// A constant string containing the source code for the "big triangle" vertex shader.
  pub const VS_TRIANGLE : &'static str = include_str!( "../shaders/big_triangle.vert" );
}

crate::mod_interface!
{
  #[ allow( missing_docs ) ]
  layer gbuffer;
  /// Unreal bloom post-processing
  layer unreal_bloom;
  /// Puts post-processing effects in a pipeline
  layer composer;
  /// Tomapping post-processing
  layer tonemapping;
  /// Convert to srgb
  layer to_srgb;
  /// Blend post-processing
  layer blend;
  /// Creates an outline
  layer outline;

  own use
  {
    VS_TRIANGLE
  };
}