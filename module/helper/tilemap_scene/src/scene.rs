//! Retained-mode rendering scene — the runtime counterpart of
//! [`crate::snapshot::SceneSnapshot`].
//!
//! `Scene` owns the *render-world*: which instances exist, where they are,
//! in what state. Game / adapter code mutates the scene through
//! `spawn` / `despawn` / `move_to` / `set_state` / `set_*` methods and reads
//! it back through `instance` / `instances_at_hex` / `instances`. The
//! retained representation eliminates per-frame `Vec< String >` allocations
//! (handles are `Copy`), supports per-instance phase / tint overrides, and
//! lets multiple renderers consume the same world without state
//! duplication.
//!
//! Construction is anchored to an `Arc<RenderSpec>`. The spec is immutable
//! for the scene's lifetime — adding a new `Object` at runtime requires
//! building a fresh scene with the extended spec. This is deliberate:
//! mutating the spec on a live scene invalidates handles and the prior
//! consumer-side workaround for that was the documented source of texture
//! flicker (see `TILEMAP_SCENE_FEEDBACK.md` §1).
//!
//! Spatial indexes (`instances_at_hex`, per-anchor `Vec`s) are maintained
//! eagerly on every mutation. A first-cut implementation uses `Vec::retain`
//! for despawn/move which is O(n) — for the workloads the crate targets
//! (a few hundred instances) this is negligible; a later pass can swap
//! in `IndexSet` or per-handle position tracking if profiling demands it.

mod private
{
  use alloc::sync::Arc;
  use rustc_hash::FxHashMap as HashMap;
  use slotmap::SlotMap;
  use crate::anchor::EdgeDirection;
  use crate::compile::animation::{ animation_duration_seconds, declared_phase_seconds };
  use crate::error::SnapshotLoadError;
  use crate::event::SceneEvent;
  use crate::instance::{ Instance, InstanceHandle, ObjectHandle, Placement, StateHandle };
  use crate::resource::{ Animation, AnimationMode, SpriteRef, TintRef };
  use crate::snapshot::SceneSnapshot;
  use crate::source::SpriteSource;
  use crate::spec::RenderSpec;

  /// Retained-mode rendering scene.
  pub struct Scene
  {
    spec : Arc< RenderSpec >,
    instances : SlotMap< InstanceHandle, Instance >,

    /// Per-object metadata: sorted state names + default state index.
    /// Indexed by `ObjectHandle::index()`.
    objects : Vec< ObjectMeta >,
    object_by_id : HashMap< String, ObjectHandle >,

    /// Spec `Animation` lookup by id. Built once in [`Self::new`]; used
    /// by [`Self::tick`] to resolve `SpriteSource::Animation` references
    /// without a per-tick linear scan over `spec.animations`.
    animation_by_id : HashMap< String, usize >,

    /// Hex spatial index — `( q, r )` → instances anchored at that cell.
    /// Includes `Placement::Hex`, `Placement::Multihex` (anchor cell), and
    /// `Placement::Edge` (owning hex). Multiple instances per cell are
    /// preserved (tile object stacks).
    instances_at_hex : HashMap< ( i32, i32 ), Vec< InstanceHandle > >,

    /// Per-anchor lists used by `Renderer` to iterate the world by pipeline
    /// pass. Lists are kept in insertion order (spawn order); `Renderer`
    /// applies per-bucket sort modes on top of this.
    hex_instances : Vec< InstanceHandle >,
    edge_instances : Vec< InstanceHandle >,
    free_instances : Vec< InstanceHandle >,
    viewport_instances : Vec< InstanceHandle >,
    multihex_instances : Vec< InstanceHandle >,

    /// Master clock — advances via [`Self::tick`] (which also emits any
    /// [`SceneEvent`]s triggered by the elapsed interval). Captured into
    /// `Instance.spawn_time` on `spawn` for `OneShot` animation timing.
    clock : f32,

    /// Optional override for `RenderPipeline.global_tint`. `None` = use
    /// the spec's declared global tint (or no tint if neither is set).
    global_tint_override : Option< TintRef >,

    /// 64-bit pseudo-random seed for `VariantSelection::Random`. Folded
    /// down to `u32` by the renderer.
    seed : u64,

    /// Monotonic mutation counter. Bumped exactly once per successful
    /// state-changing call (`spawn` / `despawn` / `move_to` / `set_state` /
    /// `set_visible` / `set_tint` / `set_phase_offset` /
    /// `set_external_sprite` / `set_global_tint` / `set_seed`). `tick`
    /// does NOT bump — clock advance is a separate signal exposed via
    /// [`Self::clock`].
    revision : u64,
  }

