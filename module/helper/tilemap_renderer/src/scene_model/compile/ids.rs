//! `IdMap` — allocator that maps string ids from the scene-model format to
//! backend [`crate::types::ResourceId`] handles.
//!
//! The scene-model format uses string ids everywhere (`Asset.id`,
//! `Object.id`, etc.); backends expect numeric `ResourceId<T>` handles. The
//! compile layer assigns ids deterministically (sequential from 0 in
//! declaration order) so every compile of the same spec produces the same
//! handles — essential for testability and for later re-compiling without
//! confusing stateful backends.

mod private
{
  use std::collections::HashMap;
  use crate::types::{ ResourceId, asset };

  /// Deterministic allocator for asset and sprite resource ids.
  ///
  /// Ids start at 0 and increase in the order in which they are requested.
  #[ derive( Debug, Default ) ]
  pub struct IdMap
  {
    /// Asset string id → image resource id.
    pub images : HashMap< String, ResourceId< asset::Image > >,
    /// `( asset_id, frame_name )` → sprite resource id.
    pub sprites : HashMap< ( String, String ), ResourceId< asset::Sprite > >,
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
    #[ inline ]
    pub fn alloc_image( &mut self, asset_id : &str ) -> ResourceId< asset::Image >
    {
      if let Some( id ) = self.images.get( asset_id ).copied()
      {
        return id;
      }
      let id = ResourceId::new( self.next_image );
      self.next_image = self.next_image.wrapping_add( 1 );
      self.images.insert( asset_id.to_owned(), id );
      id
    }

    /// Allocate or return the existing sprite id for a `(asset_id, frame_name)` pair.
    #[ inline ]
    pub fn alloc_sprite
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
      self.next_sprite = self.next_sprite.wrapping_add( 1 );
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

#[ cfg( test ) ]
mod tests
{
  use super::private::*;

  #[ test ]
  fn images_are_deterministic()
  {
    let mut m = IdMap::new();
    let a = m.alloc_image( "terrain_atlas" );
    let b = m.alloc_image( "transitions_atlas" );
    let a_again = m.alloc_image( "terrain_atlas" );
    assert_eq!( a.inner(), 0 );
    assert_eq!( b.inner(), 1 );
    assert_eq!( a, a_again, "re-allocating same id returns the same handle" );
  }

  #[ test ]
  fn sprites_namespace_by_atlas()
  {
    let mut m = IdMap::new();
    let grass = m.alloc_sprite( "terrain", "0" );
    let sand  = m.alloc_sprite( "terrain", "1" );
    let grass_other_atlas = m.alloc_sprite( "other", "0" );
    assert_ne!( grass, sand, "different frames get different ids" );
    assert_ne!( grass, grass_other_atlas, "same frame name in different atlases is distinct" );
    assert_eq!( Some( grass ), m.sprite( "terrain", "0" ) );
  }
}

mod_interface::mod_interface!
{
  own use IdMap;
}
