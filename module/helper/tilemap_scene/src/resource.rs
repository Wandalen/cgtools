//! Asset / tint / animation / effect resources declared at the top of
//! `render_spec.ron` and referenced by id from elsewhere in the spec.
//!
//! See SPEC §4. Each resource kind has a stable id and zero or more
//! `*Ref(name)` wrappers that appear in object / layer definitions.

mod private
{
  use serde::{ Deserialize, Serialize };
  use rustc_hash::FxHashMap as HashMap;
  use tilemap_renderer::types::{ MipmapMode, SamplerFilter, WrapMode };

  // ============================================================================
  // Reference wrappers
  // ============================================================================

  /// Reference to a specific frame within a declared [`Asset`].
  ///
  /// The first field is the asset id, the second is a frame name or index
  /// resolved against the asset's layout (`Single` / `Atlas` / `SpriteSheet`).
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct SpriteRef( pub String, pub String );

  /// Reference to a declared [`Tint`] by id.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct TintRef( pub String );

  /// Reference to a declared [`Animation`] by id.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct AnimationRef( pub String );

  /// Reference to a declared [`Effect`] by id.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct EffectRef( pub String );

  // ============================================================================
  // Assets
  // ============================================================================

  /// A loadable image / atlas / sprite-sheet resource. See SPEC §4.1.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Asset
  {
    /// Unique id for this asset within the spec. Referenced via [`SpriteRef`].
    pub id : String,
    /// Path to the image file (relative to the spec's base directory).
    pub path : String,
    /// Layout of frames / sprites within this asset.
    pub kind : AssetKind,
    /// Texture sampling filter for this asset. Defaults to [`SamplerFilter::Linear`].
    /// Use [`SamplerFilter::Nearest`] for pixel art.
    #[ serde( default ) ]
    pub filter : SamplerFilter,
    /// Mipmap strategy. Defaults to [`MipmapMode::Off`]; enable for textures
    /// drawn at widely varying scales (parallax mountains, zoomed overworld).
    #[ serde( default ) ]
    pub mipmap : MipmapMode,
    /// Wrap mode along both U and V axes. Defaults to [`WrapMode::Clamp`].
    /// Set to [`WrapMode::Repeat`] for tileable seamless textures
    /// (sky backgrounds, ocean, long edge segments).
    #[ serde( default ) ]
    pub wrap : WrapMode,
  }

  /// How an asset is laid out internally.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum AssetKind
  {
    /// One image, one sprite — the whole PNG is a single frame.
    ///
    /// `size` is the image's pixel dimensions `( width, height )`; the
    /// compile layer returns it as the sprite's region regardless of what
    /// frame name is requested (so `SpriteRef(asset_id, anything)` works —
    /// by convention authors pass an empty string or the asset id itself).
    Single
    {
      /// Pixel dimensions of the image.
      size : ( u32, u32 ),
    },
    /// Grid atlas. Every sprite is a `tile_size` rectangle placed at some
    /// `( col, row )` in the grid.
    ///
    /// Frame names come from two sources:
    /// - **Numeric indices** (`"0"`, `"1"`, `"63"`) — resolved automatically
    ///   via `columns` using `col = idx % columns`, `row = idx / columns`.
    ///   Great for autotile atlases where frames are authored in mask order.
    /// - **Named frames** — declared in `frames` with their explicit
    ///   `( col, row )` cell. Use these for semantic names like
    ///   `"grass_edge_n"`, `"tri_gsw_0"`, etc.
    ///
    /// Named and numeric frames coexist: a lookup first checks `frames`, then
    /// falls back to numeric parsing. If neither resolves, compilation fails
    /// with a clear error (no silent placeholder regions).
    Atlas
    {
      /// Size of one tile in pixels, `( width, height )`.
      tile_size : ( u32, u32 ),
      /// Number of columns in the atlas grid — used by numeric-index lookups.
      columns : u32,
      /// Pixel offset from the image's top-left where the grid begins,
      /// `( x, y )`. Defaults to `( 0, 0 )`. Use to skip a leading border /
      /// margin baked into the atlas PNG.
      #[ serde( default ) ]
      origin : ( u32, u32 ),
      /// Inter-cell gap in pixels, `( horizontal, vertical )`. Defaults to
      /// `( 0, 0 )`. Full column stride is `tile_size.0 + gap.0`; row stride
      /// is `tile_size.1 + gap.1`.
      #[ serde( default ) ]
      gap : ( u32, u32 ),
      /// Named-frame manifest: `"name" -> ( col, row )`. Empty by default.
      #[ serde( default ) ]
      frames : HashMap< String, ( u32, u32 ) >,
      /// Explicit per-frame specifications: pixel rect plus optional anchor
      /// point (in pixels relative to the rect's top-left). Takes precedence
      /// over `frames` for the same name. Use this for irregular atlases
      /// (non-uniform row heights / pixel-precise anchor points per sprite).
      #[ serde( default ) ]
      frame_rects : HashMap< String, FrameSpec >,
    },
    /// Horizontal / vertical / grid sprite sheet for sequential animation frames.
    SpriteSheet
    {
      /// Total frame count in the sheet.
      frame_count : u32,
      /// Layout of the frames within the image.
      layout : SheetLayout,
    },
  }

