//! Tests for `TextureDescriptor`'s default `format` value.
//!
//! `TextureDescriptor::new()` must default `format` to a `GpuTextureFormat` that stays valid
//! across every usage flag this builder can produce — including `.storage_binding()` — per the
//! WebGPU spec's texture format capability table. See `descriptor/texture.rs`.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgpu as gl;
  use gl::web_sys::GpuTextureFormat;

  // BUG-300 task/bug/300_texture_descriptor_default_format_not_storage_capable.md -- reproducer
  // for `TextureDescriptor::new()`'s default format being incompatible with `.storage_binding()`.
  /// ## Root Cause
  /// `TextureDescriptor::new()` defaulted `format` to `GpuTextureFormat::Rgba8unormSrgb`. That
  /// default is valid for this builder's `TEXTURE_BINDING`/`RENDER_ATTACHMENT`/`COPY_SRC`/
  /// `COPY_DST` usage flags, but per the WebGPU spec's texture format capability table, no
  /// `-srgb` format supports `STORAGE_BINDING` usage. A caller chaining `.storage_binding()`
  /// without an explicit `.format(..)` override therefore produced a `GPUTextureDescriptor` a
  /// real `GPUDevice.createTexture` call rejects — but only via an async device error-scope
  /// event, never a synchronous throw, so `texture::create`'s `.map_err(..)` (which only catches
  /// synchronous throws) silently returned `Ok` for an unusable texture.
  /// ## Why Not Caught
  /// No existing test ever converted a `TextureDescriptor` without first calling `.format(..)`
  /// explicitly, and nothing in this workspace calls `.storage_binding()` on this builder yet —
  /// the invalid default was never read back or asserted against.
  /// ## Fix Applied
  /// Changed the default in `TextureDescriptor::new()` from `GpuTextureFormat::Rgba8unormSrgb`
  /// to `GpuTextureFormat::Rgba8unorm` — the non-sRGB counterpart, valid for every usage flag
  /// this builder can produce (including `STORAGE_BINDING`) per the WebGPU spec's texture format
  /// capability table.
  /// ## Prevention
  /// A default shared across every usage flag a builder can produce must be valid for the
  /// narrowest usage class among them, not just the most common one — re-check a shared default
  /// against each usage flag's own spec constraints before trusting it covers all of them.
  /// ## Pitfall
  /// sRGB texture formats (`*-srgb`) never support `STORAGE_BINDING` in the WebGPU spec — a
  /// default that is valid for `TEXTURE_BINDING`/`RENDER_ATTACHMENT` usage is not automatically
  /// valid once `.storage_binding()` is also chained onto the same builder.
  // test_kind: bug_reproducer(BUG-300)
  #[ wasm_bindgen_test ]
  fn default_format_supports_storage_binding_test()
  {
    let descriptor : gl::web_sys::GpuTextureDescriptor =
      gl::TextureDescriptor::new().storage_binding().into();

    let format = descriptor.get_format();

    assert_ne!
    (
      format,
      GpuTextureFormat::Rgba8unormSrgb,
      "TextureDescriptor::new()'s default format must not be an sRGB format — sRGB formats \
      never support STORAGE_BINDING usage per the WebGPU spec's texture format capability table"
    );
    assert_eq!
    (
      format,
      GpuTextureFormat::Rgba8unorm,
      "TextureDescriptor::new()'s default format must be a format that actually supports \
      STORAGE_BINDING usage"
    );
  }
}
