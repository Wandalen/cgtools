//! Render commands — all POD (Copy, no allocations).
//!
//! Commands form a flat sequential stream that backends process in order.
//! Streaming commands (BeginPath..EndPath, BeginText..EndText) maintain
//! state implicitly through ordering.

mod private
{
  use crate::types::*;
  use crate::types::asset;

  // ============================================================================
  // Clear
  // ============================================================================

  /// Clears the framebuffer / canvas with a solid color.
  /// SVG: `<rect width="100%" height="100%" fill="..."/>`.
  /// GPU: `clear_color` on the render pass.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Clear
  {
    /// Clear color as RGBA.
    pub color : [ f32; 4 ],
  }

  // ============================================================================
  // Path commands (unified primitive for lines, curves, arcs)
  // ============================================================================

  /// Begins a new path with styling.
  /// SVG: opens `<path d="...">` with fill/stroke attributes.
  /// GPU: begins collecting vertices for tessellation.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct BeginPath
  {
    /// Transform applied to the path.
    pub transform : Transform,
    /// Fill style for the path interior.
    pub fill : FillRef,
    /// Stroke color as RGBA.
    pub stroke_color : [ f32; 4 ],
    /// Stroke width in pixels.
    pub stroke_width : f32,
    /// Line cap style.
    pub stroke_cap : LineCap,
    /// Line join style.
    pub stroke_join : LineJoin,
    /// Dash pattern style.
    pub stroke_dash : DashStyle,
    /// Blend mode for compositing.
    pub blend : BlendMode,
    /// Optional clip mask.
    pub clip : Option< ResourceId< asset::ClipMask > >,
  }

  /// SVG: `M x y`.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct MoveTo( pub f32, pub f32 );

  /// SVG: `L x y`.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct LineTo( pub f32, pub f32 );

  /// Quadratic bezier. SVG: `Q cx cy x y`.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct QuadTo
  {
    /// Control point X.
    pub cx : f32,
    /// Control point Y.
    pub cy : f32,
    /// End point X.
    pub x : f32,
    /// End point Y.
    pub y : f32,
  }

  /// Cubic bezier. SVG: `C c1x c1y c2x c2y x y`.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct CubicTo
  {
    /// First control point X.
    pub c1x : f32,
    /// First control point Y.
    pub c1y : f32,
    /// Second control point X.
    pub c2x : f32,
    /// Second control point Y.
    pub c2y : f32,
    /// End point X.
    pub x : f32,
    /// End point Y.
    pub y : f32,
  }

  /// Elliptical arc. SVG: `A rx ry rotation large_arc sweep x y`.
  /// GPU: decompose into cubic beziers, then tessellate.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct ArcTo
  {
    /// Ellipse radius X.
    pub rx : f32,
    /// Ellipse radius Y.
    pub ry : f32,
    /// Ellipse rotation in radians.
    pub rotation : f32,
    /// Large arc flag.
    pub large_arc : bool,
    /// Sweep direction flag.
    pub sweep : bool,
    /// End point X.
    pub x : f32,
    /// End point Y.
    pub y : f32,
  }

  /// Closes the current subpath. SVG: `Z`.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct ClosePath;

  /// Ends path, flushes to backend.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct EndPath;

  // ============================================================================
  // Text commands (streaming, POD)
  // ============================================================================

  /// Begins text rendering.
  /// GPU: CPU text layout (optionally along path), then render glyphs.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct BeginText
  {
    /// Font resource.
    pub font : ResourceId< asset::Font >,
    /// Font size in pixels.
    pub size : f32,
    /// Text color as RGBA.
    pub color : [ f32; 4 ],
    /// Text anchor alignment.
    pub anchor : TextAnchor,
    /// Position as `[x, y]`.
    pub position : [ f32; 2 ],
    /// If Some, text follows a path from Assets. SVG: `<textPath>`.
    pub along_path : Option< ResourceId< asset::Path > >,
    /// Optional clip mask.
    pub clip : Option< ResourceId< asset::ClipMask > >,
  }

