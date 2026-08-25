mod private
{
  /// The vertex shader for the line rendering.
  pub const MAIN_VERTEX_SHADER : &str = include_str!( "./d3/shaders/main.vert" );

  /// The fragment shader for the line rendering.
  pub const MAIN_FRAGMENT_SHADER : &str = include_str!( "./d3/shaders/main.frag" );
}

crate::mod_interface!
{
  /// Layer for line-related functionalities.
  layer line;

  own use
  {
    MAIN_VERTEX_SHADER,
    MAIN_FRAGMENT_SHADER
  };
}