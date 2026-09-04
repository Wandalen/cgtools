//! Tests for `StorageTextureBindingLayout`'s default `format` value.
//!
//! `StorageTextureBindingLayout::new()` (also reachable via `binding_type::storage_texture_type()`)
//! must default `format` to a `GpuTextureFormat` that actually supports `STORAGE_BINDING` usage per
//! the WebGPU spec's texture format capability table — see `binding_type/storage_texture.rs`.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgpu as gl;
  use gl::web_sys::GpuTextureFormat;

  // test_kind: bug_reproducer(BUG-275)
  /// ## Root Cause
  /// `StorageTextureBindingLayout::new()` defaulted `format` to `GpuTextureFormat::Rgba8unormSrgb` —
  /// a value copy-pasted from the general-purpose texture descriptor's default
  /// (`descriptor/texture.rs`) and the render-target color-attachment default
  /// (`state/color_target.rs`), both of which legitimately default to an sRGB color format. Per the
  /// WebGPU spec's texture format capability table, `rgba8unorm-srgb` does NOT support
  /// `STORAGE_BINDING` usage — no `-srgb` format does, since storage texture reads/writes operate on
  /// raw texel values with no sRGB transfer function applied. Any `StorageTextureBindingLayout` left
  /// at its default (`.format(..)` never called) therefore produces a `GPUBindGroupLayoutEntry` that
  /// a real `GPUDevice.createBindGroupLayout` call rejects with a validation error.
  /// ## Why Not Caught
  /// No existing test ever converted a `StorageTextureBindingLayout` without first calling
  /// `.format(..)` explicitly — every prior caller set a concrete, valid format before converting, so
  /// the invalid default value was never read back or asserted against.
  /// ## Fix Applied
  /// Changed the default in `StorageTextureBindingLayout::new()` from
  /// `GpuTextureFormat::Rgba8unormSrgb` to `GpuTextureFormat::Rgba8unorm` — the non-sRGB counterpart,
  /// confirmed by the WebGPU spec's texture format capability table to support `STORAGE_BINDING`
  /// usage, and the closest same-channel-layout format to the original (evidently copy-pasted)
  /// intent.
  /// ## Prevention
  /// A default value copied from a sibling type's default must be re-checked against the *new*
  /// type's own spec constraints, never assumed to carry over unchanged — `format` has fundamentally
  /// different valid-value sets for a sampled/render texture (`descriptor/texture.rs`,
  /// `state/color_target.rs`) versus a storage texture binding (`binding_type/storage_texture.rs`);
  /// the two are not interchangeable despite sharing a field name and type.
  /// ## Pitfall
  /// sRGB texture formats (`*-srgb`) never support `STORAGE_BINDING` in the WebGPU spec — a default
  /// value that is valid for `TEXTURE_BINDING`/`RENDER_ATTACHMENT` usage is not automatically valid
  /// for `STORAGE_BINDING` usage; each usage class has its own format capability subset, and a
  /// "plausible-looking" shared default across binding-type files is exactly where that gets missed.
  #[ wasm_bindgen_test ]
  fn default_format_supports_storage_binding_test()
  {
    let layout : gl::web_sys::GpuStorageTextureBindingLayout =
      gl::binding_type::storage_texture_type().into();

    let format = layout.get_format();

    assert_ne!
    (
      format,
      GpuTextureFormat::Rgba8unormSrgb,
      "StorageTextureBindingLayout::new()'s default format must not be an sRGB format — sRGB \
      formats never support STORAGE_BINDING usage per the WebGPU spec's texture format capability \
      table"
    );
    assert_eq!
    (
      format,
      GpuTextureFormat::Rgba8unorm,
      "StorageTextureBindingLayout::new()'s default format must be a format that actually \
      supports STORAGE_BINDING usage"
    );
  }
}