  /// Single character. POD.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Char( pub char );

  /// Ends text sequence.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct EndText;

  // ============================================================================
  // Mesh & Sprite (single draw)
  // ============================================================================

  /// Mesh with geometry from `Assets`.
  /// SVG: `<polygon>` or `<path>` depending on topology.
  /// GPU: vertex buffer + topology draw call.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Mesh
  {
    /// Transform applied to the mesh.
    pub transform : Transform,
    /// Geometry resource.
    pub geometry : ResourceId< asset::Geometry >,
    /// Fill style.
    pub fill : FillRef,
    /// Optional texture mapped via UV coordinates from `GeometryAsset`.
    /// GPU: binds texture, fragment shader samples using interpolated UVs.
    /// SVG: approximated as `<pattern>` fill over bounding box (no true UV mapping).
    pub texture : Option< ResourceId< asset::Image > >,
    /// Primitive topology.
    pub topology : Topology,
    /// Blend mode for compositing.
    pub blend : BlendMode,
    /// Optional clip mask.
    pub clip : Option< ResourceId< asset::ClipMask > >,
  }

  /// Renders a sprite (sub-region of an image / sprite sheet).
  /// SVG: `<use href="#sprite_N">` referencing a `<symbol viewBox="region">`.
  /// GPU: textured quad with UV coordinates mapped to the sprite's region.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Sprite
  {
    /// Transform applied to the sprite.
    pub transform : Transform,
    /// References a `SpriteAsset` (which knows its sheet + region).
    pub sprite : ResourceId< asset::Sprite >,
    /// Tint color multiplied with texture color. White `[1,1,1,1]` = no tint.
    pub tint : [ f32; 4 ],
    /// Blend mode for compositing.
    pub blend : BlendMode,
    /// Optional clip mask.
    pub clip : Option< ResourceId< asset::ClipMask > >,
  }

  // ============================================================================
  // Batch commands
  // ============================================================================

  /// Parameters for a sprite batch.
  ///
  /// ```ignore
  /// // Create + populate
  /// cmd( CreateSpriteBatch { batch: TILES, params: SpriteBatchParams { .. } } );
  /// cmd( BindBatch { batch: TILES } );
  /// cmd( AddSpriteInstance { transform: .., sprite: grass, tint: WHITE } );
  /// cmd( UnbindBatch );
  ///
  /// // Draw every frame
  /// cmd( DrawBatch { batch: TILES } );
  ///
  /// // Update later
  /// cmd( BindBatch { batch: TILES } );
  /// cmd( SetSpriteInstance { index: 42, transform: .., sprite: water, tint: WHITE } );
  /// // RemoveInstance uses swap-remove: the last instance moves into slot 5.
  /// // Update your entity→index map accordingly.
  /// cmd( RemoveInstance { index: 5 } );
  /// cmd( UnbindBatch );
  /// ```
  #[ derive( Debug, Clone, Copy ) ]
  pub struct SpriteBatchParams
  {
    /// Parent transform applied to all instances.
    pub transform : Transform,
    /// The sprite sheet image. All instances must reference sprites from this sheet.
    pub sheet : ResourceId< asset::Image >,
    /// Blend mode for compositing.
    pub blend : BlendMode,
    /// Optional clip mask.
    pub clip : Option< ResourceId< asset::ClipMask > >,
  }

  /// Parameters for a mesh batch.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct MeshBatchParams
  {
    /// Parent transform applied to all instances.
    pub transform : Transform,
    /// Geometry resource.
    pub geometry : ResourceId< asset::Geometry >,
    /// Fill style.
    pub fill : FillRef,
    /// Optional texture.
    pub texture : Option< ResourceId< asset::Image > >,
    /// Primitive topology.
    pub topology : Topology,
    /// Blend mode for compositing.
    pub blend : BlendMode,
    /// Optional clip mask.
    pub clip : Option< ResourceId< asset::ClipMask > >,
  }

