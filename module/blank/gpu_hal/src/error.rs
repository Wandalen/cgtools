mod private
{
  /// Unified error of every backend operation.
  #[ derive( Debug ) ]
  pub enum Error
  {
    /// Underlying WebGPU driver error.
    WebGpu( String ),
    /// Underlying WebGL driver error.
    WebGl( String ),
    /// The requested operation or value is not supported by the active
    /// backend.
    Unsupported( String )
  }

  impl std::fmt::Display for Error
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      match self
      {
        Error::WebGpu( message ) => write!( f, "WebGPU backend error :: {message}" ),
        Error::WebGl( message ) => write!( f, "WebGL backend error :: {message}" ),
        Error::Unsupported( message ) => write!( f, "Unsupported :: {message}" )
      }
    }
  }

  impl std::error::Error for Error {}

  #[ cfg( feature = "webgpu" ) ]
  impl From< minwebgpu::WebGPUError > for Error
  {
    fn from( error : minwebgpu::WebGPUError ) -> Self
    {
      Self::WebGpu( error.to_string() )
    }
  }

  #[ cfg( feature = "webgl" ) ]
  impl From< minwebgl::WebglError > for Error
  {
    fn from( error : minwebgl::WebglError ) -> Self
    {
      Self::WebGl( error.to_string() )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Error
  };
}
