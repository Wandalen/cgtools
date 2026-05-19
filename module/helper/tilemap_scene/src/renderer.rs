//! Retained-mode renderer — the algorithm half of [Path A][crate].
//!
//! [`Renderer`] is the sole entry point for turning a [`crate::scene::Scene`]
//! into a backend-ready `RenderCommand` stream. It owns the
//! [`crate::compile::CompiledAssets`] (built once in [`Renderer::new`]) and
//! re-uses internal scratch buffers across frames so a steady-state render
//! loop allocates nothing.
//!
//! ```ignore
//! let renderer = Renderer::new( &spec, &PathResolver )?;
//! backend.load_assets( renderer.assets() );
//!
//! // Per frame:
//! let cmds = renderer.render( &scene, &camera )?;
//! backend.submit( cmds );
//! ```
//!
//! **Multi-renderer / multi-backend.** Each renderer is an independent
//! algorithm instance: two `Renderer`s built from the same spec can
//! render the same `Scene` into different backends simultaneously
//! (e.g. WebGL plus a headless test backend) without state crosstalk.

mod private
{
  use core::mem::Discriminant;
  use rustc_hash::FxHashMap as HashMap;
  use tilemap_renderer::assets::Assets;
  use tilemap_renderer::commands::
  {
    AddSpriteInstance,
    BindBatch,
    Clear,
    CreateSpriteBatch,
    DeleteBatch,
    DrawBatch,
    RemoveInstance,
    RenderCommand,
    SetSpriteInstance,
    Sprite,
    SpriteBatchParams,
    UnbindBatch,
  };
  use tilemap_renderer::types::
  {
    asset,
    Batch,
    BlendMode,
    ResourceId,
    Transform,
  };
  use crate::compile::assets::{ CompiledAssets, compile_assets };
  use crate::compile::camera::Camera;
  use crate::compile::error::CompileError;
  use crate::compile::frame::gather_frame_emits;
  use crate::compile::resolver::AssetResolver;
  use crate::pipeline::SortMode;
  use crate::scene::Scene;
  use crate::spec::RenderSpec;

  /// Bit-equal fingerprint of a [`Camera`] used by [`Renderer`]'s per-frame
  /// cache to detect "the camera moved since the last render" without
  /// reaching into the camera struct's fields each call.
  ///
  /// `f32` fields are compared via their `to_bits` representation so
  /// `-0.0 == 0.0` and `NaN != NaN` (matches what bit-equal replay
  /// expects — any NaN treated as "different camera").
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  struct CameraSignature
  {
    world_center : ( u32, u32 ),
    zoom : u32,
    viewport_size : ( u32, u32 ),
  }

  impl CameraSignature
  {
    fn from( camera : &Camera ) -> Self
    {
      Self
      {
        world_center : ( camera.world_center.0.to_bits(), camera.world_center.1.to_bits() ),
        zoom : camera.zoom.to_bits(),
        viewport_size : camera.viewport_size,
      }
    }
  }

  /// Identity of a sprite batch: world-space sprites in the same bucket
  /// that share the same backing image sheet, blend mode, and clip mask
  /// can be drawn as one instanced batch on the GPU.
  ///
  /// `bucket_idx` is included so the same `(sheet, blend, clip)` triple
  /// emitted from two different pipeline buckets gets two distinct
  /// batches — preserves the per-bucket draw order the consumer
  /// declared in `RenderSpec.pipeline.layers`.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
  struct BatchKey
  {
    bucket_idx : u32,
    sheet : ResourceId< asset::Image >,
    /// `BlendMode` itself has no `Hash` / `Eq` derives upstream; the
    /// discriminant captures variant identity (sufficient since the
    /// enum carries no per-variant data today).
    blend : Discriminant< BlendMode >,
    clip : Option< ResourceId< asset::ClipMask > >,
  }

  /// Cached state of one allocated GPU batch, kept alive across renders
  /// so a mutation only emits the diff against `instances`.
  struct BatchEntry
  {
    id : ResourceId< Batch >,
    /// Last-emitted instance buffer — used by the batching renderer to
    /// compute the minimal Set / Add / Remove sequence on the next
    /// non-cached render.
    instances : Vec< Sprite >,
  }

  /// Retained-mode renderer.
  ///
  /// Holds the compiled asset table plus reusable per-frame buffers.
  /// See the module-level docs for the typical use pattern.
  pub struct Renderer
  {
    compiled : CompiledAssets,
    /// Output command buffer — held across frames so that an idle frame
    /// (no scene / camera / clock change) can return the previously
    /// emitted slice verbatim without re-running the per-frame walk.
    /// On a cache miss the buffer is cleared and refilled.
    cmd_buf : Vec< RenderCommand >,