  /// Creates an empty sprite batch with the given parameters.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct CreateSpriteBatch
  {
    /// Batch resource identifier.
    pub batch : ResourceId< Batch >,
    /// Sprite batch parameters.
    pub params : SpriteBatchParams,
  }

  /// Creates an empty mesh batch with the given parameters.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct CreateMeshBatch
  {
    /// Batch resource identifier.
    pub batch : ResourceId< Batch >,
    /// Mesh batch parameters.
    pub params : MeshBatchParams,
  }

  /// Binds a batch for editing. All subsequent instance commands
  /// (`AddSpriteInstance`, `SetSpriteInstance`, `RemoveInstance`, etc.)
  /// apply to this batch until `UnbindBatch`.
  ///
  /// # Invariants
  ///
  /// - Only **one** batch can be bound at a time. Issuing `BindBatch` while
  ///   another batch is already bound is a protocol error; the WebGL backend
  ///   returns `RenderError::BackendError`.
  /// - Always pair every `BindBatch` with a matching `UnbindBatch` before
  ///   issuing `DrawBatch` or a second `BindBatch`.
  ///
  /// **Correct lifecycle:**
  /// ```ignore
  /// BindBatch(id)
  /// Add/Set/RemoveInstance …
  /// UnbindBatch          // commits VAO state; safe to draw after this
  /// DrawBatch(id)
  /// ```
  #[ derive( Debug, Clone, Copy ) ]
  pub struct BindBatch
  {
    /// Batch resource identifier.
    pub batch : ResourceId< Batch >,
  }

  /// Appends a new sprite instance to the bound batch.
  ///
  /// # Errors
  ///
  /// If the internal GPU buffer needs to grow and the allocation fails,
  /// `submit` returns `RenderError::BackendError`.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct AddSpriteInstance
  {
    /// Instance transform.
    pub transform : Transform,
    /// Sprite resource.
    pub sprite : ResourceId< asset::Sprite >,
    /// Tint color as RGBA.
    pub tint : [ f32; 4 ],
  }

  /// Appends a new mesh instance to the bound batch.
  ///
  /// # Errors
  ///
  /// If the internal GPU buffer needs to grow and the allocation fails,
  /// `submit` returns `RenderError::BackendError`.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct AddMeshInstance
  {
    /// Instance transform.
    pub transform : Transform,
  }

  /// Updates a sprite instance at `index` within the bound batch.
  ///
  /// # Errors
  ///
  /// If `index >= batch.len()`, `submit` returns `RenderError::BackendError`.
  /// Stale indices are easy to introduce after `RemoveInstance` (swap-remove
  /// shifts the last element into the removed slot) — always update your
  /// entity→index map after every removal.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct SetSpriteInstance
  {
    /// Instance index.
    pub index : u32,
    /// Updated transform.
    pub transform : Transform,
    /// Sprite resource.
    pub sprite : ResourceId< asset::Sprite >,
    /// Tint color as RGBA.
    pub tint : [ f32; 4 ],
  }

  /// Updates a mesh instance at `index` within the bound batch.
  ///
  /// # Errors
  ///
  /// If `index >= batch.len()`, `submit` returns `RenderError::BackendError`.
  /// See `SetSpriteInstance` for notes on stale indices after swap-remove.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct SetMeshInstance
  {
    /// Instance index.
    pub index : u32,
    /// Updated transform.
    pub transform : Transform,
  }