  /// Per-object metadata cached at `Scene::new`.
  ///
  /// State names are stored alphabetically sorted so `StateHandle.state_index`
  /// is reproducible across loads.
  struct ObjectMeta
  {
    state_names : Vec< String >,
    default_state_index : u16,
  }

  impl Scene
  {
    /// Materialise a [`SceneSnapshot`] into a retained-mode scene.
    ///
    /// Walks the snapshot's tile stacks (or expanded `palette + map`
    /// form), edges, multihex / free / viewport instances and `entities`
    /// vector and `spawn`s each one. `initial_global_tint` and `seed`
    /// from the snapshot are applied via the corresponding setters.
    /// The snapshot's `players` field — game-logic concept — is
    /// **ignored**; consumers that need team colours read them from
    /// their own model.
    ///
    /// # Errors
    ///
    /// - [`SnapshotLoadError::UnknownObject`] when the snapshot
    ///   references an object id the spec does not declare.
    /// - [`SnapshotLoadError::UnknownPaletteChar`] when an ASCII `map`
    ///   cell uses a character missing from `palette`.
    pub fn from_snapshot
    (
      snap : &SceneSnapshot,
      spec : Arc< RenderSpec >,
    ) -> Result< Self, SnapshotLoadError >
    {
      let mut scene = Self::new( spec );

      // Tiles — explicit list takes precedence; ASCII palette+map otherwise.
      let owned_tiles;
      let tiles_iter : &[ crate::snapshot::Tile ] = if snap.tiles.is_empty()
      {
        owned_tiles = snap.expand_palette()?;
        &owned_tiles
      }
      else
      {
        &snap.tiles
      };

      for tile in tiles_iter
      {
        let ( q, r ) = tile.pos;
        for object_id in &tile.objects
        {
          let obj = scene.object( object_id ).ok_or_else( || SnapshotLoadError::UnknownObject
          {
            id : object_id.clone(),
            context : format!( "tile ({q}, {r})" ),
          })?;
          scene.spawn( obj, Placement::Hex { q, r } );
        }
      }

      for inst in &snap.edges
      {
        let obj = scene.object( &inst.object ).ok_or_else( || SnapshotLoadError::UnknownObject
        {
          id : inst.object.clone(),
          context : "edge instance".into(),
        })?;
        let _ : EdgeDirection = inst.at.dir;
        scene.spawn( obj, Placement::Edge { hex : inst.at.hex, dir : inst.at.dir } );
      }

      for inst in &snap.multihex_instances
      {
        let obj = scene.object( &inst.object ).ok_or_else( || SnapshotLoadError::UnknownObject
        {
          id : inst.object.clone(),
          context : "multihex instance".into(),
        })?;
        scene.spawn( obj, Placement::Multihex { anchor : inst.anchor } );
      }

      for inst in &snap.free_instances
      {
        let obj = scene.object( &inst.object ).ok_or_else( || SnapshotLoadError::UnknownObject
        {
          id : inst.object.clone(),
          context : "free instance".into(),
        })?;
        scene.spawn( obj, Placement::FreePos { x : inst.pos.0, y : inst.pos.1 } );
      }

      for inst in &snap.viewport_instances
      {
        let obj = scene.object( &inst.object ).ok_or_else( || SnapshotLoadError::UnknownObject
        {
          id : inst.object.clone(),
          context : "viewport instance".into(),
        })?;
        scene.spawn( obj, Placement::Viewport );
      }

      // Entities are hex-anchored game pieces in the snapshot model;
      // in the retained Scene they're just regular Placement::Hex instances.
      for ent in &snap.entities
      {
        let obj = scene.object( &ent.object ).ok_or_else( || SnapshotLoadError::UnknownObject
        {
          id : ent.object.clone(),
          context : "entity".into(),
        })?;
        scene.spawn( obj, Placement::Hex { q : ent.at.0, r : ent.at.1 } );
      }

      if let Some( tint_id ) = snap.initial_global_tint.as_ref()
      {
        scene.set_global_tint( Some( crate::resource::TintRef( tint_id.clone() ) ) );
      }
      if let Some( seed ) = snap.seed
      {
        scene.set_seed( seed );
      }

      Ok( scene )
    }

