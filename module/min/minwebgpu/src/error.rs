/// Internal namespace.
mod private
{
  use crate::*;

  #[ allow( missing_docs ) ]
  #[ derive( Debug, error::typed::Error ) ]
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
  }

  #[ allow( missing_docs ) ]
  #[ derive( Debug, error::typed::Error ) ]
  pub enum CanvasError
  {
    /// Indicates a failure to configure the canvas for WebGPU.
    #[ error( "Failed to configure a canvas: {0}" )]
    ConfigurationError( String )
  }

  #[ allow( missing_docs ) ]
  #[ derive( Debug, error::typed::Error ) ]
  pub enum ContextError
  {
    /// Indicates a failure to get the current texture from the context.
    #[ error( "Failed to get current texture: {0}" )]
    FailedToGetCurrentTextureError( String )
  }

  #[ allow( missing_docs ) ]
  #[ derive( Debug, error::typed::Error ) ]
  pub enum TextureError
  {
    /// Indicates a failure to create a view for a texture.
    #[ error( "Failed to create view for the texture: {0}" )]
    FailedToCreateView( String )
  }

  #[ allow( missing_docs ) ]
  #[ derive( Debug, error::typed::Error ) ]
  pub enum BufferError
  {
    /// Indicates a failure to get a mapped range of a buffer.
    #[ error( "Failed to get mapped range: {0}" )]
    FailedToGetMappedRange( String ),
    /// Indicates a failure to write data to a buffer.
    #[ error( "Failed to write to the buffer: {0}" )]
    FailedWriteToBuffer( String ),
  }

  #[ allow( missing_docs ) ]
  #[ derive( Debug, error::typed::Error ) ]
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
  #[ allow( missing_docs ) ]
  #[ derive( Debug, error::typed::Error ) ]
  pub enum BindGroupError
  {
    /// Indicates a `BindGroupLayoutEntry` was converted to its `web_sys` representation
    /// without ever calling `.ty(..)` to set its binding type — it is still the default,
    /// unset `BindingType::Other` placeholder.
    #[ error( "BindGroupLayoutEntry at binding {0} has no type set: call `.ty(..)` before conversion" )]
    TypeNotSet( u32 )
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
    BindGroupError
  };
}


