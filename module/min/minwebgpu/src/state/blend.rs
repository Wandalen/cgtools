/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuBlendComponent`.
  #[ derive( Default, Clone ) ]
  pub struct BlendComponent
  {
    /// The blend factor for the source color.
    ///
    /// This factor is multiplied with the source color (the output of the
    /// fragment shader) before the blend operation is applied.
    ///
    /// Defaults to `GpuBlendFactor::One`.
    src_factor : Option< web_sys::GpuBlendFactor >,
    /// The blend factor for the destination color.
    ///
    /// This factor is multiplied with the destination color (the value already
    /// in the render target) before the blend operation is applied.
    ///
    /// Defaults to `GpuBlendFactor::Zero`.
    dst_factor : Option< web_sys::GpuBlendFactor >,
    /// The blend operation to perform on the source and destination colors.
    ///
    /// This defines how the factored source and destination colors are combined.
    /// Common operations include `Add`, `Subtract`, `ReverseSubtract`, etc.
    ///
    /// Defaults to `GpuBlendOperation::Add`.
    operation : Option< web_sys::GpuBlendOperation >
  }

  impl BlendComponent
  {
    /// Creates a new `BlendComponent` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the source blend factor.
    pub fn src_factor( mut self, factor : GpuBlendFactor ) -> Self
    {
      self.src_factor = Some( factor );
      self
    }

    /// Sets the destination blend factor.
    pub fn dst_factor( mut self, factor : GpuBlendFactor ) -> Self
    {
      self.dst_factor = Some( factor );
      self
    }

    /// Sets the blend operation.
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

  /// A builder for creating a `web_sys::GpuBlendState`.
  #[ derive( Default, Clone ) ]
  pub struct BlendState
  {
    /// The blending configuration for the alpha channel.
    alpha : BlendComponent,
    /// The blending configuration for the color channels (red, green, and blue).
    color : BlendComponent
  }

  impl BlendState 
  {
    /// Creates a new `BlendState` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }    

    /// Sets the blending configuration for the alpha channel.
    pub fn alpha( mut self, alpha : BlendComponent ) -> Self
    {
      self.alpha = alpha;
      self
    }

    /// Sets the blending configuration for the color channels.
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
