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
    /// Underlying native wgpu driver error.
    Native( String ),
    /// The requested operation or value is not supported by the active
    /// backend.
    Unsupported( String ),
    /// The caller-supplied descriptor is invalid independent of which
    /// backend is active ( e.g. a zero-sized texture dimension ) — rejected
    /// before any backend is touched.
    InvalidInput( String )
  }

  impl std::fmt::Display for Error
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      match self
      {
        Error::WebGpu( message ) => write!( f, "WebGPU backend error :: {message}" ),
        Error::WebGl( message ) => write!( f, "WebGL backend error :: {message}" ),
        Error::Native( message ) => write!( f, "Native backend error :: {message}" ),
        Error::Unsupported( message ) => write!( f, "Unsupported :: {message}" ),
        Error::InvalidInput( message ) => write!( f, "Invalid input :: {message}" )
      }
    }
  }

  impl std::error::Error for Error {}

  #[ cfg( all( feature = "webgpu", target_arch = "wasm32" ) ) ]
  impl From< minwebgpu::WebGPUError > for Error
  {
    fn from( error : minwebgpu::WebGPUError ) -> Self
    {
      Self::WebGpu( error.to_string() )
    }
  }

  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  impl From< minwebgl::WebglError > for Error
  {
    fn from( error : minwebgl::WebglError ) -> Self
    {
      Self::WebGl( error.to_string() )
    }
  }

  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  impl From< minwgpu::Error > for Error
  {
    fn from( error : minwgpu::Error ) -> Self
    {
      Self::Native( error.to_string() )
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
