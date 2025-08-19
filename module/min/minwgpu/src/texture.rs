use mingl::mod_interface;

mod private
{
  #[ derive( Debug, Clone ) ]
  pub struct Texture
  {
    pub texture : wgpu::Texture,
    pub extend : wgpu::Extent3d,
    pub view : wgpu::TextureView,
    pub sampler : wgpu::Sampler,
  }

  impl Texture
  {
    pub fn new
    (
      texture : wgpu::Texture,
      extend : wgpu::Extent3d,
      view : wgpu::TextureView,
      sampler : wgpu::Sampler
    ) -> Self
    {
      Self { texture, extend, view, sampler }
    }
  }
}

mod_interface!
{
  own use Texture;
}
