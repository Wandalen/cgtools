/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Default, Clone ) ]
  pub struct BlendComponent
  {
    /// Defaults to `One`
    src_factor : Option< web_sys::GpuBlendFactor >,
    /// Defaults to `Zero`
    dst_factor : Option< web_sys::GpuBlendFactor >,
    /// Defaults to `Add`
    operation : Option< web_sys::GpuBlendOperation >
  }

  impl BlendComponent
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn src_factor( mut self, factor : GpuBlendFactor ) -> Self
    {
      self.src_factor = Some( factor );
      self
    }

    pub fn dst_factor( mut self, factor : GpuBlendFactor ) -> Self
    {
      self.dst_factor = Some( factor );
      self
    }
    pub fn operation( mut self, operation : GpuBlendOperation ) -> Self
    {
      self.operation = Some( operation );
      self
    }
  }

  impl From< BlendComponent > for web_sys::GpuBlendComponent
  {
    fn from( value: BlendComponent ) -> Self 
    {
      let c = web_sys::GpuBlendComponent::new();    

      if let Some( v ) = value.src_factor { c.set_src_factor( v ); }
      if let Some( v ) = value.dst_factor { c.set_dst_factor( v ); }
      if let Some( v ) = value.operation { c.set_operation( v ); }

      c
    }
  }

  #[ derive( Default, Clone ) ]
  pub struct BlendState
  {
    alpha : BlendComponent,
    color : BlendComponent
  }

  impl BlendState 
  {
    pub fn new() -> Self
    {
      Self::default()
    }    

    pub fn alpha( mut self, alpha : BlendComponent ) -> Self
    {
      self.alpha = alpha;
      self
    }

    pub fn color( mut self, color : BlendComponent ) -> Self
    {
      self.color = color;
      self
    }
  }

  impl From< BlendState > for web_sys::GpuBlendState
  {
    fn from( value: BlendState ) -> Self 
    {
      let state = web_sys::GpuBlendState::new
      ( 
        &value.alpha.into(), 
        &value.color.into()
      );    

      state
    }
  }
}

crate::mod_interface!
{

  exposed use
  {
    BlendState,
    BlendComponent
  };
  
}
