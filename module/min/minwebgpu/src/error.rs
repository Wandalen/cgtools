/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Debug, error::typed::Error ) ]
  pub enum WebGPUError
  {
    #[ error( "Dom error :: {0}" ) ]
    DomError( #[ from ] crate::dom::Error ),
    #[ error( "Canvas error :: {0}" ) ]
    CanvasError( #[ from ] CanvasError ),
    #[ error( "Device error :: {0}" ) ]
    DeviceError( #[ from ] DeviceError ),
    #[ error( "Context error :: {0}" ) ]
    ContexError( #[ from ] ContextError ),
    #[ error( "Texture error :: {0}" ) ]
    TextureError( #[ from ] TextureError ),
    #[ error( "Buffer error :: {0}" ) ]
    BufferError( #[ from ] BufferError ),
  }


  #[ derive( Debug, error::typed::Error ) ]
  pub enum CanvasError
  {
    #[ error( "Failed to configure a canvas: {0}" )]
    ConfigurationError( String )
  }

  #[ derive( Debug, error::typed::Error ) ]
  pub enum ContextError
  {
    #[ error( "Failed to get current texture: {0}" )]
    FailedToGetCurrentTextureError( String )
  }

  #[ derive( Debug, error::typed::Error ) ]
  pub enum TextureError
  {
    #[ error( "Failed to create view for the texture: {0}" )]
    FailedToCreateView( String )
  }

  #[ derive( Debug, error::typed::Error ) ]
  pub enum BufferError
  {
    #[ error( "Failed to get mapped range: {0}" )]
    FailedToGetMappedRange( String ),
    #[ error( "Failed to write to the buffer: {0}" )]
    FailedWriteToBuffer( String ),
  }

  #[ derive( Debug, error::typed::Error ) ]
  pub enum DeviceError
  {
    #[ error( "Failed to create BindGroupLayout: {0}" )]
    FailedToCreateBindGroupLayout( String ),
    #[ error( "Failed to create RenderPipeline: {0}" )]
    FailedToCreateRenderPipeline( String ),
    #[ error( "Failed to create ComputePipeline: {0}" )]
    FailedToCreateComputePipeline( String ),
    #[ error( "Failed to create Texture: {0}" )]
    FailedToCreateTexture( String ),
    #[ error( "Failed to create Buffer: {0}" )]
    FailedToCreateBuffer( String )
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
    BufferError
  };
}


