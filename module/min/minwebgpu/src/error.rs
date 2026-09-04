/// Internal namespace.
mod private
{
  use error_tools::{ dependency::thiserror, error };

  /// The top-level error type unifying every WebGPU operation failure exposed by this crate.
  #[ derive( Debug, error::typed::Error ) ]
  #[ non_exhaustive ]
  pub enum WebGPUError
  {
    /// This indicates an error with the web browser's Document Object Model.
    #[ error( "Dom error :: {0}" ) ]
    DomError( #[ from ] crate::dom::Error ),
    /// An error related to the HTML canvas element.
    #[ error( "Canvas error :: {0}" ) ]
    CanvasError( #[ from ] CanvasError ),
    /// An error related to the WebGPU device.
    #[ error( "Device error :: {0}" ) ]
    DeviceError( #[ from ] DeviceError ),
    /// An error related to the WebGPU context.
    #[ error( "Context error :: {0}" ) ]
    ContexError( #[ from ] ContextError ),
    /// An error related to WebGPU textures.
    #[ error( "Texture error :: {0}" ) ]
    TextureError( #[ from ] TextureError ),
    /// An error related to WebGPU buffers.
    #[ error( "Buffer error :: {0}" ) ]
    BufferError( #[ from ] BufferError ),
    /// An error related to WebGPU bind group layout entries.
    #[ error( "BindGroup error :: {0}" ) ]
    BindGroupError( #[ from ] BindGroupError ),
    /// An error related to WebGPU render passes.
    #[ error( "RenderPass error :: {0}" ) ]
    RenderPassError( #[ from ] RenderPassError ),
  }

  /// Errors that can occur while configuring a WebGPU canvas context.
  #[ derive( Debug, error::typed::Error ) ]
  #[ non_exhaustive ]
  pub enum CanvasError
  {
    /// Indicates a failure to configure the canvas for WebGPU.
    #[ error( "Failed to configure a canvas: {0}" )]
    ConfigurationError( String )
  }

  /// Errors that can occur while acquiring a WebGPU adapter/device, or while retrieving state
  /// from an already-configured WebGPU canvas context.
  #[ derive( Debug, error::typed::Error ) ]
  #[ non_exhaustive ]
  pub enum ContextError
  {
    /// Indicates a failure to get the current texture from the context.
    #[ error( "Failed to get current texture: {0}" )]
    FailedToGetCurrentTextureError( String ),
    /// Indicates `navigator.gpu.requestAdapter()` resolved to `null` — a spec-defined outcome
    /// meaning no compatible `GPUAdapter` is available on this system, not a JS exception.
    #[ error( "No WebGPU adapter available on this system" )]
    NoAdapterAvailable,
    /// Indicates `GPUAdapter.requestDevice()`'s returned promise was rejected.
    #[ error( "WebGPU device request was rejected: {0}" )]
    DeviceRequestRejected( String ),
    /// Indicates `navigator.gpu` itself is absent -- this browser has no WebGPU support at all,
    /// as distinct from [`ContextError::NoAdapterAvailable`]'s "supported but no compatible
    /// adapter" outcome.
    #[ error( "WebGPU is not supported by this browser (navigator.gpu is undefined)" )]
    WebGpuUnsupported
  }

  /// Errors that can occur while creating a view of a WebGPU texture.
  #[ derive( Debug, error::typed::Error ) ]
  #[ non_exhaustive ]
  pub enum TextureError
  {
    /// Indicates a failure to create a view for a texture.
    #[ error( "Failed to create view for the texture: {0}" )]
    FailedToCreateView( String ),
    /// Indicates a failure to write data to a texture.
    #[ error( "Failed to write to the texture: {0}" )]
    FailedWriteToTexture( String ),
  }

  /// Errors that can occur while mapping or writing to a WebGPU buffer.
  #[ derive( Debug, error::typed::Error ) ]
  #[ non_exhaustive ]
  pub enum BufferError
  {
    /// Indicates a failure to get a mapped range of a buffer.
    #[ error( "Failed to get mapped range: {0}" )]
    FailedToGetMappedRange( String ),
    /// Indicates a failure to write data to a buffer.
    #[ error( "Failed to write to the buffer: {0}" )]
    FailedWriteToBuffer( String ),
  }

  /// Errors that can occur while asking a WebGPU device to create a GPU resource.
  #[ derive( Debug, error::typed::Error ) ]
  #[ non_exhaustive ]
  pub enum DeviceError
  {
    /// Indicates a failure to create a `BindGroupLayout`.
    #[ error( "Failed to create BindGroupLayout: {0}" )]
    FailedToCreateBindGroupLayout( String ),
    /// Indicates a failure to create a `RenderPipeline`.
    #[ error( "Failed to create RenderPipeline: {0}" )]
    FailedToCreateRenderPipeline( String ),
    /// Indicates a failure to create a `ComputePipeline`.
    #[ error( "Failed to create ComputePipeline: {0}" )]
    FailedToCreateComputePipeline( String ),
    /// Indicates a failure to create a `Texture`.
    #[ error( "Failed to create Texture: {0}" )]
    FailedToCreateTexture( String ),
    /// Indicates a failure to create a `Buffer`.
    #[ error( "Failed to create Buffer: {0}" )]
    FailedToCreateBuffer( String )
  }

  // Fix(BUG-051): new variant carrying the case that used to panic in
  // `BindGroupLayoutEntry`'s `web_sys` conversion (`descriptor/bind_group_layout_entry.rs`)
  // instead of returning an error.
  // Root cause: no existing `WebGPUError` variant represented "binding type never set" —
  // the conversion had to panic because there was nowhere to route a proper error.
  // Pitfall: an umbrella error enum with only browser/FFI-failure variants (all carrying a
  // JS-originated `String`) has no natural home for a caller-side "you forgot to configure
  // this" error — don't force such cases to panic for lack of a matching variant, add one.
  /// Errors that can occur while building a WebGPU bind group layout entry.
  #[ derive( Debug, error::typed::Error ) ]
  #[ non_exhaustive ]
  pub enum BindGroupError
  {
    /// Indicates a `BindGroupLayoutEntry` was converted to its `web_sys` representation
    /// without ever calling `.ty(..)` to set its binding type — it is still the default,
    /// unset `BindingType::Other` placeholder.
    #[ error( "BindGroupLayoutEntry at binding {0} has no type set: call `.ty(..)` before conversion" )]
    TypeNotSet( u32 )
  }

  /// Errors that can occur while beginning a WebGPU render pass.
  #[ derive( Debug, error::typed::Error ) ]
  #[ non_exhaustive ]
  pub enum RenderPassError
  {
    /// Indicates a failure to begin a render pass on a command encoder.
    #[ error( "Failed to begin render pass: {0}" )]
    FailedToBegin( String )
  }

}

crate::mod_interface!
{
  reuse ::mingl::error;

  exposed use
  {
    WebGPUError
  };

  orphan use
  {
    CanvasError,
    DeviceError,
    ContextError,
    TextureError,
    BufferError,
    BindGroupError,
    RenderPassError
  };
}


