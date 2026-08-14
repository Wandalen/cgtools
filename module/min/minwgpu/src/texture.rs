//! This module provides a convenient `Texture` struct that encapsulates a `wgpu` texture
//! along with its view, sampler, and dimensions, simplifying texture management.

use mingl::mod_interface;

mod private
{
  /// Creates a 2D render-target [`Texture`] of the given `( width, height )` size and format.
  ///
  /// The texture is created with `RENDER_ATTACHMENT | COPY_SRC` usage, one mip level and
  /// no multisampling, so it can serve as a color attachment and later be copied out —
  /// e.g. by `readback::rgba8`. The bundled view and sampler are default-configured.
  #[ must_use ]
  pub fn render_target_2d( device : &wgpu::Device, size : ( u32, u32 ), format : wgpu::TextureFormat ) -> Texture
  {
    let ( width, height ) = size;
    let extend = wgpu::Extent3d { width, height, depth_or_array_layers : 1 };
    let texture = device.create_texture
    (
      &wgpu::TextureDescriptor
      {
        label : Some( "render_target_2d" ),
        size : extend,
        mip_level_count : 1,
        sample_count : 1,
        dimension : wgpu::TextureDimension::D2,
        format,
        usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats : &[],
      }
    );
    let view = texture.create_view( &wgpu::TextureViewDescriptor::default() );
    let sampler = device.create_sampler( &wgpu::SamplerDescriptor::default() );
    Texture::new( texture, extend, view, sampler )
  }

  /// A struct that bundles a `wgpu::Texture` with its associated view, sampler, and extent.
  ///
  /// This provides a convenient way to manage all the components of a texture as a single unit.
  #[ non_exhaustive ]
  #[ derive( Debug, Clone ) ]
  pub struct Texture
  {
    /// The raw `wgpu` texture resource.
    pub texture : wgpu::Texture,
    /// The dimensions (width, height, depth) of the texture.
    pub extend : wgpu::Extent3d,
    /// A view into the texture, describing how it should be accessed by shaders.
    pub view : wgpu::TextureView,
    /// The sampler that defines how the texture should be sampled in a shader.
    pub sampler : wgpu::Sampler,
  }

  impl Texture
  {
    /// Creates a new `Texture` instance from its constituent `wgpu` components.
    ///
    /// # Arguments
    /// * `texture` - The `wgpu::Texture` handle.
    /// * `extend` - The dimensions of the texture.
    /// * `view` - A pre-created `wgpu::TextureView` for the texture.
    /// * `sampler` - A pre-created `wgpu::Sampler` for the texture.
    #[ must_use ]
    #[ inline ]
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
  own use render_target_2d;
}
