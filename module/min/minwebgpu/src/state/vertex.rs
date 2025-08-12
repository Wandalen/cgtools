/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuVertexState`.
  #[ derive( Clone ) ]
  pub struct VertexState< 'a >
  {
    /// The required shader module containing the vertex shader code.
    ///
    /// This is the compiled representation of the WGSL vertex shader.
    module : &'a web_sys::GpuShaderModule,
    /// An optional name of the entry point function within the shader module.
    ///
    /// If not specified, the WebGPU implementation will typically look for a
    /// default entry point, like "main".
    ///
    /// Defaults to `None`.
    entry_point : Option< &'a str >,
    /// A list of vertex buffer layouts.
    ///
    /// Each `GpuVertexBufferLayout` describes the memory layout of a single
    /// vertex buffer, including its stride and attributes.
    buffers : Vec< web_sys::GpuVertexBufferLayout >
  }

  impl< 'a > VertexState< 'a > 
  {
    /// Creates a new `VertexState` builder with a required shader module.
    pub fn new( module : &'a web_sys::GpuShaderModule ) -> Self
    {
      let entry_point = None;
      let buffers = Vec::new();

      VertexState
      {
        module,
        entry_point,
        buffers
      }
    }

    /// Sets the entry point function name for the vertex shader.
    pub fn entry_point( mut self, entry : &'a str ) -> Self
    {
      self.entry_point = Some( entry );
      self
    }

    /// Adds a vertex buffer layout to the list of buffers.
    pub fn buffer( mut self, buffer : &web_sys::GpuVertexBufferLayout ) -> Self
    {
      self.buffers.push( buffer.clone() );
      self
    }
  }

  impl From< VertexState< '_ > > for web_sys::GpuVertexState 
  {
    fn from( value: VertexState< '_ > ) -> Self 
    {
      let state = web_sys::GpuVertexState::new( &value.module );

      if let Some( v ) = value.entry_point { state.set_entry_point( v ); }
      if !value.buffers.is_empty() { state.set_buffers( &value.buffers.into() ); }

      state
    }   
  }
}

crate::mod_interface!
{
  exposed use
  {
    VertexState
  };
}
