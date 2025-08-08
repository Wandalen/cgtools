mod private
{
  /// The vertex shader for the merged line rendering.
  pub const MERGED_VERTEX_SHADER : &'static str = include_str!( "./d3/shaders/merged.vert" );
}

crate::mod_interface!
{
  /// Layer for line-related functionalities.
  layer line;

  own use
  {
    MERGED_VERTEX_SHADER
  };
}