  /// Removes an instance at `index` from the bound batch using **swap-remove**.
  ///
  /// The last instance is moved into slot `index` to fill the gap.
  /// If you maintain an external mapping of entity → instance index, you must
  /// update it after removal: the entity that previously occupied the last slot
  /// now lives at `index`.
  ///
  /// If `index` is already the last element no swap occurs — it is simply dropped.
  ///
  /// # Errors
  ///
  /// If `index >= batch.len()`, `submit` returns `RenderError::BackendError`.
  ///
  /// # Example
  /// ```ignore
  /// // Before: [A, B, C, D]  (len = 4)
  /// cmd( RemoveInstance { index: 1 } ); // remove B
  /// // After:  [A, D, C]     (len = 3) — D moved from index 3 to index 1
  /// // → update your map: entity_D.index = 1
  /// ```
  #[ derive( Debug, Clone, Copy ) ]
  pub struct RemoveInstance
  {
    /// Index of the instance to remove.
    pub index : u32,
  }

  /// Updates parameters of the bound sprite batch.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct SetSpriteBatchParams
  {
    /// Updated sprite batch parameters.
    pub params : SpriteBatchParams,
  }

  /// Updates parameters of the bound mesh batch.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct SetMeshBatchParams
  {
    /// Updated mesh batch parameters.
    pub params : MeshBatchParams,
  }

  /// Unbinds the current batch and commits all pending instance changes.
  ///
  /// In the WebGL backend, `UnbindBatch` re-configures the batch's VAO with
  /// the current instance buffer. This step is **required** if any `AddInstance`
  /// calls caused the internal buffer to grow (reallocate), because a grow
  /// replaces the underlying `WebGlBuffer` and invalidates the previous VAO
  /// attribute pointers.
  ///
  /// Calling `UnbindBatch` when no batch is bound is a no-op.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct UnbindBatch;

  /// Draws a previously created batch (sprite or mesh).
  /// GPU: single instanced draw call.
  ///
  /// # Invariants
  ///
  /// The batch must **not** be bound at the time of this command. Calling
  /// `DrawBatch` while the batch is still bound (i.e. without a preceding
  /// `UnbindBatch`) is a protocol error; the WebGL backend returns
  /// `RenderError::BackendError`.
  ///
  /// This restriction exists because `UnbindBatch` is responsible for
  /// refreshing the VAO when the instance buffer grew during recording.
  /// Drawing with a stale VAO produces undefined GPU behavior.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct DrawBatch
  {
    /// Batch resource identifier.
    pub batch : ResourceId< Batch >,
  }

  /// Deletes a batch and frees its GPU resources.
  ///
  /// If the batch is currently bound when this command is processed, the
  /// WebGL backend implicitly clears the binding (equivalent to `UnbindBatch`
  /// without a VAO refresh, since the batch is about to be destroyed).
  /// Subsequent instance commands that would have targeted this batch become
  /// no-ops.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct DeleteBatch
  {
    /// Batch resource identifier.
    pub batch : ResourceId< Batch >,
  }

  // ============================================================================
  // Effects
  // ============================================================================

  /// Visual effect applied to a group.
  /// SVG: `<filter>` with corresponding `fe*` element.
  /// GPU: render to offscreen texture, apply post-process shader, composite.
  #[ derive( Debug, Clone, Copy ) ]
  pub enum Effect
  {
    /// SVG: `feGaussianBlur`. GPU: separable gaussian blur shader.
    Blur
    {
      /// Blur radius in pixels.
      radius : f32,
    },
    /// SVG: `feDropShadow`. GPU: blur + offset + composite.
    DropShadow
    {
      /// Shadow offset X.
      dx : f32,
      /// Shadow offset Y.
      dy : f32,
      /// Shadow blur radius.
      blur : f32,
      /// Shadow color as RGBA.
      color : [ f32; 4 ],
    },
    /// SVG: `feColorMatrix`. GPU: 4x5 matrix in fragment shader.
    ColorMatrix( [ f32; 20 ] ),
    /// SVG: `opacity`. GPU: alpha multiply.
    Opacity( f32 ),
  }

  // ============================================================================
  // Grouping
  // ============================================================================