    // ──────────────────────────────────────────────────────────────────
    // Per-renderer delta cache — Step 4.
    //
    // The retained-mode `Scene` exposes a monotonic `revision()` counter
    // bumped on every mutation. Combined with the master clock and a
    // bit-equal camera fingerprint, we get a 3-tuple that fully
    // characterises the rendered output. If all three match the values
    // captured on the previous `render()` call, the cached `cmd_buf` is
    // still valid and we hand it back without re-walking the scene.
    // ──────────────────────────────────────────────────────────────────
    /// Snapshot of `scene.revision()` at the most recent successful render.
    last_scene_revision : u64,
    /// Snapshot of `scene.clock()` at the most recent successful render.
    last_clock : f32,
    /// Bit-equal fingerprint of the camera used for the most recent render.
    last_camera_signature : CameraSignature,
    /// `false` until the first successful `render()` populates the cache;
    /// guards against returning an empty `cmd_buf` as a "cached" replay.
    has_rendered : bool,
    /// Number of `render()` calls served from the cache without
    /// re-walking the scene. Useful introspection for consumers
    /// monitoring redraw effectiveness, and gives tests a deterministic
    /// signal that the cache path was taken.
    cache_hits : u64,

    // ──────────────────────────────────────────────────────────────────
    // Sprite-batch state — Step 4b.
    //
    // `SortMode::None` buckets (terrain / vertex / edge — the bulk of
    // hex count in a typical Slay map) get collapsed into instanced
    // batches grouped by `(sheet, blend, clip)`. Each batch survives
    // across frames so a per-instance mutation emits one
    // `SetSpriteInstance` rather than a full rebuild. Sorted buckets
    // continue emitting per-sprite `Sprite` commands (the alternative
    // — batched repopulate — preserves visual order only when the
    // bucket fits in a single sheet, which we can't guarantee).
    // ──────────────────────────────────────────────────────────────────
    /// Live batches keyed by `(bucket_idx, sheet, blend, clip)`. Entries
    /// are added on first encounter and removed via `DeleteBatch` when
    /// a render no longer emits any instance for that key.
    batches : HashMap< BatchKey, BatchEntry >,
    /// Monotonic id allocator for `ResourceId<Batch>`. Caller-supplied
    /// ids per `tilemap_renderer/src/commands.rs:276-484`; never reused.
    next_batch_id : u32,
    /// `sprite_id → sheet_id` lookup built once at construction from
    /// `compiled.assets.sprites`. Lets the batch grouping walk derive
    /// each emitted `Sprite`'s batch key in O(1) instead of scanning
    /// the sprites vector per emit.
    sprite_to_sheet : HashMap< ResourceId< asset::Sprite >, ResourceId< asset::Image > >,
  }

  impl Renderer
  {
    /// Build a renderer for `spec`. Walks every reachable sprite under
    /// the spec's objects and animations, allocates resource ids, and
    /// resolves frame regions.
    ///
    /// # Errors
    ///
    /// - [`CompileError::AssetResolution`] when the resolver rejects an asset.
    /// - [`CompileError::UnsupportedAssetKind`] when an asset uses a kind
    ///   not yet implemented.
    /// - [`CompileError::UnresolvedRef`] / [`CompileError::InvalidFrameName`]
    ///   when a sprite reference cannot be resolved.
    pub fn new
    (
      spec : &RenderSpec,
      resolver : &impl AssetResolver,
    ) -> Result< Self, CompileError >
    {
      let compiled = compile_assets( spec, resolver )?;
      let mut sprite_to_sheet : HashMap< ResourceId< asset::Sprite >, ResourceId< asset::Image > > =
        HashMap::default();
      for sprite in &compiled.assets.sprites
      {
        sprite_to_sheet.insert( sprite.id, sprite.sheet );
      }
      Ok( Self
      {
        compiled,
        cmd_buf : Vec::new(),
        last_scene_revision : 0,
        last_clock : 0.0,
        last_camera_signature : CameraSignature
        {
          world_center : ( 0, 0 ),
          zoom : 0,
          viewport_size : ( 0, 0 ),
        },
        has_rendered : false,
        cache_hits : 0,
        batches : HashMap::default(),
        next_batch_id : 0,
        sprite_to_sheet,
      })
    }

    /// Number of `render()` calls served from the per-frame cache (i.e.
    /// returned the previously emitted command slice without re-walking
    /// the scene). Stable across renderer lifetime — does not reset.
    #[ inline ]
    #[ must_use ]
    pub fn cache_hits( &self ) -> u64 { self.cache_hits }

    /// Backend-ready asset table. Submit once at startup via
    /// `backend.load_assets( renderer.assets() )`.
    #[ inline ]
    #[ must_use ]
    pub fn assets( &self ) -> &Assets { &self.compiled.assets }

