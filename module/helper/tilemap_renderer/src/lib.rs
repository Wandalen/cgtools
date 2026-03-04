//! Agnostic 2D rendering engine.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

// Prevent "unused" warnings when features are disabled
#![ cfg_attr( not( feature = "std" ), allow( unused ) ) ]

use std::path::PathBuf;

// Module declarations - using ultra-granular feature gating
#[ cfg( any( feature = "scene-container", feature = "scene-methods" ) ) ]
pub mod scene;

#[ cfg( any(
  feature = "command-line",
  feature = "command-curve",
  feature = "command-text",
  feature = "command-tilemap",
  feature = "command-particle",
  feature = "commands"
) ) ]
pub mod commands;

#[ cfg( any(
  feature = "traits-renderer",
  feature = "traits-primitive",
  feature = "traits-async",
  feature = "ports"
) ) ]
pub mod ports;

#[ cfg( any(
  feature = "adapter-svg-basic",
  feature = "adapter-svg",
  feature = "adapter-svg-browser",
  feature = "adapter-webgl",
  feature = "adapter-webgpu",
  feature = "adapter-terminal-basic",
  feature = "adapter-terminal",
  feature = "adapter-wgpu"
) ) ]
pub mod adapters;

#[ cfg( any(
  feature = "query-basic",
  feature = "query-by-type",
  feature = "query-predicate",
  feature = "query"
) ) ]
pub mod query;

#[ cfg( any(
  feature = "cli-basic",
  feature = "cli-commands",
  feature = "cli-repl",
  feature = "cli"
) ) ]
pub mod cli;

// ============================================================================
// Resource ID
// ============================================================================

/// Handle to a resource stored in Assets. POD, Copy.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
struct ResourceId( u32 );

// ============================================================================
// Commands — all POD (Copy, no allocations)
// ============================================================================

// -- Clear --

/// Clears the framebuffer / canvas with a solid color.
/// SVG: `<rect width="100%" height="100%" fill="..."/>`.
/// GPU: `clear_color` on the render pass.
#[ derive( Debug, Clone, Copy ) ]
struct Clear
{
  color : [ f32; 4 ],
}

// -- Path (unified primitive for lines, curves, arcs) --

/// Begins a new path with styling.
/// SVG: opens `<path d="...">` with fill/stroke attributes.
/// GPU: begins collecting vertices for tessellation.
#[ derive( Debug, Clone, Copy ) ]
struct BeginPath
{
  transform : Transform,
  fill : FillRef,
  stroke_color : [ f32; 4 ],
  stroke_width : f32,
  stroke_cap : LineCap,
  stroke_join : LineJoin,
  stroke_dash : DashStyle,
  blend : BlendMode,
  clip : Option< ResourceId >,
}

/// SVG: `M x y`
#[ derive( Debug, Clone, Copy ) ]
struct MoveTo( f32, f32 );

/// SVG: `L x y`
#[ derive( Debug, Clone, Copy ) ]
struct LineTo( f32, f32 );

/// Quadratic bezier. SVG: `Q cx cy x y`
#[ derive( Debug, Clone, Copy ) ]
struct QuadTo
{
  cx : f32, cy : f32,
  x : f32, y : f32,
}

/// Cubic bezier. SVG: `C c1x c1y c2x c2y x y`
#[ derive( Debug, Clone, Copy ) ]
struct CubicTo
{
  c1x : f32, c1y : f32,
  c2x : f32, c2y : f32,
  x : f32, y : f32,
}

/// Elliptical arc. SVG: `A rx ry rotation large_arc sweep x y`
/// GPU: decompose into cubic beziers, then tessellate.
#[ derive( Debug, Clone, Copy ) ]
struct ArcTo
{
  rx : f32, ry : f32,
  rotation : f32,
  large_arc : bool,
  sweep : bool,
  x : f32, y : f32,
}

/// SVG: `Z` (close subpath)
#[ derive( Debug, Clone, Copy ) ]
struct ClosePath;