  /// Begins a group with shared transform/clip/effect.
  /// SVG: `<g>`. GPU: push state stack.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct BeginGroup
  {
    /// Group transform.
    pub transform : Transform,
    /// Optional clip mask.
    pub clip : Option< ResourceId< asset::ClipMask > >,
    /// Optional visual effect.
    pub effect : Option< Effect >,
  }

  /// Ends group. SVG: `</g>`. GPU: pop state stack.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct EndGroup;

  // ============================================================================
  // Unified command enum
  // ============================================================================

  /// A single render command. All variants are POD (Copy, no allocations).
  /// Backends process these sequentially from a `&[RenderCommand]` slice.
  #[ derive( Debug, Clone, Copy ) ]
  pub enum RenderCommand
  {
    /// Clear the framebuffer.
    Clear( Clear ),

    /// Begin a new path.
    BeginPath( BeginPath ),
    /// Move to a point.
    MoveTo( MoveTo ),
    /// Line to a point.
    LineTo( LineTo ),
    /// Quadratic bezier curve.
    QuadTo( QuadTo ),
    /// Cubic bezier curve.
    CubicTo( CubicTo ),
    /// Elliptical arc.
    ArcTo( ArcTo ),
    /// Close the current subpath.
    ClosePath( ClosePath ),
    /// End the current path.
    EndPath( EndPath ),

    /// Begin text rendering.
    BeginText( BeginText ),
    /// A single character.
    Char( Char ),
    /// End text rendering.
    EndText( EndText ),

    /// Draw a mesh.
    Mesh( Mesh ),
    /// Draw a sprite.
    Sprite( Sprite ),

    /// Create a sprite batch.
    CreateSpriteBatch( CreateSpriteBatch ),
    /// Create a mesh batch.
    CreateMeshBatch( CreateMeshBatch ),
    /// Bind a batch for editing.
    BindBatch( BindBatch ),
    /// Add a sprite instance to the bound batch.
    AddSpriteInstance( AddSpriteInstance ),
    /// Add a mesh instance to the bound batch.
    AddMeshInstance( AddMeshInstance ),
    /// Update a sprite instance in the bound batch.
    SetSpriteInstance( SetSpriteInstance ),
    /// Update a mesh instance in the bound batch.
    SetMeshInstance( SetMeshInstance ),
    /// Remove an instance from the bound batch.
    RemoveInstance( RemoveInstance ),
    /// Update sprite batch parameters.
    SetSpriteBatchParams( SetSpriteBatchParams ),
    /// Update mesh batch parameters.
    SetMeshBatchParams( SetMeshBatchParams ),
    /// Unbind the current batch.
    UnbindBatch( UnbindBatch ),
    /// Draw a batch.
    DrawBatch( DrawBatch ),
    /// Delete a batch.
    DeleteBatch( DeleteBatch ),

    /// Begin a group.
    BeginGroup( BeginGroup ),
    /// End a group.
    EndGroup( EndGroup ),
  }

}


mod_interface::mod_interface!
{
  own use Clear;
  own use BeginPath;
  own use MoveTo;
  own use LineTo;
  own use QuadTo;
  own use CubicTo;
  own use ArcTo;
  own use ClosePath;
  own use EndPath;
  own use BeginText;
  own use Char;
  own use EndText;
  own use Mesh;
  own use Sprite;
  own use SpriteBatchParams;
  own use MeshBatchParams;
  own use CreateSpriteBatch;
  own use CreateMeshBatch;
  own use BindBatch;
  own use AddSpriteInstance;
  own use AddMeshInstance;
  own use SetSpriteInstance;
  own use SetMeshInstance;
  own use RemoveInstance;
  own use SetSpriteBatchParams;
  own use SetMeshBatchParams;
  own use UnbindBatch;
  own use DrawBatch;
  own use DeleteBatch;
  own use Effect;
  own use BeginGroup;
  own use EndGroup;
  own use RenderCommand;
}
