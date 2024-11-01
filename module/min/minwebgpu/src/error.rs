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
  }


  #[ derive( Debug, error::typed::Error ) ]
  pub enum CanvasError
  {
    #[ error( "Failed to configure a canvas: {0}" )]
    ConfigurationError( String )
  }

  #[ derive( Debug, error::typed::Error ) ]
  pub enum DeviceError
  {
    #[ error( "Failed to create BindGroupLayout: {0}" )]
    FailedToCreateBindGroupLayout( String ),
    #[ error( "Failed to create RenderPipeline: {0}" )]
    FailedToCreateRenderPipeline( String ),
    #[ error( "Failed to create Texture: {0}" )]
    FailedToCreateTexture( String )
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
    DeviceError
  };
}