    /// Build an empty scene anchored to `spec`.
    ///
    /// Walks `spec.objects` to build the lookup tables. Each object's state
    /// names are sorted alphabetically — the position in that sorted list
    /// is the value of `StateHandle.state_index`. The object's
    /// `default_state` is resolved to its sorted index and cached for
    /// `Scene::default_state`.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if two objects share an `id` (the spec is
    /// expected to be SPEC §16-validated before reaching this constructor).
    /// Release builds keep the first definition and shadow later
    /// duplicates.
    #[ must_use ]
    pub fn new( spec : Arc< RenderSpec > ) -> Self
    {
      let mut objects : Vec< ObjectMeta > = Vec::with_capacity( spec.objects.len() );
      let mut object_by_id : HashMap< String, ObjectHandle > = HashMap::default();
      let mut animation_by_id : HashMap< String, usize > = HashMap::default();
      for ( idx, anim ) in spec.animations.iter().enumerate()
      {
        animation_by_id.entry( anim.id.clone() ).or_insert( idx );
      }

      for ( idx, obj ) in spec.objects.iter().enumerate()
      {
        let mut state_names : Vec< String > = obj.states.keys().cloned().collect();
        state_names.sort();
        let default_state_index = state_names
          .iter()
          .position( | n | n == &obj.default_state )
          .map_or( 0, | i | i as u16 );

        let handle = ObjectHandle( idx as u32 );
        debug_assert!
        (
          !object_by_id.contains_key( &obj.id ),
          "duplicate object id {:?} in spec", obj.id,
        );
        object_by_id.entry( obj.id.clone() ).or_insert( handle );
        objects.push( ObjectMeta
        {
          state_names,
          default_state_index,
        });
      }

      Self
      {
        spec,
        instances : SlotMap::with_key(),
        objects,
        object_by_id,
        animation_by_id,
        instances_at_hex : HashMap::default(),
        hex_instances : Vec::new(),
        edge_instances : Vec::new(),
        free_instances : Vec::new(),
        viewport_instances : Vec::new(),
        multihex_instances : Vec::new(),
        clock : 0.0,
        global_tint_override : None,
        seed : 0,
        revision : 0,
      }
    }

    // ════════════════════════════════════════════════════════════════
    // Handle resolution
    // ════════════════════════════════════════════════════════════════

    /// Resolve a spec object id to its [`ObjectHandle`]. `None` if not declared.
    #[ inline ]
    #[ must_use ]
    pub fn object( &self, id : &str ) -> Option< ObjectHandle >
    {
      self.object_by_id.get( id ).copied()
    }

    /// Resolve a state name on the given object. `None` if the object has
    /// no state with that name.
    #[ inline ]
    #[ must_use ]
    pub fn state( &self, object : ObjectHandle, name : &str ) -> Option< StateHandle >
    {
      let meta = self.objects.get( object.0 as usize )?;
      let state_index = meta.state_names.iter().position( | n | n == name )? as u16;
      Some( StateHandle { object, state_index } )
    }

    /// The object's `default_state` as a [`StateHandle`].
    ///
    /// # Panics
    ///
    /// Panics if `object` does not refer to a declared object — this
    /// indicates the handle was forged or came from a different scene.
    #[ inline ]
    #[ must_use ]
    pub fn default_state( &self, object : ObjectHandle ) -> StateHandle
    {
      let meta = &self.objects[ object.0 as usize ];
      StateHandle { object, state_index : meta.default_state_index }
    }

    /// Return the alphabetically-sorted state name corresponding to a
    /// [`StateHandle`]. Useful for debug logging and for renderers that
    /// need to look the layer stack up in `RenderSpec.objects[i].states`.
    #[ inline ]
    #[ must_use ]
    pub fn state_name( &self, state : StateHandle ) -> Option< &str >
    {
      self.objects.get( state.object.0 as usize )
        .and_then( | meta | meta.state_names.get( state.state_index as usize ) )
        .map( String::as_str )
    }

    // ════════════════════════════════════════════════════════════════
    // Mutation API
    // ════════════════════════════════════════════════════════════════

    /// Spawn a new instance of `object` at `placement`.
    ///
    /// Initial state = object's `default_state`. Visible by default. Returns
    /// the handle the caller uses for subsequent mutations / queries.
    ///
    /// # Panics
    ///
    /// Panics if `object` does not refer to a declared object in this
    /// scene's spec (forged handle).
    pub fn spawn
    (
      &mut self,
      object : ObjectHandle,
      placement : Placement,
    ) -> InstanceHandle
    {
      let state = self.default_state( object );
      let instance = Instance
      {
        object,
        placement,
        state,
        visible : true,
        tint : None,
        phase_offset : None,
        spawn_time : self.clock,
        external_sprites : HashMap::default(),
      };
      let handle = self.instances.insert( instance );
      self.index_insert( handle, placement );
      self.revision += 1;
      handle
    }

