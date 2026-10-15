//! This module provides a convenient `Texture` struct that encapsulates a `wgpu` texture
//! along with its view, sampler, and dimensions, simplifying texture management.

use mingl::mod_interface;

mod private
{
  // Fix(BUG-276): `render_target_2d` forwarded `size` straight to `wgpu::Device::create_texture`
  // with no precondition check, panicking ( `wgpu-core`'s `CreateTextureError::InvalidDimension(
  // TextureDimensionError::Zero(..))`, surfaced through `wgpu`'s default uncaptured-error handler
  // -- `panic!("wgpu error: {err}")` in `wgpu-core`'s `backend/wgpu_core.rs`, confirmed by reading
  // that source directly, since this crate never installs a custom handler ) whenever either
  // dimension was `0`.
  // Root cause: a render target is routinely sized to match a live window/canvas -- the exact
  // source BUG-165 already identified for `surface::surface_configure`'s `size` parameter -- so a
  // transiently zero-sized read ( e.g. a minimized window ) is a normal, reachable event, not a
  // caller bug, yet no precondition check stood between that reachable input and
  // `wgpu::Device::create_texture`'s panic-by-default behavior.
  // Pitfall: fixing one unguarded call path into a `wgpu` API that panics on zero-sized input does
  // not protect a sibling call path taking the same shape of input -- `surface_configure`
  // (BUG-165) and `render_target_2d` both accept a caller-supplied `( u32, u32 )` size with no
  // shared validation chokepoint between them, so each needed its own guard.
  /// Returns `true` when both components of `size` are non-zero.
  ///
  /// Split out of [`render_target_2d`] so this precondition is unit-testable without a real GPU
  /// device -- see `tests/texture_test.rs`. Returns `bool` rather than a `Result` (unlike
  /// `surface::validate_size`) because a full `Result`-returning fix would need a new
  /// `crate::Error` variant analogous to `ZeroSizeSurface`, out of scope for this fix; see
  /// [`render_target_2d`]'s `# Panics` section for the resulting fail-fast, not fail-soft,
  /// treatment.
  #[ doc( hidden ) ]
  #[ must_use ]
  pub const fn is_nonzero_size( size : ( u32, u32 ) ) -> bool
  {
    let ( width, height ) = size;
    width != 0 && height != 0
  }

  /// Creates a 2D render-target [`Texture`] of the given `( width, height )` size and format.
  ///
  /// The texture is created with `RENDER_ATTACHMENT | COPY_SRC` usage, one mip level and
  /// no multisampling, so it can serve as a color attachment and later be copied out —
  /// e.g. by `readback::rgba8`. The bundled view and sampler are default-configured.
  ///
  /// # Panics
  /// Panics when either `size` component is `0`, with a clear crate-authored message --
  /// `wgpu::Device::create_texture` panics on a zero-sized `Extent3d` via its default
  /// uncaptured-error handler (BUG-276), several layers deeper than this crate, so this function
  /// asserts the precondition itself instead of letting the caller hit that opaque panic.
  #[ must_use ]
  pub fn render_target_2d( device : &wgpu::Device, size : ( u32, u32 ), format : wgpu::TextureFormat ) -> Texture
  {
    let ( width, height ) = size;
    assert!
    (
      is_nonzero_size( size ),
      "render_target_2d: width and height must both be non-zero, got {width}x{height} -- \
      wgpu::Device::create_texture panics on a zero-sized Extent3d"
    );
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
  own use is_nonzero_size;
}
