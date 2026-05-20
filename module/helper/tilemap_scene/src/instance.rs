//! Retained-mode handles and per-instance data.
//!
//! These types form the value side of the retained-mode API exposed by
//! [`crate::scene::Scene`]:
//!
//! - [`InstanceHandle`] — opaque, generational key for one live instance.
//!   Issued by `Scene::spawn`, invalidated by `Scene::despawn`.
//! - [`ObjectHandle`] — interned reference to an `Object` declared in the
//!   spec. Resolved once by `Scene::new`; stable for the lifetime of the
//!   scene's spec.
//! - [`StateHandle`] — interned reference to one named state within an
//!   object. Carries its parent [`ObjectHandle`] so cross-object mistakes
//!   are debug-detectable.
//! - [`Placement`] — anchor-specific position payload supplied at spawn
//!   time. The variant determines which rendering path applies to the
//!   instance; an explicit `move_to` is required to switch variants.
//! - [`Instance`] — the per-instance runtime state mutated by `Scene`
//!   methods. Exposed read-only via [`crate::scene::Scene::instance`] for
//!   game / debug queries.

mod private
{
  use crate::anchor::EdgeDirection;
  use crate::resource::SpriteRef;

  slotmap::new_key_type!
  {
    /// Generational handle to a live instance inside a [`crate::scene::Scene`].
    ///
    /// Returned by `Scene::spawn`, accepted by every mutation / query method
    /// that operates on a single instance. Becomes stale after `despawn`;
    /// stale handles are silently rejected by mutations (release builds) or
    /// trigger a `debug_assert!` (debug builds).
    pub struct InstanceHandle;
  }

  /// Interned reference to an `Object` declared in the spec.
  ///
  /// `ObjectHandle(i)` corresponds to `RenderSpec.objects[i]`. Resolved by
  /// [`crate::scene::Scene::object`]; stable across the scene's lifetime
  /// because [`crate::scene::Scene`] holds an `Arc<RenderSpec>` that is
  /// immutable after construction.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd ) ]
  pub struct ObjectHandle( pub( crate ) u32 );

  impl ObjectHandle
  {
    /// Numeric index of this handle (matches `RenderSpec.objects[i]`).
    #[ inline ]
    #[ must_use ]
    pub fn index( self ) -> u32 { self.0 }
  }

  /// Interned reference to a named state on a specific object.
  ///
  /// Pairs the parent [`ObjectHandle`] with a per-object `state_index`. The
  /// index is the position of the state's name in the object's
  /// alphabetically-sorted state list — sorting (not raw `HashMap`
  /// iteration) is what makes the index reproducible across runs.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
  pub struct StateHandle
  {
    /// Object this state belongs to.
    pub object : ObjectHandle,
    /// Index into the object's alphabetically-sorted state list.
    pub state_index : u16,
  }

  /// Anchor-specific position payload for an instance.
  ///
  /// The variant determines what rendering path applies. Switching variants
  /// requires an explicit `move_to` call — the renderer rejects a placement
  /// whose variant is incompatible with the owning object's declared
  /// sprite sources (e.g. `Placement::FreePos` paired with
  /// `SpriteSource::NeighborBitmask`).
  #[ derive( Debug, Clone, Copy ) ]
  pub enum Placement
  {
    /// One hex cell. Position = `( q, r )` in axial coordinates.
    Hex
    {
      /// Axial q coordinate.
      q : i32,
      /// Axial r coordinate.
      r : i32,
    },
    /// An edge between two hex cells. Position = `( hex, direction )`.
    Edge
    {
      /// Owning hex (the canonical-side decision is made by the renderer).
      hex : ( i32, i32 ),
      /// Direction from `hex` to the neighbour across the edge.
      dir : EdgeDirection,
    },
    /// Free world-space pixel point.
    FreePos
    {
      /// World-space x in pixels.
      x : f32,
      /// World-space y in pixels.
      y : f32,
    },
    /// Multihex shape anchored at a cell. Shape is read from the object's
    /// `Anchor::Multihex.shape` field at render time.
    Multihex
    {
      /// Anchor cell in axial coordinates.
      anchor : ( i32, i32 ),
    },
    /// Screen-space. No world position — the renderer reads viewport state
    /// and the layer's `ViewportTiled` source to determine pixel placement.
    Viewport,
  }

  impl Placement
  {
    /// Returns the hex coordinate this placement occupies, if any. Used by
    /// `Scene`'s spatial index for fast `instances_at_hex` lookups.
    #[ inline ]
    #[ must_use ]
    pub fn hex_coord( &self ) -> Option< ( i32, i32 ) >
    {
      match *self
      {
        Self::Hex { q, r } => Some( ( q, r ) ),
        Self::Multihex { anchor } => Some( anchor ),
        Self::Edge { hex, .. } => Some( hex ),
        Self::FreePos { .. } | Self::Viewport => None,
      }
    }
  }

  /// Per-instance runtime state.
  ///
  /// Mutated only through [`crate::scene::Scene`] methods so spatial indexes
  /// and lookup tables stay in sync. Read-only access via
  /// [`crate::scene::Scene::instance`].
  #[ derive( Debug, Clone ) ]
  pub struct Instance
  {
    /// The object class this instance is rendered as.
    pub object : ObjectHandle,
    /// Current world / screen position.
    pub placement : Placement,
    /// Active state — selects the layer stack rendered each frame.
    pub state : StateHandle,
    /// When `false`, the instance is skipped during rendering. Spatial
    /// indexes still include it (toggling visibility is O(1)).
    pub visible : bool,
    /// Optional per-instance tint multiplier. `None` falls back to whatever
    /// the layer / global tint pipeline yields.
    pub tint : Option< [ f32; 4 ] >,
    /// Per-instance animation phase offset in seconds. Overrides the
    /// animation's declared [`crate::resource::PhaseOffset`] for this
    /// instance when set — lets `OneShot` animations start "now" without
    /// spec mutation.
    pub phase_offset : Option< f32 >,
    /// Scene clock value captured at [`crate::scene::Scene::spawn`].
    /// Stable for the instance's lifetime — represents literal birth
    /// time, useful for age-based effects and debug logging. Not used
    /// for `OneShot` timing; see [`Self::state_entered_time`].
    pub spawn_time : f32,
    /// Scene clock value captured every time the instance enters a new
    /// state — set by [`crate::scene::Scene::spawn`] (to the same value
    /// as `spawn_time`) and updated by every successful
    /// [`crate::scene::Scene::set_state`]. This is the origin used for
    /// `OneShot` animation timing and completion events, so calling
    /// `set_state` on an existing instance restarts its `OneShot`
    /// animation from frame 0.
    pub state_entered_time : f32,
    /// External-source sprite overrides keyed by slot name. Populated via
    /// `Scene::set_external_sprite`; consumed by `SpriteSource::External`
    /// layers during rendering.
    ///
    /// Stored on the instance (not in a separate scene-level map) so
    /// `despawn` cleans them up for free.
    pub external_sprites : rustc_hash::FxHashMap< String, SpriteRef >,
  }
}

mod_interface::mod_interface!
{
  exposed use InstanceHandle;
  exposed use ObjectHandle;
  exposed use StateHandle;
  exposed use Placement;
  exposed use Instance;
}