/// Ends path, flushes to backend.
#[ derive( Debug, Clone, Copy ) ]
struct EndPath;

// -- Text (streaming, POD) --

/// Begins text rendering.
/// `path` — if Some, text follows a path from Assets (SVG: `<textPath>`).
/// GPU: CPU text layout along path, then render glyphs.
#[ derive( Debug, Clone, Copy ) ]
struct BeginText
{
  font : ResourceId,
  size : f32,
  color : [ f32; 4 ],
  anchor : TextAnchor,
  position : [ f32; 2 ],
  along_path : Option< ResourceId >,  // text on curve (SVG: <textPath>)
  clip : Option< ResourceId >,
}

/// Single character. POD.
#[ derive( Debug, Clone, Copy ) ]
struct Char( char );

/// Ends text sequence.
#[ derive( Debug, Clone, Copy ) ]
struct EndText;

// -- Mesh --

#[ derive( Debug, Clone, Copy ) ]
struct Transform
{
  position : [ f32; 2 ],
  rotation : f32,
  scale : [ f32; 2 ],
  /// Skew in radians. SVG: `skewX()` / `skewY()`. GPU: added to affine matrix.
  skew : [ f32; 2 ],
}

/// Mesh with geometry from Assets.
/// SVG: renders as `<polygon>` or `<path>` depending on topology.
/// GPU: draws with vertex buffer + topology.
#[ derive( Debug, Clone, Copy ) ]
struct Mesh
{
  transform : Transform,
  geometry : ResourceId,
  fill : FillRef,
  topology : Topology,
  blend : BlendMode,
  clip : Option< ResourceId >,
}

/// Renders a sprite (sub-region of an image / sprite sheet). POD.
/// SVG: `<use href="#sprite_N">` referencing a `<symbol viewBox="region">`.
/// GPU: textured quad with UV coordinates mapped to the sprite's region within the atlas.
#[ derive( Debug, Clone, Copy ) ]
struct Sprite
{
  transform : Transform,
  /// References a SpriteAsset (which knows its sheet + region).
  sprite : ResourceId,
  /// Optional tint color (multiplied with texture color). White = no tint.
  tint : [ f32; 4 ],
  blend : BlendMode,
  clip : Option< ResourceId >,
}

/// Instanced mesh — one geometry, many transforms.
/// SVG: `<defs>` + `<use>` per instance.
/// GPU: instanced draw call.
#[ derive( Debug, Clone, Copy ) ]
struct BeginInstancedMesh
{
  geometry : ResourceId,
  fill : FillRef,
  topology : Topology,
  clip : Option< ResourceId >,
}

/// One instance transform. POD.
#[ derive( Debug, Clone, Copy ) ]
struct Instance
{
  transform : Transform,
}

#[ derive( Debug, Clone, Copy ) ]
struct EndInstancedMesh;

// -- Effects (POD, applied to next element or group) --

/// Pushes an effect onto the current group.
/// SVG: `<filter>` with corresponding `fe*` element.
/// GPU: post-process pass (render to texture, apply shader, composite).
#[ derive( Debug, Clone, Copy ) ]
enum Effect
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

// -- Group (for shared transform/clip/effect) --

/// Begins a group. SVG: `<g>`. GPU: push transform/state stack.
#[ derive( Debug, Clone, Copy ) ]
struct BeginGroup
{
  transform : Transform,
  clip : Option< ResourceId >,
  effect : Option< Effect >,
}

/// Ends group. SVG: `</g>`. GPU: pop state stack.
#[ derive( Debug, Clone, Copy ) ]
struct EndGroup;

// ============================================================================
// Command enum — the unified command queue element
// ============================================================================

/// A single render command. All variants are POD (Copy, no allocations).
/// The scene stores an ordered `Vec< RenderCommand >` that backends process sequentially.
#[ derive( Debug, Clone, Copy ) ]
enum RenderCommand
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

  // Grouping
  BeginGroup( BeginGroup ),
  EndGroup( EndGroup ),
}

