//! [`RenderTargetId`] and the slot table behind it: the generation-checked
//! handles [`WebGlBackend`]( crate::adapters::webgl::WebGlBackend ) hands out for its offscreen cache surfaces.
//! GL-free, so the stale-handle rule is tested on the host
//! ( `tests/webgl_render_target_test.rs` ).

mod private
{
  /// Handle to an offscreen cache surface, returned by
  /// [`WebGlBackend::create_render_target`]( crate::adapters::webgl::WebGlBackend::create_render_target ).
  ///
  /// Besides the surface's slot it records which generation of surfaces it
  /// belongs to. [`WebGlBackend::reset_render_targets`]( crate::adapters::webgl::WebGlBackend::reset_render_targets ) starts a new
  /// generation and reuses the slots, so a handle kept across a reset is
  /// rejected (the methods taking it do nothing) instead of silently
  /// addressing whichever new surface took its slot.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
  pub struct RenderTargetId
  {
    index : u32,
    generation : u32,
  }

  /// Slot table behind [`RenderTargetId`]: dense slots plus the generation
  /// stamped into each handle. GL-free so the stale-handle rule is testable
  /// on the host.
  pub struct TargetSlots< T >
  {
    items : Vec< T >,
    generation : u32,
  }

  impl< T > TargetSlots< T >
  {
    /// An empty table at generation 0.
    #[ must_use ]
    pub const fn new() -> Self
    {
      Self { items : Vec::new(), generation : 0 }
    }

    /// Store `item` and return its handle; `None` only past `u32::MAX` slots.
    pub fn push( &mut self, item : T ) -> Option< RenderTargetId >
    {
      let index = u32::try_from( self.items.len() ).ok()?;
      self.items.push( item );
      Some( RenderTargetId { index, generation : self.generation } )
    }

    /// The item `id` names, or `None` if `id` predates the last [`Self::clear`].
    #[ must_use ]
    pub fn get( &self, id : RenderTargetId ) -> Option< &T >
    {
      if id.generation != self.generation { return None; }
      self.items.get( id.index as usize )
    }

    /// Drop every item and start a new generation, so no earlier handle
    /// resolves even once its slot is reused.
    pub fn clear( &mut self )
    {
      self.items.clear();
      self.generation = self.generation.wrapping_add( 1 );
    }
  }

  impl< T > Default for TargetSlots< T >
  {
    fn default() -> Self { Self::new() }
  }
}

mod_interface::mod_interface!
{
  own use RenderTargetId;
  own use TargetSlots;
}

