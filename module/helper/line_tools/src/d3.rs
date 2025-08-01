mod private
{
  pub const MERGED_VERTEX_SHADER : &'static str = include_str!( "./d3/shaders/merged.vert" );
}

crate::mod_interface!
{
  layer line;

  own use
  {
    MERGED_VERTEX_SHADER
  };
}