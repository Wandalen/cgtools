/// Internal namespace.
mod private
{
  use crate::*;

  pub struct VertexState< 'a >
  {
    module : &'a web_sys::GpuShaderModule,
    entry_point : Option< &'a str >,
    buffers : Vec< web_sys::GpuVertexBufferLayout >
  }

  impl< 'a > VertexState< 'a > 
  {
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

    pub fn entry_point( mut self, entry : &'a str ) -> Self
    {
      self.entry_point = Some( entry );
      self
    }

    pub fn buffer( mut self, buffer : web_sys::GpuVertexBufferLayout ) -> Self
    {
      self.buffers.push( buffer );
      self
    }

    pub fn buffers( mut self, buffers : &[ web_sys::GpuVertexBufferLayout ] ) -> Self
    {
      self.buffers.extend_from_slice( buffers );
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