    /// Despawn an instance. No-op (with `debug_assert`) if the handle is
    /// stale.
    pub fn despawn( &mut self, h : InstanceHandle )
    {
      let Some( inst ) = self.instances.remove( h )
      else
      {
        debug_assert!( false, "despawn on stale handle {h:?}" );
        return;
      };
      self.index_remove( h, inst.placement );
      self.revision += 1;
    }

    /// Move an existing instance to a new placement. The new placement
    /// MAY use a different variant than the previous one (e.g. `Hex` →
    /// `FreePos`); spatial indexes are updated accordingly.
    pub fn move_to( &mut self, h : InstanceHandle, placement : Placement )
    {
      let Some( inst ) = self.instances.get_mut( h )
      else
      {
        debug_assert!( false, "move_to on stale handle {h:?}" );
        return;
      };
      let old = inst.placement;
      inst.placement = placement;
      self.index_remove( h, old );
      self.index_insert( h, placement );
      self.revision += 1;
    }

    /// Switch the active state of `h`. `state` must belong to the same
    /// object as the instance (otherwise debug-asserts and is ignored).
    pub fn set_state( &mut self, h : InstanceHandle, state : StateHandle )
    {
      let Some( inst ) = self.instances.get_mut( h )
      else
      {
        debug_assert!( false, "set_state on stale handle {h:?}" );
        return;
      };
      let inst_object = inst.object;
      debug_assert_eq!
      (
        inst_object, state.object,
        "set_state: state {state:?} does not belong to instance's object {inst_object:?}",
      );
      if inst_object == state.object
      {
        inst.state = state;
        self.revision += 1;
      }
    }

    /// Toggle instance visibility. Hidden instances stay in spatial indexes
    /// so flipping back is O(1).
    pub fn set_visible( &mut self, h : InstanceHandle, on : bool )
    {
      if let Some( inst ) = self.instances.get_mut( h )
      {
        inst.visible = on;
        self.revision += 1;
      }
      else { debug_assert!( false, "set_visible on stale handle {h:?}" ); }
    }

    /// Override the instance's tint multiplier. `None` clears any prior override.
    pub fn set_tint( &mut self, h : InstanceHandle, tint : Option< [ f32; 4 ] > )
    {
      if let Some( inst ) = self.instances.get_mut( h )
      {
        inst.tint = tint;
        self.revision += 1;
      }
      else { debug_assert!( false, "set_tint on stale handle {h:?}" ); }
    }

    /// Override the animation phase offset for this instance in seconds.
    /// `None` falls back to the animation's declared `PhaseOffset`.
    pub fn set_phase_offset( &mut self, h : InstanceHandle, t : Option< f32 > )
    {
      if let Some( inst ) = self.instances.get_mut( h )
      {
        inst.phase_offset = t;
        self.revision += 1;
      }
      else { debug_assert!( false, "set_phase_offset on stale handle {h:?}" ); }
    }

    /// Populate an `SpriteSource::External` slot for this instance.
    /// `slot` is the source's declared slot name.
    pub fn set_external_sprite
    (
      &mut self,
      h : InstanceHandle,
      slot : &str,
      sprite : SpriteRef,
    )
    {
      if let Some( inst ) = self.instances.get_mut( h )
      {
        inst.external_sprites.insert( slot.to_owned(), sprite );
        self.revision += 1;
      }
      else
      {
        debug_assert!( false, "set_external_sprite on stale handle {h:?}" );
      }
    }

    /// Override the scene-wide global tint. `None` falls back to the spec's
    /// declared `pipeline.global_tint`.
    #[ inline ]
    pub fn set_global_tint( &mut self, t : Option< TintRef > )
    {
      self.global_tint_override = t;
      self.revision += 1;
    }

    /// Set the seed consumed by `VariantSelection::Random`. Stays stable
    /// across frames so variant choices don't flicker.
    #[ inline ]
    pub fn set_seed( &mut self, seed : u64 )
    {
      self.seed = seed;
      self.revision += 1;
    }

