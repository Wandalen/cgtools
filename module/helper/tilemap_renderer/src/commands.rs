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
  pub cx : f32, pub cy : f32,
  pub x : f32, pub y : f32,
}

/// Cubic bezier. SVG: `C c1x c1y c2x c2y x y`.
#[ derive( Debug, Clone, Copy ) ]
pub struct CubicTo
{
  pub c1x : f32, pub c1y : f32,
  pub c2x : f32, pub c2y : f32,
  pub x : f32, pub y : f32,
}

/// Elliptical arc. SVG: `A rx ry rotation large_arc sweep x y`.
/// GPU: decompose into cubic beziers, then tessellate.
#[ derive( Debug, Clone, Copy ) ]
pub struct ArcTo
{
  pub rx : f32, pub ry : f32,
  pub rotation : f32,
  pub large_arc : bool,
  pub sweep : bool,
  pub x : f32, pub y : f32,
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
// Mesh & Sprite
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
// Instancing (streaming)
// ============================================================================

/// Instanced mesh — one geometry, many transforms.
/// SVG: `<defs>` + `<use>` per instance.
/// GPU: instanced draw call.
#[ derive( Debug, Clone, Copy ) ]
pub struct BeginInstancedMesh
{
  pub geometry : ResourceId< asset::Geometry >,
  pub fill : FillRef,
  pub topology : Topology,
  pub clip : Option< ResourceId< asset::ClipMask > >,
}

/// One instance transform. POD.
#[ derive( Debug, Clone, Copy ) ]
pub struct Instance
{
  pub transform : Transform,
}

/// Ends instanced mesh sequence.
#[ derive( Debug, Clone, Copy ) ]
pub struct EndInstancedMesh;

/// Batched sprite rendering — many sprites from the same sheet in one batch.
/// Each SpriteInstance can reference a different region (SpriteAsset) within the sheet.
/// SVG: one `<symbol>` per unique sprite + `<use>` per instance.
/// GPU: single draw call — one texture bind, per-instance UV + transform in instance buffer.
#[ derive( Debug, Clone, Copy ) ]
pub struct BeginInstancedSprite
{
  /// The sprite sheet image. All SpriteInstances must reference sprites from this sheet.
  pub sheet : ResourceId< asset::Image >,
  pub blend : BlendMode,
  pub clip : Option< ResourceId< asset::ClipMask > >,
}

/// One sprite instance within a batch. POD.
#[ derive( Debug, Clone, Copy ) ]
pub struct SpriteInstance
{
  pub transform : Transform,
  /// References a SpriteAsset defining the region within the sheet.
  pub sprite : ResourceId< asset::Sprite >,
  /// Tint color. White `[1,1,1,1]` = no tint.
  pub tint : [ f32; 4 ],
}

/// Ends instanced sprite batch.
#[ derive( Debug, Clone, Copy ) ]
pub struct EndInstancedSprite;

// ============================================================================
// Batch recording (persistent instance buffer)
// ============================================================================

/// Begins recording a persistent batch.
/// SpriteInstance commands between Begin/End are stored in the backend
/// under the given ResourceId<Batch>, not drawn immediately.
///
/// GPU: allocates instance buffer, fills with subsequent SpriteInstance data.
/// SVG: stores instance list keyed by batch id.
#[ derive( Debug, Clone, Copy ) ]
pub struct BeginRecordBatch
{
  pub batch : ResourceId< Batch >,
  pub sheet : ResourceId< asset::Image >,
  pub blend : BlendMode,
  pub clip : Option< ResourceId< asset::ClipMask > >,
}

/// Ends batch recording. The batch is now ready to be drawn via `BatchBackend::draw_batch`.
#[ derive( Debug, Clone, Copy ) ]
pub struct EndRecordBatch;

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

  // Mesh & Sprite
  Mesh( Mesh ),
  Sprite( Sprite ),

  // Instancing
  BeginInstancedMesh( BeginInstancedMesh ),
  Instance( Instance ),
  EndInstancedMesh( EndInstancedMesh ),
  BeginInstancedSprite( BeginInstancedSprite ),
  SpriteInstance( SpriteInstance ),
  EndInstancedSprite( EndInstancedSprite ),

  // Batch recording
  BeginRecordBatch( BeginRecordBatch ),
  // SpriteInstance commands go here — reused from instancing
  EndRecordBatch( EndRecordBatch ),

  // Grouping
  BeginGroup( BeginGroup ),
  EndGroup( EndGroup ),
}
