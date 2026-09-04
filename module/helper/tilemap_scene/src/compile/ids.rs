//! `IdMap` — allocator that maps string ids from the scene-model format to
//! backend [`tilemap_renderer::types::ResourceId`] handles.
//!
//! The scene-model format uses string ids everywhere (`Asset.id`,
//! `Object.id`, etc.); backends expect numeric `ResourceId<T>` handles. The
//! compile layer assigns ids deterministically (sequential from 0 in
//! declaration order) so every compile of the same spec produces the same
//! handles — essential for testability and for later re-compiling without
//! confusing stateful backends.

mod private
{
  use rustc_hash::FxHashMap as HashMap;
  use tilemap_renderer::types::{ ResourceId, asset };

  /// Deterministic allocator for asset and sprite resource ids.
  ///
  /// Ids start at 0 and increase in the order in which they are requested.
  ///
  /// The two map fields are `pub(crate)` so the asset compile pass can
  /// iterate them in `compile/assets.rs`. External callers go through
  /// [`Self::image_alloc`] / [`Self::sprite_alloc`] / [`Self::image`] /
  /// [`Self::sprite`] so the next-id counters remain consistent with the
  /// recorded entries.
  #[ derive( Debug, Default ) ]
  pub struct IdMap
  {
    /// Asset string id → image resource id.
    pub( crate ) images : HashMap< String, ResourceId< asset::Image > >,
    /// `( asset_id, frame_name )` → sprite resource id.
    pub( crate ) sprites : HashMap< ( String, String ), ResourceId< asset::Sprite > >,
    next_image : u32,
    next_sprite : u32,
  }

  impl IdMap
  {
    /// Create an empty allocator.
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self { Self::default() }

    /// Allocate or return the existing image id for `asset_id`.
    ///
    /// # Panics
    ///
    /// Panics if the image id counter overflows `u32::MAX`.
    #[ inline ]
    pub fn image_alloc( &mut self, asset_id : &str ) -> ResourceId< asset::Image >
    {
      if let Some( id ) = self.images.get( asset_id ).copied()
      {
        return id;
      }
      let id = ResourceId::new( self.next_image );
      self.next_image = self.next_image.checked_add( 1 ).expect( "exceeded u32::MAX image resource ids" );
      self.images.insert( asset_id.to_owned(), id );
      id
    }

    /// Allocate or return the existing sprite id for a `(asset_id, frame_name)` pair.
    ///
    /// # Panics
    ///
    /// Panics if the sprite id counter overflows `u32::MAX`.
    #[ inline ]
    pub fn sprite_alloc
    (
      &mut self,
      asset_id : &str,
      frame_name : &str,
    ) -> ResourceId< asset::Sprite >
    {
      let key = ( asset_id.to_owned(), frame_name.to_owned() );
      if let Some( id ) = self.sprites.get( &key ).copied()
      {
        return id;
      }
      let id = ResourceId::new( self.next_sprite );
      self.next_sprite = self.next_sprite.checked_add( 1 ).expect( "exceeded u32::MAX sprite resource ids" );
      self.sprites.insert( key, id );
      id
    }

    /// Look up the image id for `asset_id`, if allocated.
    #[ inline ]
    #[ must_use ]
    pub fn image( &self, asset_id : &str ) -> Option< ResourceId< asset::Image > >
    {
      self.images.get( asset_id ).copied()
    }

    /// Look up the sprite id for `(asset_id, frame_name)`, if allocated.
    #[ inline ]
    #[ must_use ]
    pub fn sprite
    (
      &self,
      asset_id : &str,
      frame_name : &str,
    ) -> Option< ResourceId< asset::Sprite > >
    {
      // Avoid a temporary String allocation for the happy-path lookup by
      // hashing a tuple of borrows; std's HashMap needs the key type to
      // match exactly, so allocate a temporary tuple on lookup for now.
      // Compile isn't in a hot loop, so this is cheap.
      let key = ( asset_id.to_owned(), frame_name.to_owned() );
      self.sprites.get( &key ).copied()
    }
  }
}

mod_interface::mod_interface!
{
  exposed use IdMap;
}