    /// Advance the master clock by `dt` seconds and return every
    /// [`SceneEvent`] produced during the interval.
    ///
    /// Currently emits [`SceneEvent::AnimationCompleted`] for every
    /// `(instance, layer)` pair whose leaf `SpriteSource::Animation`
    /// has `AnimationMode::OneShot` and whose effective local time
    /// crossed the animation's total duration during this tick.
    ///
    /// **Visibility.** Hidden instances (`set_visible(_, false)`) do
    /// NOT emit events — visibility gates both rendering and event
    /// emission so a paused / preview unit doesn't churn the
    /// consumer's event loop with completions it can't see.
    ///
    /// **Composite sources.** `OneShot` detection inspects only leaf
    /// [`SpriteSource::Animation`] layers. `OneShot` animations nested
    /// inside [`SpriteSource::Variant`], [`SpriteSource::NeighborBitmask`],
    /// [`SpriteSource::ViewportTiled`], etc., are not tracked — those
    /// composites mask which sub-source is active per-instance and
    /// aren't typical OneShot-script carriers. Lifting this limit is
    /// a deferred polish item.
    ///
    /// **Determinism.** For a given `(scene state, dt)` the returned
    /// event list is byte-equal across runs; iteration follows the
    /// per-anchor `Vec`s in spawn order.
    pub fn tick( &mut self, dt : f32 ) -> Vec< SceneEvent >
    {
      let clock_before = self.clock;
      self.clock += dt;
      let clock_after = self.clock;

      let mut events : Vec< SceneEvent > = Vec::new();

      // Iterate every live placement bucket; spawn-order preserved.
      let buckets : [ &[ InstanceHandle ]; 5 ] =
      [
        &self.hex_instances,
        &self.edge_instances,
        &self.free_instances,
        &self.viewport_instances,
        &self.multihex_instances,
      ];

      for bucket in buckets
      {
        for &handle in bucket
        {
          let Some( inst ) = self.instances.get( handle )
          else
          {
            continue;
          };
          if !inst.visible
          {
            continue;
          }

          let Some( state_name ) = self.state_name_internal( inst.state )
          else
          {
            continue;
          };
          let Some( object ) = self.spec.objects.get( inst.object.0 as usize )
          else
          {
            continue;
          };
          let Some( layers ) = object.states.get( state_name )
          else
          {
            continue;
          };

          let pos = inst.placement.hex_coord().unwrap_or( ( 0, 0 ) );

          for ( layer_index, layer ) in layers.iter().enumerate()
          {
            let SpriteSource::Animation( anim_ref ) = &layer.sprite_source
            else
            {
              continue;
            };

            let Some( &anim_idx ) = self.animation_by_id.get( &anim_ref.0 )
            else
            {
              continue;
            };
            let anim : &Animation = &self.spec.animations[ anim_idx ];
            if !matches!( anim.mode, AnimationMode::OneShot )
            {
              continue;
            }

            let duration = animation_duration_seconds( anim );
            if duration <= 0.0
            {
              continue;
            }

            let phase = inst.phase_offset
              .unwrap_or_else( || declared_phase_seconds( anim, pos ) );
            let t_before = clock_before + phase;
            let t_after = clock_after + phase;
            if t_before < duration && t_after >= duration
            {
              events.push( SceneEvent::AnimationCompleted
              {
                instance : handle,
                state : inst.state,
                layer_index : layer_index as u16,
                animation : anim_ref.clone(),
              });
            }
          }
        }
      }

      events
    }

    /// Internal `state_name` borrow that takes `&self` without
    /// reborrowing through the public API (avoids any future borrow
    /// snags inside `tick`'s nested loops).
    fn state_name_internal( &self, state : StateHandle ) -> Option< &str >
    {
      self.objects.get( state.object.0 as usize )
        .and_then( | meta | meta.state_names.get( state.state_index as usize ) )
        .map( String::as_str )
    }

    // ════════════════════════════════════════════════════════════════
    // Query API
    // ════════════════════════════════════════════════════════════════

    /// The spec this scene was built against.
    #[ inline ]
    #[ must_use ]
    pub fn spec( &self ) -> &RenderSpec { &self.spec }

    /// Current clock value in seconds.
    #[ inline ]
    #[ must_use ]
    pub fn clock( &self ) -> f32 { self.clock }

    /// Monotonic mutation counter. Bumped exactly once per successful
    /// state-changing call. Renderers snapshot this to detect "anything
    /// mutated since I last looked" without diffing the scene contents.
    ///
    /// [`Self::tick`] does NOT bump revision — clock advance is a
    /// separate signal available via [`Self::clock`].
    #[ inline ]
    #[ must_use ]
    pub fn revision( &self ) -> u64 { self.revision }

