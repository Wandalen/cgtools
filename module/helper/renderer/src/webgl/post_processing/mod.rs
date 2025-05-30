
mod private
{
  pub const VS_TRIANGLE : &'static str = include_str!( "../shaders/big_triangle.vert" );
}

crate::mod_interface!
{
  layer gbuffer;
  layer unreal_bloom;
  layer composer;
  layer tonemapping;
  layer to_srgb;
  layer blend;
  layer outline;

  own use
  {
    VS_TRIANGLE
  };
}