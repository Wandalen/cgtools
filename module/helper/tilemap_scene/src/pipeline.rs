//! Render pipeline — ordered pipeline buckets, tiling strategy, global tint.
//!
//! See SPEC §8. The pipeline names the z-buckets objects draw into, the sort
//! mode within each bucket, and the tiling strategy used for neighbour / vertex
//! queries.

mod private
{
  use serde::{ Deserialize, Serialize };
  use crate::resource::TintRef;

  /// Top-level pipeline declaration embedded in [`crate::spec::RenderSpec`].
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct RenderPipeline
  {
    /// Grid geometry.
    pub hex : HexConfig,
    /// Pipeline z-buckets, rendered in declaration order (bottom to top).
    pub layers : Vec< PipelineLayer >,
    /// Optional global tint multiplied into every draw call. See SPEC §8.4.
    #[ serde( default ) ]
    pub global_tint : Option< TintRef >,
    /// Viewport size in screen pixels (width, height). Optional — when absent,
    /// backends derive it from the window / canvas.
    #[ serde( default ) ]
    pub viewport_size : Option< ( u32, u32 )>,
    /// Colour the framebuffer is cleared to at the start of every frame,
    /// as linear RGBA in `[0..1]`. When `None` (the default) the compile layer
    /// emits a transparent-black clear so the backend's own background shows
    /// through. Set this to give the scene a solid sky / sea / void colour
    /// without patching the emitted command stream.
    #[ serde( default ) ]
    pub clear_color : Option< [ f32; 4 ] >,
  }

  /// One pipeline bucket — a named z-layer objects can route draw calls into.
  #[ derive( Debug, Clone, Serialize, Deserialize ) ]
  pub struct PipelineLayer
  {
    /// Bucket id. Referenced by `Object.global_layer` and
    /// `ObjectLayer.pipeline_layer`.
    pub id : String,
    /// Sort mode applied to draw calls gathered in this bucket.
    #[ serde( default ) ]
    pub sort : SortMode,
    /// Optional per-bucket tint mask applied between per-object tints and
    /// the global tint. See SPEC §12.1 step 4.
    #[ serde( default ) ]
    pub tint_mask : Option< TintRef >,
    /// Coverage cut-off for this bucket's sprites: fragments whose sampled
    /// texture alpha is below `alpha_clip` are `discard`ed (no colour, no
    /// depth write). `0.0` (the default) disables the test, so every
    /// fragment is kept exactly as before.
    ///
    /// Combined with [`occlude_overlap`](Self::occlude_overlap) this turns a
    /// bucket of overlapping translucent tiles (e.g. a bled dual-grid drop
    /// shadow) into a single-coverage mask: the clip removes the soft AA
    /// fringe so the depth test can reject the overlap cleanly. Edges become
    /// hard at the clip contour — that is the intended trade for not
    /// double-blending the overlap.
    #[ serde( default ) ]
    pub alpha_clip : f32,
    /// When `true`, the backend clears the depth buffer immediately before
    /// this bucket draws and switches the depth test to `LESS` for its draw
    /// calls (restoring the default `LEQUAL` afterwards). The first fragment
    /// to cover a pixel wins; later overlapping fragments (from a sibling
    /// tile's bleed) are rejected, so a translucent bucket composites each
    /// pixel exactly once — no double-blend on the overlap.
    ///
    /// Safe because every *other* bucket draws painter's-style at depth `0`
    /// under `LEQUAL`: they never read the depth this bucket writes, and the
    /// pre-clear only discards depth they don't depend on. Pair with a
    /// non-zero [`alpha_clip`](Self::alpha_clip) so the AA fringe doesn't
    /// write depth and block a neighbour. Default `false` = unchanged.
    #[ serde( default ) ]
    pub occlude_overlap : bool,
    /// Marks this bucket as **fully opaque** (after `alpha_clip` coverage), so
    /// it joins the opaque depth-culling pass. When ANY layer in the pipeline
    /// sets this, the renderer splits the frame in two: opaque buckets draw
    /// first, **front-to-back** (topmost layer first) with depth writes on, so
    /// a nearer opaque layer early-Z rejects the covered fragments of farther
    /// ones — cutting the multi-layer overdraw of a flat tilemap (background
    /// under terrain under region). Transparent buckets (`false`, the default)
    /// then draw in painter's order with depth writes off, so blending is
    /// unaffected.
    ///
    /// Correctness requirements for an opaque layer:
    /// - It must be visually opaque where it covers (alpha ≈ 1); a translucent
    ///   layer meant to show what is beneath it must stay `false`.
    /// - Pair with a non-zero [`alpha_clip`](Self::alpha_clip) so transparent
    ///   texels `discard` (no depth write) and lower layers show through gaps.
    ///
    /// When no layer is opaque the renderer takes the original single-pass
    /// path unchanged, so this is inert for existing specs.
    #[ serde( default ) ]
    pub opaque : bool,
  }

