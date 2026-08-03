//! Shared test helpers for `tilemap_scene` integration tests.
//!
//! Importing pattern in each `tests/*.rs`:
//!
//! ```ignore
//! mod common;
//! use common::flatten_to_sprites;
//! ```
//!
//! Cargo does NOT treat `tests/common/mod.rs` as a test target on its
//! own — it only compiles into the test crates that explicitly `mod`
//! it in.

#![ allow( dead_code ) ]
#![ allow( clippy::min_ident_chars ) ]

use rustc_hash::FxHashMap as HashMap;
use tilemap_renderer::commands::
{
  AddSpriteInstance,
  RenderCommand,
  Sprite,
  SpriteBatchParams,
};
use tilemap_renderer::types::{ asset, Batch, ResourceId };

#[ derive( Clone, Copy ) ]
struct Instance
{
  transform : tilemap_renderer::types::Transform,
  sprite : ResourceId< asset::Sprite >,
  tint : [ f32; 4 ],
}

struct BatchState
{
  params : SpriteBatchParams,
  instances : Vec< Instance >,
}

/// Stateful flattener that mirrors what a real backend would do: it
/// tracks the live batch table across multiple `apply()` calls, so the
/// second render's `Bind` + `Set` + `Unbind` + `Draw` sequence
/// correctly resolves against the batch the first render created.
///
/// Use this when a test issues more than one `render()` call against
/// the same `Renderer` — the stateless free-function form
/// [`flatten_to_sprites`] only handles a single frame's command stream.
pub struct BatchFlattener
{
  batches : HashMap< ResourceId< Batch >, BatchState >,
  bound : Option< ResourceId< Batch > >,
}

impl BatchFlattener
{
  pub fn new() -> Self
  {
    Self { batches : HashMap::default(), bound : None }
  }

  /// Apply this frame's `cmds`, mutating the internal batch table, and
  /// return the equivalent per-sprite command stream.
  pub fn apply( &mut self, cmds : &[ RenderCommand ] ) -> Vec< RenderCommand >
  {
    let mut out : Vec< RenderCommand > = Vec::with_capacity( cmds.len() );

    for cmd in cmds
    {
      match cmd
      {
        RenderCommand::CreateSpriteBatch( c ) =>
        {
          self.batches.insert( c.batch, BatchState
          {
            params : c.params,
            instances : Vec::new(),
          });
        },
        RenderCommand::BindBatch( b ) => { self.bound = Some( b.batch ); },
        RenderCommand::UnbindBatch( _ ) => { self.bound = None; },
        RenderCommand::AddSpriteInstance( a ) =>
        {
          let id = self.bound.expect( "AddSpriteInstance outside Bind/Unbind" );
          let state = self.batches.get_mut( &id ).expect( "Add to unknown batch" );
          state.instances.push( Instance
          {
            transform : a.transform,
            sprite : a.sprite,
            tint : a.tint,
          });
        },
        RenderCommand::SetSpriteInstance( s ) =>
        {
          let id = self.bound.expect( "SetSpriteInstance outside Bind/Unbind" );
          let state = self.batches.get_mut( &id ).expect( "Set to unknown batch" );
          let i = s.index as usize;
          assert!( i < state.instances.len(), "SetSpriteInstance out-of-range" );
          state.instances[ i ] = Instance
          {
            transform : s.transform,
            sprite : s.sprite,
            tint : s.tint,
          };
        },
        RenderCommand::RemoveInstance( r ) =>
        {
          let id = self.bound.expect( "RemoveInstance outside Bind/Unbind" );
          let state = self.batches.get_mut( &id ).expect( "Remove from unknown batch" );
          let i = r.index as usize;
          assert!( i < state.instances.len(), "RemoveInstance out-of-range" );
          state.instances.swap_remove( i );
        },
        RenderCommand::DrawBatch( d ) =>
        {
          let state = self.batches.get( &d.batch ).expect( "Draw of unknown batch" );
          for inst in &state.instances
          {
            out.push( RenderCommand::Sprite( Sprite
            {
              transform : inst.transform,
              sprite : inst.sprite,
              tint : inst.tint,
              blend : state.params.blend,
              clip : state.params.clip,
            }));
          }
        },
        RenderCommand::DeleteBatch( d ) =>
        {
          self.batches.remove( &d.batch );
        },
        _ => out.push( *cmd ),
      }
    }

    out
  }
}

impl Default for BatchFlattener
{
  fn default() -> Self { Self::new() }
}

/// Single-shot flattener equivalent to `BatchFlattener::new().apply(cmds)`.
/// Use only for tests that issue one render call against a fresh
/// renderer; multi-render tests need the stateful form above.
pub fn flatten_to_sprites( cmds : &[ RenderCommand ] ) -> Vec< RenderCommand >
{
  BatchFlattener::new().apply( cmds )
}

/// Count `RenderCommand::Sprite` entries after a single-shot flatten.
/// For multi-render tests, route through [`BatchFlattener`] and count
/// the per-call output.
#[ allow( dead_code ) ]
pub fn flat_sprite_count( cmds : &[ RenderCommand ] ) -> usize
{
  flatten_to_sprites( cmds )
    .iter()
    .filter( | c | matches!( c, RenderCommand::Sprite( _ ) ) )
    .count()
}

/// Extract just the `Sprite` payloads (world-space) from a single-shot
/// flatten.
#[ allow( dead_code ) ]
pub fn flat_sprites( cmds : &[ RenderCommand ] ) -> Vec< Sprite >
{
  flatten_to_sprites( cmds )
    .into_iter()
    .filter_map( | c | if let RenderCommand::Sprite( s ) = c { Some( s ) } else { None } )
    .collect()
}

// Silence "unused" warnings for the `AddSpriteInstance` import when a
// test file pulls common/mod.rs in but doesn't reference all helpers.
#[ allow( dead_code ) ]
fn _silence_unused() { let _ = core::mem::size_of::< AddSpriteInstance >(); }
