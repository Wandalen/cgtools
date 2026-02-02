//! This module provides a convenient `Texture` struct that encapsulates a `wgpu` texture
//! along with its view, sampler, and dimensions, simplifying texture management.

use mingl::mod_interface;

mod private
{
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
}