  /// Sort mode for a [`PipelineLayer`]. See SPEC §8.2.
  #[ derive( Debug, Clone, Copy, Default, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum SortMode
  {
    /// Draw calls keep their emission order. Deterministic for static scenes.
    #[ default ]
    None,
    /// Sort by world X ascending — draws with smaller x render first (so
    /// draws on the right end up on top).
    XAsc,
    /// Sort by world X descending.
    XDesc,
    /// Sort by world Y ascending — objects lower on screen draw on top.
    YAsc,
    /// Sort by world Y descending.
    YDesc,
    /// Lexicographic: primary X ascending, tiebreaker Y descending. Gives
    /// left-to-right, top-to-bottom render order in world Y-up.
    XAscYDesc,
    /// Lexicographic: primary X ascending, tiebreaker Y ascending.
    XAscYAsc,
    /// Lexicographic: primary Y descending (painter's in Y-up — top-of-
    /// screen first, bottom-of-screen last), tiebreaker X ascending. Use
    /// this for isometric / zigzag hex stacks where a tile lower on
    /// screen should paint over tiles higher up regardless of their X.
    YDescXAsc,
    /// Lexicographic: primary Y ascending, tiebreaker X ascending.
    YAscXAsc,
  }

  /// Hex / grid geometry configuration. See SPEC §2.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize ) ]
  pub struct HexConfig
  {
    /// Which tiling strategy the scene uses.
    pub tiling : TilingStrategy,
    /// Pixel stride between the centres of adjacent cells along the grid's
    /// primary axes, `( x_stride, y_stride )`. For a flat-top hex this is the
    /// `q`-axis step in X and the `r`-axis step in Y (the latter is scaled by
    /// `sqrt(3)/2` internally to preserve equilateral geometry); pointy-top
    /// swaps the roles. For equilateral hex sprites the stride equals the
    /// sprite's visible bounding box; for stylised / non-equilateral art it
    /// is whatever spacing makes neighbours tile without gaps or overlap.
    pub grid_stride : ( u32, u32 ),
  }

  impl HexConfig
  {
    /// Convenience constructor for the common case: an equilateral hex
    /// sprite with pixel bounding box `( w, h )`. Computes
    /// `grid_stride` so adjacent cells tile without gap or overlap.
    ///
    /// Flat-top: the `q`-axis (horizontal) stride is `¾·w`, the
    /// `r`-axis stride is `h` (the `sqrt(3)/2` factor is applied
    /// inside `hex_to_world_pixel_flat`). Pointy-top swaps the axes.
    ///
    /// For stylised pixel-art hexes whose visible silhouette is not a
    /// perfect equilateral triangle ratio, construct `HexConfig`
    /// directly with the empirically-tuned `grid_stride` instead.
    ///
    /// `Square4` / `Square8` are accepted without panic (stride
    /// defaults to `(w, h)`) but are not yet implemented. Load-time
    /// rejection of unsupported tilings is a tracked TODO in
    /// [`crate::validate`] (SPEC §16), so [`crate::load::load`]
    /// currently returns `Ok( () )` for square specs; compilation
    /// later fails at render time with
    /// [`crate::compile::CompileError::UnsupportedAnchor`].
    #[ inline ]
    #[ must_use ]
    pub fn from_hex_size( w : u32, h : u32, tiling : TilingStrategy ) -> Self
    {
      let grid_stride = match tiling
      {
        TilingStrategy::HexFlatTop   => ( w * 3 / 4, h ),
        TilingStrategy::HexPointyTop => ( w, h * 3 / 4 ),
        // Square tilings are not yet implemented; we still produce a
        // sane default (1:1 with the bounding box) rather than panic so
        // callers exploring the API don't trip a destructive failure.
        TilingStrategy::Square4 | TilingStrategy::Square8 => ( w, h ),
      };
      Self { tiling, grid_stride }
    }
  }

  /// Tiling strategy — determines neighbour ordering, dual-mesh shape, and
  /// pixel-conversion. See SPEC §2.1.
  ///
  /// Version 0.2.0 implements the two hex variants; the square variants are
  /// reserved. Load-time validation of [`TilingStrategy`] is a tracked TODO
  /// in [`crate::validate`] (SPEC §16) — [`crate::error::ValidationError::UnsupportedTiling`]
  /// is declared for that future check but is not yet constructed.
  /// Square specs therefore pass [`crate::load::load`] today and fail at
  /// render time with [`crate::compile::CompileError::UnsupportedAnchor`].
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
  #[ non_exhaustive ]
  pub enum TilingStrategy
  {
    /// Flat-top hex — six neighbours, dual-mesh triangles.
    HexFlatTop,
    /// Pointy-top hex — six neighbours, dual-mesh triangles.
    HexPointyTop,
    /// 4-neighbour square grid (reserved, not implemented).
    Square4,
    /// 8-neighbour square grid (reserved, not implemented).
    Square8,
  }
}

mod_interface::mod_interface!
{
  exposed use RenderPipeline;
  exposed use PipelineLayer;
  exposed use SortMode;
  exposed use HexConfig;
  exposed use TilingStrategy;
}
