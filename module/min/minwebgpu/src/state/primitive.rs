/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuPrimitiveState`.
  #[ derive( Default, Clone ) ]
  pub struct PrimitiveState
  {
    /// The face culling mode.
    ///
    /// Culling is a performance optimization that discards primitives facing away
    /// from the camera. This field specifies whether to cull the front face, the back face,
    /// or to disable culling entirely.
    ///
    /// Defaults to `None`.
    cull_mode : Option< GpuCullMode >,
    /// The winding order that defines a "front-facing" primitive.
    ///
    /// This is used in conjunction with `cull_mode` to determine which primitives
    /// to discard. `Ccw` (counter-clockwise) is the standard for right-handed coordinate systems.
    ///
    /// Defaults to `GpuFrontFace::Ccw`.
    front_face : Option< GpuFrontFace >,
    /// The primitive topology.
    ///
    /// This specifies how the vertex data is assembled into primitives. Common
    /// options include `TriangleList`, `PointList`, and `LineList`.
    ///
    /// Defaults to `GpuPrimitiveTopology::TriangleList`.
    topology : Option< GpuPrimitiveTopology >,
    /// The index format for strip primitive topologies.
    ///
    /// This is required when the `topology` is `LineStrip` or `TriangleStrip` to
    /// specify the data type of the indices.
    strip_index_format : Option< GpuIndexFormat >,
    /// A flag to enable or disable unclipped depth.
    ///
    /// When `true`, depth values are not clamped to the `[0, 1]` range, which
    /// can be useful for certain rendering techniques like shadow mapping.
    ///
    /// Defaults to `false`.
    unclipped_depth : Option< bool >
  }

  impl  PrimitiveState 
  {
    /// Creates a new `PrimitiveState` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the cull mode to `None`, disabling culling.
    pub fn cull_none( mut self ) -> Self
    {
      self.cull_mode = Some( GpuCullMode::None );
      self
    }

    /// Sets the cull mode to `Front`, culling front-facing primitives.
    pub fn cull_front( mut self ) -> Self
    {
      self.cull_mode = Some( GpuCullMode::Front );
      self
    }

    /// Sets the cull mode to `Back`, culling back-facing primitives.
    pub fn cull_back( mut self ) -> Self
    {
      self.cull_mode = Some( GpuCullMode::Back );
      self
    }

    /// Sets the front face winding order to `Cw` (clockwise).
    pub fn cw( mut self ) -> Self
    {
      self.front_face = Some( GpuFrontFace::Cw );
      self
    }

    /// Enables unclipped depth.
    ///
    /// This is a convenience method that sets `unclipped_depth` to `Some(true)`.
    pub fn unclipped_depth( mut self ) -> Self
    {
      self.unclipped_depth = Some( true );
      self
    }

    /// Sets the primitive topology.
    pub fn topology( mut self, topology : GpuPrimitiveTopology ) -> Self
    {
      self.topology = Some( topology );
      self
    }

    /// Sets the topology to `PointList`.
    ///
    /// This is a convenience method for a common topology.
    pub fn points( mut self ) -> Self
    {
      self.topology = Some( GpuPrimitiveTopology::PointList );
      self
    }

    /// Sets the topology to `LineList`.
    pub fn lines( mut self ) -> Self
    {
      self.topology = Some( GpuPrimitiveTopology::LineList );
      self
    }

    /// Sets the topology to `TriangleList`.
    pub fn triangles( mut self ) -> Self
    {
      self.topology = Some( GpuPrimitiveTopology::TriangleList );
      self
    }

    /// Sets the topology to `LineStrip`.
    pub fn line_strip( mut self ) -> Self
    {
      self.topology = Some( GpuPrimitiveTopology::LineStrip );
      self
    }

    /// Sets the topology to `TriangleStrip`.
    pub fn triangle_strip( mut self ) -> Self
    {
      self.topology = Some( GpuPrimitiveTopology::TriangleStrip );
      self
    }
  }

  impl From< PrimitiveState > for web_sys::GpuPrimitiveState
  {
    fn from( value: PrimitiveState ) -> Self 
    {
      let state = web_sys::GpuPrimitiveState::new();

      if let Some( v ) = value.cull_mode { state.set_cull_mode( v ); }
      if let Some( v ) = value.front_face { state.set_front_face( v ); }
      if let Some( v ) = value.topology { state.set_topology( v ); }
      if let Some( v ) = value.strip_index_format { state.set_strip_index_format( v ); }
      if let Some( v ) = value.unclipped_depth { state.set_unclipped_depth( v ); }

      state
    }
  }
}

crate::mod_interface!
{

  exposed use
  {
    PrimitiveState
  };

}