    /// Produce the per-frame command stream for `scene` viewed through
    /// `camera`. Returns a borrow of the internal buffer — valid until
    /// the next call to [`Renderer::render`]. Submit the slice to a
    /// backend before calling `render` again.
    ///
    /// **Idle-replay cache.** When the scene's revision, clock, and the
    /// camera fingerprint all match the values captured on the previous
    /// successful call, the renderer skips the per-frame scene walk and
    /// returns the previously emitted command slice verbatim. This
    /// closes feedback §9 (`compile_frame` rebuilds the whole command
    /// stream every frame) for the common idle case where the
    /// consumer drives a continuous redraw loop but the world hasn't
    /// changed.
    ///
    /// # Errors
    ///
    /// Returns [`CompileError`] when a sprite source references an
    /// unresolved asset / animation / frame, or when an anchor / source
    /// pair is rejected as unsupported.
    ///
    /// # Panics
    ///
    /// Panics in debug builds on inconsistent internal state — e.g. a
    /// batch entry inserted just above is somehow missing from the
    /// `batches` map when looked up. The renderer maintains this
    /// invariant directly; a panic here indicates a bug in the
    /// renderer itself.
    pub fn render
    (
      &mut self,
      scene : &Scene,
      camera : &Camera,
    ) -> Result< &[ RenderCommand ], CompileError >
    {
      let scene_revision = scene.revision();
      let clock = scene.clock();
      let camera_signature = CameraSignature::from( camera );

      if self.has_rendered
        && scene_revision == self.last_scene_revision
        && clock.to_bits() == self.last_clock.to_bits()
        && camera_signature == self.last_camera_signature
      {
        // Idle replay — buffer from previous render is still valid.
        self.cache_hits += 1;
        return Ok( &self.cmd_buf );
      }

      let emits = gather_frame_emits( &self.compiled, scene, camera )?;

      self.cmd_buf.clear();
      self.cmd_buf.push( RenderCommand::Clear( Clear { color : emits.clear_color } ) );

      // Track which batches were touched this frame; anything in
      // `self.batches` not in this set at the end gets `DeleteBatch`'d.
      let mut used_keys : Vec< BatchKey > = Vec::new();
      // Draw order: emit `DrawBatch`es in the order batches first appear
      // in the walk so the per-bucket / per-sheet draw sequence matches
      // the consumer's declared pipeline order.
      let mut draw_order : Vec< ResourceId< Batch > > = Vec::new();

      for ( bucket_idx, bucket ) in emits.buckets.into_iter().enumerate()
      {
        let bucket_idx_u32 = bucket_idx as u32;

        // Decide batched vs per-sprite for this bucket. `SortMode::None`
        // gives no constraint on instance order within the GPU's
        // instance buffer, so we can freely group by sheet. Sorted
        // buckets are emitted per-sprite (see field doc on
        // `Renderer.batches`).
        let batch_this_bucket = matches!( bucket.sort, SortMode::None );

        if batch_this_bucket && !bucket.sprites.is_empty()
        {
          // Group consecutive same-(sheet, blend, clip) sprites. Order
          // within a group is the bucket's pre-sorted order, which for
          // `SortMode::None` is the spawn / iteration order from
          // `Scene` — stable across frames given identical state.
          let groups = self.group_sprites( bucket_idx_u32, &bucket.sprites );
          for ( key, sprites_in_group ) in groups
          {
            self.emit_or_update_batch( key, sprites_in_group );
            used_keys.push( key );
            // batch id was just inserted; safe to look up
            let id = self.batches.get( &key ).expect( "just inserted" ).id;
            draw_order.push( id );
          }
        }
        else
        {
          // Fall-back: emit per-sprite `Sprite` commands so existing
          // sorted-bucket ordering semantics are preserved.
          for sprite in bucket.sprites
          {
            self.cmd_buf.push( RenderCommand::Sprite( sprite ) );
          }
        }

        // Viewport sprites — always per-sprite for Step 4 (deferred
        // batching item in the plan).
        for sprite in bucket.screen_space
        {
          self.cmd_buf.push( RenderCommand::ScreenSpaceSprite( sprite ) );
        }
      }

      // Garbage-collect batches that received no instances this frame.
      let used_set : rustc_hash::FxHashSet< BatchKey > =
        used_keys.iter().copied().collect();
      let stale : Vec< ( BatchKey, ResourceId< Batch > ) > = self.batches.iter()
        .filter( | ( k, _ ) | !used_set.contains( k ) )
        .map( | ( k, v ) | ( *k, v.id ) )
        .collect();
      for ( key, id ) in stale
      {
        self.cmd_buf.push( RenderCommand::DeleteBatch( DeleteBatch { batch : id } ) );
        self.batches.remove( &key );
      }

      // Issue draw calls in walk order.
      for id in draw_order
      {
        self.cmd_buf.push( RenderCommand::DrawBatch( DrawBatch { batch : id } ) );
      }

      self.last_scene_revision = scene_revision;
      self.last_clock = clock;
      self.last_camera_signature = camera_signature;
      self.has_rendered = true;
      Ok( &self.cmd_buf )
    }

