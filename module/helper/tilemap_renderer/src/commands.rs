//! Render commands — all POD (Copy, no allocations).
//!
//! Commands form a flat sequential stream that backends process in order.
//! Streaming commands (BeginPath..EndPath, BeginText..EndText) maintain
//! state implicitly through ordering.

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
  pub transform : Transform,
  pub fill : FillRef,
  pub stroke_color : [ f32; 4 ],
  pub stroke_width : f32,
  pub stroke_cap : LineCap,
  pub stroke_join : LineJoin,
  pub stroke_dash : DashStyle,
  pub blend : BlendMode,
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
  pub cx : f32,
  pub cy : f32,
  pub x : f32,
  pub y : f32,
}

/// Cubic bezier. SVG: `C c1x c1y c2x c2y x y`.
#[ derive( Debug, Clone, Copy ) ]
pub struct CubicTo
{
  pub c1x : f32,
  pub c1y : f32,
  pub c2x : f32,
  pub c2y : f32,
  pub x : f32,
  pub y : f32,
}

/// Elliptical arc. SVG: `A rx ry rotation large_arc sweep x y`.
/// GPU: decompose into cubic beziers, then tessellate.
#[ derive( Debug, Clone, Copy ) ]
pub struct ArcTo
{
  pub rx : f32,
  pub ry : f32,
  pub rotation : f32,
  pub large_arc : bool,
  pub sweep : bool,
  pub x : f32,
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
  pub font : ResourceId< asset::Font >,
  pub size : f32,
  pub color : [ f32; 4 ],
  pub anchor : TextAnchor,
  pub position : [ f32; 2 ],
  /// If Some, text follows a path from Assets. SVG: `<textPath>`.
  pub along_path : Option< ResourceId< asset::Path > >,
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

/// Mesh with geometry from Assets.
/// SVG: `<polygon>` or `<path>` depending on topology.
/// GPU: vertex buffer + topology draw call.
#[ derive( Debug, Clone, Copy ) ]
pub struct Mesh
{
  pub transform : Transform,
  pub geometry : ResourceId< asset::Geometry >,
  pub fill : FillRef,
  /// Optional texture mapped via UV coordinates from GeometryAsset.
  /// GPU: binds texture, fragment shader samples using interpolated UVs.
  /// SVG: approximated as `<pattern>` fill over bounding box (no true UV mapping).
  pub texture : Option< ResourceId< asset::Image > >,
  pub topology : Topology,
  pub blend : BlendMode,
  pub clip : Option< ResourceId< asset::ClipMask > >,
}

/// Renders a sprite (sub-region of an image / sprite sheet).
/// SVG: `<use href="#sprite_N">` referencing a `<symbol viewBox="region">`.
/// GPU: textured quad with UV coordinates mapped to the sprite's region.
#[ derive( Debug, Clone, Copy ) ]
pub struct Sprite
{
  pub transform : Transform,
  /// References a SpriteAsset (which knows its sheet + region).
  pub sprite : ResourceId< asset::Sprite >,
  /// Tint color multiplied with texture color. White `[1,1,1,1]` = no tint.
  pub tint : [ f32; 4 ],
  pub blend : BlendMode,
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
  pub blend : BlendMode,
  pub clip : Option< ResourceId< asset::ClipMask > >,
}

/// Parameters for a mesh batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct MeshBatchParams
{
  /// Parent transform applied to all instances.
  pub transform : Transform,
  pub geometry : ResourceId< asset::Geometry >,
  pub fill : FillRef,
  pub texture : Option< ResourceId< asset::Image > >,
  pub topology : Topology,
  pub blend : BlendMode,
  pub clip : Option< ResourceId< asset::ClipMask > >,
}

/// Creates an empty sprite batch with the given parameters.
#[ derive( Debug, Clone, Copy ) ]
pub struct CreateSpriteBatch
{
  pub batch : ResourceId< Batch >,
  pub params : SpriteBatchParams,
}

/// Creates an empty mesh batch with the given parameters.
#[ derive( Debug, Clone, Copy ) ]
pub struct CreateMeshBatch
{
  pub batch : ResourceId< Batch >,
  pub params : MeshBatchParams,
}

/// Binds a batch for editing. All subsequent instance commands
/// apply to this batch until `UnbindBatch`.
#[ derive( Debug, Clone, Copy ) ]
pub struct BindBatch
{
  pub batch : ResourceId< Batch >,
}

/// Appends a new sprite instance to the bound batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct AddSpriteInstance
{
  pub transform : Transform,
  pub sprite : ResourceId< asset::Sprite >,
  pub tint : [ f32; 4 ],
}

/// Appends a new mesh instance to the bound batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct AddMeshInstance
{
  pub transform : Transform,
}

/// Updates a sprite instance at `index` within the bound batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct SetSpriteInstance
{
  pub index : u32,
  pub transform : Transform,
  pub sprite : ResourceId< asset::Sprite >,
  pub tint : [ f32; 4 ],
}

/// Updates a mesh instance at `index` within the bound batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct SetMeshInstance
{
  pub index : u32,
  pub transform : Transform,
}

/// Removes an instance at `index` from the bound batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct RemoveInstance
{
  pub index : u32,
}

/// Updates parameters of the bound sprite batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct SetSpriteBatchParams
{
  pub params : SpriteBatchParams,
}

/// Updates parameters of the bound mesh batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct SetMeshBatchParams
{
  pub params : MeshBatchParams,
}

/// Unbinds the batch and applies all pending changes.
#[ derive( Debug, Clone, Copy ) ]
pub struct UnbindBatch;

/// Draws a previously created batch (sprite or mesh).
/// GPU: single instanced draw call.
#[ derive( Debug, Clone, Copy ) ]
pub struct DrawBatch
{
  pub batch : ResourceId< Batch >,
}

/// Deletes a batch and frees its resources.
#[ derive( Debug, Clone, Copy ) ]
pub struct DeleteBatch
{
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
  Blur { radius : f32 },
  /// SVG: `feDropShadow`. GPU: blur + offset + composite.
  DropShadow { dx : f32, dy : f32, blur : f32, color : [ f32; 4 ] },
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
  pub transform : Transform,
  pub clip : Option< ResourceId< asset::ClipMask > >,
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
  Clear( Clear ),

  // Path
  BeginPath( BeginPath ),
  MoveTo( MoveTo ),
  LineTo( LineTo ),
  QuadTo( QuadTo ),
  CubicTo( CubicTo ),
  ArcTo( ArcTo ),
  ClosePath( ClosePath ),
  EndPath( EndPath ),

  // Text
  BeginText( BeginText ),
  Char( Char ),
  EndText( EndText ),

  // Single draw
  Mesh( Mesh ),
  Sprite( Sprite ),

  // Batch lifecycle
  CreateSpriteBatch( CreateSpriteBatch ),
  CreateMeshBatch( CreateMeshBatch ),
  BindBatch( BindBatch ),
  AddSpriteInstance( AddSpriteInstance ),
  AddMeshInstance( AddMeshInstance ),
  SetSpriteInstance( SetSpriteInstance ),
  SetMeshInstance( SetMeshInstance ),
  RemoveInstance( RemoveInstance ),
  SetSpriteBatchParams( SetSpriteBatchParams ),
  SetMeshBatchParams( SetMeshBatchParams ),
  UnbindBatch( UnbindBatch ),
  DrawBatch( DrawBatch ),
  DeleteBatch( DeleteBatch ),

  // Grouping
  BeginGroup( BeginGroup ),
  EndGroup( EndGroup ),
}