  /// Per-frame entry for a non-uniform atlas.
  ///
  /// Every frame has an explicit pixel `rect` in the atlas. An optional
  /// `anchor` point (in pixels **relative to the rect's top-left**) tells
  /// the compile layer where the sprite's logical scene-anchor sits within
  /// the drawing. Useful for characters (anchor at feet), buildings (anchor
  /// at ground contact), or tiles whose visible centre isn't at the
  /// geometric rect centre. When `anchor` is `None` the compile layer falls
  /// back to the owning [`crate::object::Object`]'s
  /// normalized `pivot` field.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct FrameSpec
  {
    /// Pixel rectangle `[ x, y, w, h ]` in the atlas image.
    pub rect : [ u32; 4 ],
    /// Optional pixel-anchor inside the rect (0..w, 0..h). When present,
    /// overrides the object-level `pivot`; `scene_model::compile` places
    /// this anchor pixel exactly on the scene position (hex centre etc.).
    #[ serde( default ) ]
    pub anchor : Option< ( u32, u32 ) >,
  }

  /// Arrangement of frames inside a sprite-sheet asset.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum SheetLayout
  {
    /// Frames laid out left-to-right in a single row.
    Horizontal,
    /// Frames laid out top-to-bottom in a single column.
    Vertical,
    /// Frames laid out in a grid with `columns` frames per row.
    Grid
    {
      /// Number of frames per row.
      columns : u32,
    },
  }

  // ============================================================================
  // Tints
  // ============================================================================

  /// A named colour tint applied multiplicatively (or otherwise) to sprites.
  ///
  /// Referenced by [`TintRef`]. See SPEC §4.2 and §6.1.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Tint
  {
    /// Unique id for this tint within the spec.
    pub id : String,
    /// Colour as `"#rrggbb"` or `"#rrggbbaa"`.
    pub color : String,
    /// Strength `0.0..=1.0`: `0.0` = identity (no tint), `1.0` = full replacement.
    pub strength : f32,
    /// Blend mode when composing. Defaults to [`BlendMode::Multiply`].
    #[ serde( default = "default_blend_mode" ) ]
    pub mode : BlendMode,
  }

  #[ inline ]
  fn default_blend_mode() -> BlendMode { BlendMode::Multiply }

  /// Compositing modes for tint / layer composition. See SPEC §6.2.
  #[ derive( Debug, Clone, Copy, Default, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum BlendMode
  {
    /// Standard alpha-over compositing. The default.
    #[ default ]
    Normal,
    /// Multiplicative compositing: `dst = src * dst`.
    Multiply,
    /// Inverse multiply: `dst = 1 - ( 1 - src ) * ( 1 - dst )`.
    Screen,
    /// Additive compositing: `dst = src + dst` (clamped).
    Add,
    /// Combined multiply / screen per channel.
    Overlay,
  }

  // ============================================================================
  // Animations
  // ============================================================================

  /// A named purely-temporal sprite animation declared at the top level.
  ///
  /// Referenced by [`AnimationRef`]. See SPEC §4.3 and §7.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Animation
  {
    /// Unique id for this animation within the spec.
    pub id : String,
    /// Frame sequence: either regular (equal `fps`) or irregular (per-frame duration).
    pub timing : AnimationTiming,
    /// Playback mode.
    pub mode : AnimationMode,
    /// Per-instance phase offset strategy. See SPEC §7.1.
    #[ serde( default ) ]
    pub phase_offset : PhaseOffset,
  }

