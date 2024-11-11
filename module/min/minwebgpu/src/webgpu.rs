/// Internal namespace.
mod private
{
  pub const COPY_BUFFER_ALIGNMENT: u64 = 4;

  pub use web_sys::GpuCanvasContext as GL;
  pub use web_sys::gpu_color_write as ColorWrite;
  pub use web_sys::gpu_buffer_usage as BufferUsage;
  pub use web_sys::gpu_shader_stage as ShaderStage;
  pub use web_sys::gpu_texture_usage as TextureUsage;

  pub use web_sys::GpuIndexFormat;
  pub use web_sys::GpuStencilOperation;
  pub use web_sys::GpuTextureFormat;
  pub use web_sys::GpuCompareFunction;
  pub use web_sys::GpuPrimitiveTopology;
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
  pub use web_sys::GpuLoadOp;
  pub use web_sys::GpuStoreOp;
  pub use web_sys::GpuCullMode;
  pub use web_sys::GpuFrontFace;
}

crate::mod_interface!
{

  orphan use super::private::*;
  
}