    /// Group a bucket's pre-sorted sprite stream by batch key while
    /// preserving original order. Two non-adjacent runs with the same
    /// key end up in the same returned group (sprites are appended to
    /// the running entry for that key).
    fn group_sprites
    (
      &self,
      bucket_idx : u32,
      sprites : &[ Sprite ],
    ) -> Vec< ( BatchKey, Vec< Sprite > ) >
    {
      let mut groups : Vec< ( BatchKey, Vec< Sprite > ) > = Vec::new();
      let mut index_of : HashMap< BatchKey, usize > = HashMap::default();
      for sprite in sprites
      {
        let Some( &sheet ) = self.sprite_to_sheet.get( &sprite.sprite )
        else
        {
          // Sprite id not in the compiled assets — should be impossible
          // since `gather_frame_emits` only emits sprite ids from the
          // same `compiled` table. Skip defensively.
          continue;
        };
        let key = BatchKey
        {
          bucket_idx,
          sheet,
          blend : core::mem::discriminant( &sprite.blend ),
          clip : sprite.clip,
        };
        if let Some( &i ) = index_of.get( &key )
        {
          groups[ i ].1.push( *sprite );
        }
        else
        {
          index_of.insert( key, groups.len() );
          groups.push( ( key, vec![ *sprite ] ) );
        }
      }
      groups
    }

    /// Reuse an existing batch under `key` (emitting `Bind` + the
    /// minimal Set / Remove / Add diff + `Unbind`) or allocate a fresh
    /// one (emitting `CreateSpriteBatch` + `Bind` + N×`Add` + `Unbind`).
    fn emit_or_update_batch
    (
      &mut self,
      key : BatchKey,
      sprites : Vec< Sprite >,
    )
    {
      if let Some( entry ) = self.batches.get_mut( &key )
      {
        let old_n = entry.instances.len();
        let new_n = sprites.len();
        self.cmd_buf.push( RenderCommand::BindBatch( BindBatch { batch : entry.id } ) );
        // Set slots 0..min(old, new) — for simplicity we always emit
        // `SetSpriteInstance` (no "is unchanged" elision). The
        // alternative (per-field bit-equal compare) saves commands on
        // partial updates but adds complexity; defer that optimisation.
        let common = old_n.min( new_n );
        for ( i, s ) in sprites.iter().enumerate().take( common )
        {
          self.cmd_buf.push( RenderCommand::SetSpriteInstance( SetSpriteInstance
          {
            index : i as u32,
            transform : s.transform,
            sprite : s.sprite,
            tint : s.tint,
          }));
        }
        // Trim from the tail (swap-remove semantics — see
        // `tilemap_renderer/src/commands.rs:418-422`). Walking from the
        // highest index down keeps later indices stable.
        for i in ( new_n..old_n ).rev()
        {
          self.cmd_buf.push( RenderCommand::RemoveInstance( RemoveInstance { index : i as u32 } ) );
        }
        // Extend.
        for s in sprites.iter().skip( common )
        {
          self.cmd_buf.push( RenderCommand::AddSpriteInstance( AddSpriteInstance
          {
            transform : s.transform,
            sprite : s.sprite,
            tint : s.tint,
          }));
        }
        self.cmd_buf.push( RenderCommand::UnbindBatch( UnbindBatch ) );
        entry.instances = sprites;
      }
      else
      {
        let id : ResourceId< Batch > = ResourceId::new( self.next_batch_id );
        self.next_batch_id += 1;
        // Recover the live `BlendMode` from the first sprite (the
        // discriminant in `key` is a comparison helper, not a value).
        let blend = sprites.first().map_or( BlendMode::default(), | s | s.blend );
        self.cmd_buf.push( RenderCommand::CreateSpriteBatch( CreateSpriteBatch
        {
          batch : id,
          params : SpriteBatchParams
          {
            transform : Transform::default(),
            sheet : key.sheet,
            blend,
            clip : key.clip,
          },
        }));
        self.cmd_buf.push( RenderCommand::BindBatch( BindBatch { batch : id } ) );
        for s in &sprites
        {
          self.cmd_buf.push( RenderCommand::AddSpriteInstance( AddSpriteInstance
          {
            transform : s.transform,
            sprite : s.sprite,
            tint : s.tint,
          }));
        }
        self.cmd_buf.push( RenderCommand::UnbindBatch( UnbindBatch ) );
        self.batches.insert( key, BatchEntry { id, instances : sprites } );
      }
    }
  }
}

mod_interface::mod_interface!
{
  exposed use Renderer;
}