    /// The override for `pipeline.global_tint`, if set.
    #[ inline ]
    #[ must_use ]
    pub fn global_tint( &self ) -> Option< &TintRef > { self.global_tint_override.as_ref() }

    /// `VariantSelection::Random` seed value.
    #[ inline ]
    #[ must_use ]
    pub fn seed( &self ) -> u64 { self.seed }

    /// Borrow one instance read-only. `None` if the handle is stale.
    #[ inline ]
    #[ must_use ]
    pub fn instance( &self, h : InstanceHandle ) -> Option< &Instance >
    {
      self.instances.get( h )
    }

    /// Iterate over every live `(handle, instance)` pair. No order
    /// guarantee — use the per-anchor accessors for renderer iteration.
    pub fn instances( &self ) -> impl Iterator< Item = ( InstanceHandle, &Instance ) > + '_
    {
      self.instances.iter()
    }

    /// Live handles anchored at `( q, r )`. Returns an empty iterator
    /// when the cell has no instances.
    pub fn instances_at_hex
    (
      &self,
      q : i32,
      r : i32,
    ) -> impl Iterator< Item = InstanceHandle > + '_
    {
      self.instances_at_hex
        .get( &( q, r ) )
        .into_iter()
        .flatten()
        .copied()
    }

    /// Handles of every `Placement::Hex` instance, in spawn order.
    #[ inline ]
    #[ must_use ]
    pub fn hex_instances( &self ) -> &[ InstanceHandle ] { &self.hex_instances }

    /// Handles of every `Placement::Edge` instance, in spawn order.
    #[ inline ]
    #[ must_use ]
    pub fn edge_instances( &self ) -> &[ InstanceHandle ] { &self.edge_instances }

    /// Handles of every `Placement::FreePos` instance, in spawn order.
    #[ inline ]
    #[ must_use ]
    pub fn free_instances( &self ) -> &[ InstanceHandle ] { &self.free_instances }

    /// Handles of every `Placement::Viewport` instance, in spawn order.
    #[ inline ]
    #[ must_use ]
    pub fn viewport_instances( &self ) -> &[ InstanceHandle ] { &self.viewport_instances }

    /// Handles of every `Placement::Multihex` instance, in spawn order.
    #[ inline ]
    #[ must_use ]
    pub fn multihex_instances( &self ) -> &[ InstanceHandle ] { &self.multihex_instances }

    /// Total live instance count.
    #[ inline ]
    #[ must_use ]
    pub fn len( &self ) -> usize { self.instances.len() }

    /// `true` when no instances are alive.
    #[ inline ]
    #[ must_use ]
    pub fn is_empty( &self ) -> bool { self.instances.is_empty() }

    // ════════════════════════════════════════════════════════════════
    // Spatial-index maintenance — private helpers
    // ════════════════════════════════════════════════════════════════

    fn index_insert( &mut self, h : InstanceHandle, p : Placement )
    {
      match p
      {
        Placement::Hex { .. }      => self.hex_instances.push( h ),
        Placement::Edge { .. }     => self.edge_instances.push( h ),
        Placement::FreePos { .. }  => self.free_instances.push( h ),
        Placement::Viewport        => self.viewport_instances.push( h ),
        Placement::Multihex { .. } => self.multihex_instances.push( h ),
      }
      if let Some( cell ) = p.hex_coord()
      {
        self.instances_at_hex.entry( cell ).or_default().push( h );
      }
    }

    fn index_remove( &mut self, h : InstanceHandle, p : Placement )
    {
      let bucket : &mut Vec< InstanceHandle > = match p
      {
        Placement::Hex { .. }      => &mut self.hex_instances,
        Placement::Edge { .. }     => &mut self.edge_instances,
        Placement::FreePos { .. }  => &mut self.free_instances,
        Placement::Viewport        => &mut self.viewport_instances,
        Placement::Multihex { .. } => &mut self.multihex_instances,
      };
      bucket.retain( | &x | x != h );

      if let Some( cell ) = p.hex_coord()
        && let Some( vec ) = self.instances_at_hex.get_mut( &cell )
      {
        vec.retain( | &x | x != h );
        if vec.is_empty() { self.instances_at_hex.remove( &cell ); }
      }
    }
  }
}

mod_interface::mod_interface!
{
  exposed use Scene;
}