  /// Frame-sequence shape for an [`Animation`].
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum AnimationTiming
  {
    /// Equal-length frames listed explicitly, replayed at `fps`.
    ///
    /// Each frame is a full `( asset_id, frame_name )` pair — animations
    /// can draw frames from different assets if authoring demands it.
    Regular
    {
      /// Frames to cycle through.
      frames : Vec< SpriteRef >,
      /// Frames per second.
      fps : f32,
    },
    /// Equal-length frames drawn from a contiguous range of a `SpriteSheet` asset.
    ///
    /// Shorthand for `Regular` when the frames are consecutive sheet indices.
    FromSheet
    {
      /// Source asset id (must be a `SpriteSheet` kind).
      asset : String,
      /// Starting frame index within the sheet.
      start_frame : u32,
      /// Number of frames to consume.
      count : u32,
      /// Frames per second.
      fps : f32,
    },
    /// Per-frame duration — supports accented frames (e.g. hold the impact frame).
    Irregular
    {
      /// List of `( sprite_ref, duration_ms )` pairs.
      frames : Vec< TimedFrame >,
    },
  }

  /// One frame of an [`AnimationTiming::Irregular`] animation.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct TimedFrame
  {
    /// Which sprite to show during this frame.
    pub sprite : SpriteRef,
    /// How long to hold this frame, in milliseconds.
    pub duration_ms : u32,
  }

  /// Playback mode of an [`Animation`].
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum AnimationMode
  {
    /// Restart from frame 0 after the last frame.
    Loop,
    /// Alternate forward and backward through the frame list.
    PingPong,
    /// Play once and stop on the last frame.
    OneShot,
  }

  /// Per-instance animation time offset. See SPEC §7.1.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, Default ) ]
  #[ non_exhaustive ]
  pub enum PhaseOffset
  {
    /// No offset; all instances play at the master clock.
    #[ default ]
    None,
    /// Offset derived deterministically from the instance's grid coordinate
    /// (via [`crate::hash::hash_coord`]). Requires a grid-anchored anchor.
    HashCoord,
    /// Constant offset in seconds.
    Fixed( f32 ),
    /// Linear offset from grid coordinate: `phase = q * per_q + r * per_r`
    /// seconds. Use for traveling-wave effects where each row / column lags
    /// the previous by a fixed amount. Example: `per_q = 1.0 / fps, per_r = 0.0`
    /// shifts by one frame per step to the right.
    Linear
    {
      /// Seconds of phase shift added per unit of `q`.
      per_q : f32,
      /// Seconds of phase shift added per unit of `r`.
      per_r : f32,
    },
  }

  // ============================================================================
  // Effects
  // ============================================================================

  /// A shader-level procedural modifier. See SPEC §4.4.
  ///
  /// Effects don't produce sprite frames; they warp / modulate already-sampled
  /// pixels. Referenced by [`EffectRef`].
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct Effect
  {
    /// Unique id for this effect within the spec.
    pub id : String,
    /// The effect kind with its parameters.
    pub kind : EffectKind,
    /// Per-instance phase offset for the effect's own animation.
    #[ serde( default ) ]
    pub phase_offset : PhaseOffset,
  }

  /// The kinds of procedural effect available.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum EffectKind
  {
    /// Displace vertices sinusoidally along an axis (trees in wind).
    VertexDisplace
    {
      /// Axis of displacement.
      axis : Axis,
      /// Peak displacement amplitude in pixels.
      amplitude : f32,
      /// Oscillation frequency in Hz.
      frequency : f32,
    },
    /// Pulse the sprite's alpha between two values.
    AlphaPulse
    {
      /// Minimum alpha.
      min : f32,
      /// Maximum alpha.
      max : f32,
      /// Oscillation frequency in Hz.
      frequency : f32,
    },
    /// Modulate sprite colour toward a target colour over time.
    ColorShift
    {
      /// Target colour `"#rrggbb"`.
      target : String,
      /// Peak blend amount.
      amplitude : f32,
      /// Oscillation frequency in Hz.
      frequency : f32,
    },
  }

  /// Axis of a vertex displacement effect.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum Axis
  {
    /// Horizontal (screen-space X).
    X,
    /// Vertical (screen-space Y).
    Y,
  }
}

mod_interface::mod_interface!
{
  exposed use SpriteRef;
  exposed use TintRef;
  exposed use AnimationRef;
  exposed use EffectRef;
  exposed use Asset;
  exposed use AssetKind;
  exposed use FrameSpec;
  exposed use SheetLayout;
  exposed use Tint;
  exposed use BlendMode;
  exposed use Animation;
  exposed use AnimationTiming;
  exposed use TimedFrame;
  exposed use AnimationMode;
  exposed use PhaseOffset;
  exposed use Effect;
  exposed use EffectKind;
  exposed use Axis;
}
