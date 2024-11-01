/// Internal namespace.
mod private
{
  pub use web_sys::GpuCanvasContext as GL;
  pub use web_sys::GpuIndexFormat;
  pub use web_sys::GpuStencilOperation;
  pub use web_sys::GpuTextureFormat;
  pub use web_sys::GpuCompareFunction;
  pub use web_sys::GpuPrimitiveTopology;
  pub use web_sys::gpu_color_write as GpuColorWrite;
  pub use web_sys::GpuBlendFactor;
  pub use web_sys::GpuBlendOperation;
  pub use web_sys::GpuTextureDimension;
  pub use web_sys::GpuAddressMode;
  pub use web_sys::GpuFilterMode;
  pub use web_sys::GpuMipmapFilterMode;
  pub use web_sys::GpuBufferBindingType;
  pub use web_sys::GpuSamplerBindingType;
  pub use web_sys::GpuTextureSampleType;
  pub use web_sys::GpuTextureViewDimension;
  pub use web_sys::GpuStorageTextureAccess;
  pub use web_sys::GpuVertexFormat;
  pub use web_sys::GpuVertexStepMode;
}

crate::mod_interface!
{

  orphan use super::private::*;
  
}
