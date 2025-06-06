/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Default, Clone ) ]
  pub struct BufferBindingLayout
  {
    /// Defaults to `uniform`
    b_type : Option< GpuBufferBindingType >,
    /// Defaults to `false`
    has_dynamic_offset : Option< bool >,
    /// Default to `0`
    min_binding_size : Option< f64 >
  }

  impl BufferBindingLayout {
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the type of the buffer from the provided type
    pub fn set_type( mut self, b_type : GpuBufferBindingType ) -> Self
    {
      self.b_type = Some( b_type );
      self
    }

    /// Sets the type of the buffer to `Uniform`
    pub fn uniform( mut self ) -> Self
    {
      self.b_type = Some( GpuBufferBindingType::Uniform );
      self
    }

    /// Sets the type of the buffer to `Storage`
    pub fn storage( mut self ) -> Self
    {
      self.b_type = Some( GpuBufferBindingType::Storage );
      self
    }

    /// Sets the type of the buffer to `ReadOnlyStorage`
    pub fn storage_readonly( mut self ) -> Self
    {
      self.b_type = Some( GpuBufferBindingType::ReadOnlyStorage );
      self
    }

    /// Sets the property `has_dynamic_offset` of the buffer to `true`
    pub fn dynamic_offset( mut self ) -> Self
    {
      self.has_dynamic_offset = Some( true );
      self
    }

    /// Sets the property `min_binding_size` of the buffer to specified value
    pub fn min_binding_size( mut self, size : f64 ) -> Self
    {
      self.min_binding_size = Some( size );
      self
    }
  }

  impl From< BufferBindingLayout > for web_sys::GpuBufferBindingLayout
  {
    fn from( value: BufferBindingLayout ) -> Self 
    {
      let layout = web_sys::GpuBufferBindingLayout::new();

      if let Some( v ) = value.b_type { layout.set_type( v ); }
      if let Some( v ) = value.has_dynamic_offset { layout.set_has_dynamic_offset( v ); }
      if let Some( v ) = value.min_binding_size { layout.set_min_binding_size( v ); }

      layout
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    BufferBindingLayout
  };
}