// ============================================================================
// Style types — all POD
// ============================================================================

#[ derive( Debug, Clone, Copy ) ]
enum LineCap { Butt, Round, Square }

#[ derive( Debug, Clone, Copy ) ]
enum LineJoin { Miter, Round, Bevel }

/// Dash pattern. Fixed-size to stay POD (no Vec).
/// Up to 4 dash-gap pairs covers most cases.
/// SVG: `stroke-dasharray`. GPU: fragment shader or geometry expansion.
#[ derive( Debug, Clone, Copy ) ]
struct DashStyle
{
  /// dash-gap pairs, zero-terminated. e.g. [5.0, 3.0, 0.0, ...] = "5 3"
  pattern : [ f32; 8 ],
  offset : f32,
}

#[ derive( Debug, Clone, Copy ) ]
enum TextAnchor
{
  TopLeft, TopCenter, TopRight,
  CenterLeft, Center, CenterRight,
  BottomLeft, BottomCenter, BottomRight,
}

#[ derive( Debug, Clone, Copy ) ]
enum Topology
{
  TriangleList,
  TriangleStrip,
  LineList,
  LineStrip,
}

/// Texture sampling filter.
/// SVG: `image-rendering` CSS property on `<image>`.
/// GPU: `mag_filter` / `min_filter` on the texture sampler.
#[ derive( Debug, Clone, Copy ) ]
enum SamplerFilter
{
  /// Nearest-neighbor: sharp pixels, no interpolation. Ideal for pixel art.
  /// SVG: `image-rendering: pixelated`. GPU: `FilterMode::Nearest`.
  Nearest,
  /// Bilinear interpolation: smooth scaling.
  /// SVG: `image-rendering: auto`. GPU: `FilterMode::Linear`.
  Linear,
}

/// Blend mode for compositing.
/// SVG: `mix-blend-mode` CSS property.
/// GPU: blend state on the pipeline (src/dst factors).
#[ derive( Debug, Clone, Copy ) ]
enum BlendMode
{
  /// Default: source over (alpha blending).
  Normal,
  /// SVG: `multiply`. GPU: src * dst.
  Multiply,
  /// SVG: `screen`. GPU: 1 - (1-src)*(1-dst).
  Screen,
  /// SVG: `overlay`.
  Overlay,
  /// Additive blending. GPU: src + dst.
  Add,
}

/// Fill reference — POD. Points to a fill definition in Assets, or solid color.
/// SVG: solid → `fill="rgb(...)"`, gradient → `fill="url(#grad_N)"`, pattern → `fill="url(#pat_N)"`.
/// GPU: solid → uniform color, gradient → gradient shader, pattern → texture with repeat sampler.
#[ derive( Debug, Clone, Copy ) ]
enum FillRef
{
  None,
  Solid( [ f32; 4 ] ),
  /// References a gradient or pattern stored in Assets.
  Asset( ResourceId ),
}

// ============================================================================
// Assets — CAN allocate, loaded before rendering
// ============================================================================

pub struct Assets
{
  fonts : Vec< FontAsset >,
  images : Vec< ImageAsset >,
  sprites : Vec< SpriteAsset >,
  geometries : Vec< GeometryAsset >,
  gradients : Vec< GradientAsset >,
  patterns : Vec< PatternAsset >,
  clip_masks : Vec< ClipMaskAsset >,
  /// Named paths that can be referenced (e.g. for text-on-path).
  paths : Vec< PathAsset >,
}

struct FontAsset
{
  id : ResourceId,
  source : PathBuf,
}

struct ImageAsset
{
  id : ResourceId,
  source : Source,
  /// Sampling filter for this image.
  /// SVG: `image-rendering: pixelated` (Nearest) vs `auto` (Linear).
  /// GPU: sampler mag/min filter.
  filter : SamplerFilter,
}

/// A rectangular region within a loaded image (sprite sheet support).
/// SVG: `<symbol viewBox="x y w h"><use href="#sheet" .../></symbol>` — viewBox crops to the region.
/// GPU: UV coordinates mapped to the sub-rectangle within the texture atlas.
struct SpriteAsset
{
  id : ResourceId,
  /// The source image (sprite sheet) this sprite is cut from.
  sheet : ResourceId,
  /// Region within the sheet: x, y, width, height in pixels.
  region : [ f32; 4 ],
}

struct GeometryAsset
{
  id : ResourceId,
  source : Source,
  r#type : Type,
}

/// Gradient definition.
/// SVG: `<linearGradient>` / `<radialGradient>` in `<defs>`.
/// GPU: uploaded as a 1D texture or evaluated analytically in shader.
struct GradientAsset
{
  id : ResourceId,
  kind : GradientKind,
  stops : Vec< GradientStop >,
}

#[ derive( Debug, Clone, Copy ) ]
struct GradientStop
{
  offset : f32,        // 0.0..1.0
  color : [ f32; 4 ],
}

#[ derive( Debug, Clone, Copy ) ]
enum GradientKind
{
  Linear
  {
    start : [ f32; 2 ],
    end : [ f32; 2 ],
  },
  Radial
  {
    center : [ f32; 2 ],
    radius : f32,
    focal : [ f32; 2 ],  // focal point, can equal center
  },
}

/// A repeating tile pattern.
/// SVG: `<pattern>` in `<defs>` containing an `<image>` or shape.
/// GPU: texture with `AddressMode::Repeat` sampler. The image is the tile,
///   the sampler wraps UVs so it repeats automatically across any surface.
struct PatternAsset
{
  id : ResourceId,
  /// The image or geometry to tile.
  content : ResourceId,
  width : f32,
  height : f32,
}

/// A clip mask — a shape that limits rendering to its interior.
/// SVG: `<clipPath>` in `<defs>`, elements use `clip-path="url(#...)"`.
/// GPU: draw clip shape into **stencil buffer** (write 1 where mask covers),
///   then draw actual content with stencil test (pass only where stencil == 1).
///   After done, clear stencil. Nested clips increment stencil value.
struct ClipMaskAsset
{
  id : ResourceId,
  /// Path segments defining the clip shape. Allocated in asset, not in commands.
  segments : Vec< PathSegmentOwned >,
}

/// Stored path (e.g. for text-on-path references).
struct PathAsset
{
  id : ResourceId,
  segments : Vec< PathSegmentOwned >,
}

/// Owning path segment for use in Assets (can allocate).
#[ derive( Debug, Clone, Copy ) ]
enum PathSegmentOwned
{
  MoveTo( f32, f32 ),
  LineTo( f32, f32 ),
  QuadTo { cx : f32, cy : f32, x : f32, y : f32 },
  CubicTo { c1x : f32, c1y : f32, c2x : f32, c2y : f32, x : f32, y : f32 },
  ArcTo { rx : f32, ry : f32, rotation : f32, large_arc : bool, sweep : bool, x : f32, y : f32 },
  Close,
}

// ============================================================================
// Port — the backend trait
// ============================================================================

/// Errors that can occur during rendering.
#[ derive( Debug ) ]
enum RenderError
{
  /// A command references a ResourceId not present in Assets.
  MissingAsset( ResourceId ),
  /// Backend does not support this command (e.g. terminal can't do gradients).
  Unsupported( &'static str ),
  /// Backend-specific error.
  BackendError( String ),
}

/// The core trait that all backends implement.
///
/// Creation is backend-specific (each has its own `new()`).
/// Usage is uniform through this trait.
///
/// ```ignore
/// // SVG
/// let mut svg = SvgBackend::new( 800, 600 );
/// svg.load_assets( &assets )?;
/// svg.submit( &commands )?;
/// let svg_string = svg.output()?;
///
/// // GPU
/// let mut gpu = WgpuBackend::new( &window, config );
/// gpu.load_assets( &assets )?;
/// gpu.submit( &commands )?; // presents to screen
/// ```
trait Backend
{
  /// Upload / prepare assets for this backend.
  /// Called once (or when assets change). Backend converts assets into
  /// its internal representation:
  /// - SVG: generates `<defs>` with `<symbol>`, `<linearGradient>`, `<pattern>`, `<clipPath>`
  /// - GPU: uploads textures, creates samplers, builds vertex buffers for geometries
  fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >;

  /// Process a command queue. This is the main render call.
  /// Backend iterates commands sequentially, maintaining internal state
  /// for streaming commands (BeginPath..EndPath, BeginText..EndText, etc.).
  fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >;

  /// Retrieve the rendered output.
  /// - SVG: `Output::String` containing the SVG document
  /// - GPU (offscreen): `Output::BitMap` with pixel data
  /// - GPU (realtime): `Output::ScreenPresent` (already presented in submit)
  /// - Terminal: `Output::String` with ANSI text
  fn output( &self ) -> Result< Output, RenderError >;

  /// Query backend capabilities so the caller can adapt.
  fn capabilities( &self ) -> Capabilities;
}

/// What a backend supports. Caller can check before submitting commands.
#[ derive( Debug, Clone, Copy ) ]
struct Capabilities
{
  /// Can render path commands (BeginPath..EndPath).
  pub paths : bool,
  /// Can render text commands (BeginText..EndText).
  pub text : bool,
  /// Can render Mesh commands.
  pub meshes : bool,
  /// Can render Sprite commands (requires image support).
  pub sprites : bool,
  /// Can render instanced meshes.
  pub instancing : bool,
  /// Supports gradient fills.
  pub gradients : bool,
  /// Supports pattern fills.
  pub patterns : bool,
  /// Supports clip masks.
  pub clip_masks : bool,
  /// Supports effects (blur, shadow, etc.).
  pub effects : bool,
  /// Supports blend modes beyond Normal.
  pub blend_modes : bool,
  /// Supports text along a path.
  pub text_on_path : bool,
  /// Maximum texture/image dimension (0 = unlimited, e.g. SVG).
  pub max_texture_size : u32,
}

// ============================================================================
// Adapter sketches — how backends implement the trait
// ============================================================================

// -- SVG Backend --

/// Static SVG file generation.
/// Creates a complete SVG 1.1 document from commands.
///
/// ```ignore
/// let mut svg = SvgBackend::new( 800, 600 );
/// svg.load_assets( &assets )?;
/// svg.submit( &commands )?;
/// let Output::String( svg_doc ) = svg.output()? else { unreachable!() };
/// std::fs::write( "out.svg", svg_doc )?;
/// ```
struct SvgBackend
{
  width : u32,
  height : u32,
  /// Accumulated SVG content.
  content : String,
  /// `<defs>` section: symbols, gradients, patterns, clip paths.
  defs : String,
  /// Body section: rendered elements.
  body : String,
  // -- streaming state --
  /// Active path being built between BeginPath..EndPath.
  path_active : bool,
  path_data : String,         // "M 0 0 L 10 10 C ..."
  path_style : Option< BeginPath >,
  /// Active text being built between BeginText..EndText.
  text_active : bool,
  text_buf : String,
  text_style : Option< BeginText >,
  /// Group nesting depth.
  group_depth : u32,
}

// impl Backend for SvgBackend
// {
//   fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >
//   {
//     // For each image: base64 encode, create <symbol id="img_N" viewBox="..."><image .../></symbol>
//     // For each sprite: <symbol id="spr_N" viewBox="region"><use href="#img_sheet"/></symbol>
//     // For each gradient: <linearGradient id="grad_N"> or <radialGradient>
//     // For each pattern: <pattern id="pat_N"><use href="#content"/></pattern>
//     // For each clip mask: <clipPath id="clip_N"><path d="segments"/></clipPath>
//     // For each path asset: <path id="path_N" d="segments"/> (for textPath references)
//     todo!()
//   }
//
//   fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
//   {
//     self.body.clear();
//     for cmd in commands
//     {
//       match cmd
//       {
//         RenderCommand::Clear( c ) =>
//         {
//           // <rect width="100%" height="100%" fill="color"/>
//         }
//         RenderCommand::BeginPath( bp ) =>
//         {
//           self.path_active = true;
//           self.path_data.clear();
//           self.path_style = Some( *bp );
//         }
//         RenderCommand::MoveTo( m ) => { write!( self.path_data, "M {} {} ", m.0, m.1 ); }
//         RenderCommand::LineTo( l ) => { write!( self.path_data, "L {} {} ", l.0, l.1 ); }
//         RenderCommand::QuadTo( q ) => { write!( self.path_data, "Q {} {} {} {} ", q.cx, q.cy, q.x, q.y ); }
//         RenderCommand::CubicTo( c ) => { write!( self.path_data, "C {} {} {} {} {} {} ", c.c1x, c.c1y, c.c2x, c.c2y, c.x, c.y ); }
//         RenderCommand::ArcTo( a ) => { write!( self.path_data, "A {} {} {} {} {} {} {} ", a.rx, a.ry, a.rotation, a.large_arc as u8, a.sweep as u8, a.x, a.y ); }
//         RenderCommand::ClosePath( _ ) => { self.path_data.push_str( "Z " ); }
//         RenderCommand::EndPath( _ ) =>
//         {
//           // Emit: <path d="..." fill="..." stroke="..." transform="..." clip-path="..." mix-blend-mode="..."/>
//           self.path_active = false;
//         }
//         RenderCommand::BeginText( bt ) =>
//         {
//           self.text_active = true;
//           self.text_buf.clear();
//           self.text_style = Some( *bt );
//         }
//         RenderCommand::Char( ch ) => { self.text_buf.push( ch.0 ); }
//         RenderCommand::EndText( _ ) =>
//         {
//           // If along_path: <text><textPath href="#path_N">text</textPath></text>
//           // Else: <text x="..." y="..." font-family="..." font-size="...">text</text>
//           self.text_active = false;
//         }
//         RenderCommand::Mesh( m ) =>
//         {
//           // Lookup geometry from loaded assets, emit <polygon> or <path>
//         }
//         RenderCommand::Sprite( s ) =>
//         {
//           // <use href="#spr_N" transform="..." style="mix-blend-mode: ..."/>
//           // Tint via <feColorMatrix> filter if tint != white
//         }
//         RenderCommand::BeginInstancedMesh( bim ) =>
//         {
//           // Store shared style, prepare for Instance commands
//         }
//         RenderCommand::Instance( inst ) =>
//         {
//           // <use href="#geom_N" transform="..."/>
//         }
//         RenderCommand::EndInstancedMesh( _ ) => {}
//         RenderCommand::BeginGroup( bg ) =>
//         {
//           // <g transform="..." clip-path="url(#clip_N)" filter="url(#effect_N)" opacity="...">
//           self.group_depth += 1;
//         }
//         RenderCommand::EndGroup( _ ) =>
//         {
//           // </g>
//           self.group_depth -= 1;
//         }
//       }
//     }
//     Ok( () )
//   }
//
//   fn output( &self ) -> Result< Output, RenderError >
//   {
//     // Assemble: <?xml?><svg ...><defs>...</defs>{body}</svg>
//     Ok( Output::String )
//   }
//
//   fn capabilities( &self ) -> Capabilities
//   {
//     Capabilities
//     {
//       paths : true,
//       text : true,
//       meshes : true,
//       sprites : true,
//       instancing : true,       // via <defs> + <use>
//       gradients : true,
//       patterns : true,
//       clip_masks : true,
//       effects : true,          // via <filter>
//       blend_modes : true,      // via mix-blend-mode
//       text_on_path : true,     // via <textPath>
//       max_texture_size : 0,    // unlimited
//     }
//   }
// }

// -- Terminal Backend --

/// ASCII art terminal output.
///
/// ```ignore
/// let mut term = TerminalBackend::new( 120, 40 );
/// term.load_assets( &assets )?;
/// term.submit( &commands )?;
/// let Output::String( text ) = term.output()? else { unreachable!() };
/// print!( "{}", text );
/// ```
struct TerminalBackend
{
  width : usize,
  height : usize,
  buffer : Vec< char >,           // width * height flat grid
  color_buffer : Vec< [u8; 3] >,  // ANSI RGB per cell
  unicode : bool,
  color : bool,
}

// impl Backend for TerminalBackend
// {
//   fn load_assets( &mut self, _assets : &Assets ) -> Result< (), RenderError >
//   {
//     // Terminal ignores most assets. Could pre-rasterize sprites to braille patterns.
//     Ok( () )
//   }
//
//   fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
//   {
//     // Same streaming state machine as SVG.
//     // Path segments → Bresenham line drawing into char buffer.
//     // Text → direct char placement.
//     // Sprites/Meshes/Effects → Unsupported or best-effort.
//     todo!()
//   }
//
//   fn output( &self ) -> Result< Output, RenderError >
//   {
//     // Join buffer rows into String with ANSI color codes.
//     Ok( Output::String )
//   }
//
//   fn capabilities( &self ) -> Capabilities
//   {
//     Capabilities
//     {
//       paths : true,          // Bresenham approximation
//       text : true,           // direct char placement
//       meshes : false,
//       sprites : false,
//       instancing : false,
//       gradients : false,
//       patterns : false,
//       clip_masks : false,
//       effects : false,
//       blend_modes : false,
//       text_on_path : false,
//       max_texture_size : 0,
//     }
//   }
// }

// -- GPU Backend (wgpu) sketch --

// struct WgpuBackend
// {
//   device : wgpu::Device,
//   queue : wgpu::Queue,
//   surface : wgpu::Surface,
//   // Uploaded textures, keyed by ResourceId
//   textures : HashMap< ResourceId, wgpu::Texture >,
//   samplers : HashMap< ResourceId, wgpu::Sampler >,  // Nearest vs Linear per image
//   // Vertex buffers for geometries
//   geometry_buffers : HashMap< ResourceId, wgpu::Buffer >,
//   // Stencil buffer for clip masks
//   stencil_view : wgpu::TextureView,
//   // State stack for BeginGroup/EndGroup
//   transform_stack : Vec< [f32; 6] >,  // affine matrices
//   stencil_depth : u32,                 // for nested clips
// }
//
// Key implementation notes for GPU:
// - Path: tessellate (lyon crate) into triangles, draw as TriangleList
// - Text: rasterize glyphs (fontdue/ab_glyph), upload as texture atlas, draw textured quads
// - Sprite: textured quad, UV = sprite.region / sheet_size
// - Instancing: wgpu instanced draw call, upload Instance transforms as instance buffer
// - Gradient: 1D texture lookup in fragment shader, or analytical evaluation
// - Pattern: texture with AddressMode::Repeat sampler
// - ClipMask: render mask geometry to stencil buffer, enable stencil test
// - Effect: render to offscreen texture, apply post-process shader, composite back
// - BlendMode: set wgpu::BlendState on the pipeline

// ============================================================================
// Supporting types
// ============================================================================

enum Source
{
  Path( PathBuf ),
  Bytes( Vec< u8 > ),
}

struct BitMap
{
  bytes : Vec< u8 >,
  format : ImageFormat,
  r#type : Type,
  width : u32,
  height : u32,
}

enum ImageFormat
{
  R,
  RG,
  RGB,
  RGBA,
}

enum GeometryFormat
{
  Vec1,
  Vec2,
  Vec3,
  Vec4,
}

enum Type
{
  U8,
  U16,
  U32,
  F32,
}

enum Output
{
  BitMap( BitMap ),
  String,
  ScreenPresent,
}
