//! `TerminalBackend` contract tests.
//!
//! Proves the coarse cell-grid rasterizer documented in
//! `src/adapters/terminal.rs`'s module docs: world-space commands land on
//! the expected grid cell (accounting for the `CELL_PX_WIDTH`/
//! `CELL_PX_HEIGHT` downsample and the Y-flip), asset validation matches
//! the `MissingAsset` contract shared with `SvgBackend`, and `output()`
//! encodes the grid as the exact ANSI truecolor bytes documented in the
//! module docs -- not merely that every call returns `Ok`.
//!
//! Grid-position assertions go through the `#[doc(hidden)]` test-only
//! accessors (`cols`/`rows`/`cell_bg`/`cell_fg`/`cell_glyph`), mirroring how
//! `svg_backend_test.rs` reaches into `SvgContentManager`'s own
//! `#[doc(hidden)]` surface.

#![ cfg( feature = "adapter-terminal" ) ]

use tilemap_renderer::assets::*;
use tilemap_renderer::backend::*;
use tilemap_renderer::commands::*;
use tilemap_renderer::types::*;
use tilemap_renderer::adapters::terminal::TerminalBackend;

mod helpers;
use helpers::empty_assets;

/// Default `RenderConfig` (800x600) downsamples to a 50x19 cell grid:
/// `cols = 800.div_ceil(16) = 50`, `rows = 600.div_ceil(32) = 19` (ceiling,
/// not floor -- floor would make row 18 unreachable and drop the coordinate
/// origin `[0, 0]` off the bottom edge of the grid).
fn term() -> TerminalBackend
{
  TerminalBackend::new( RenderConfig::default() )
}

/// `Assets` with sprite id 0 and geometry id 0 both registered. Terminal's
/// `assets_load` only records resource ids (it never reads pixels/vertices
/// -- see the module docs), so a minimal fixture is enough to satisfy
/// `cmd_sprite`/`cmd_mesh`'s `MissingAsset` check.
fn loaded_assets() -> Assets
{
  let mut assets = empty_assets();
  assets.sprites.push( SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 1.0, 1.0 ] } );
  assets.geometries.push( GeometryAsset { id : ResourceId::new( 0 ), positions : Source::Bytes( vec![] ), uvs : None, indices : None, data_type : DataType::F32 } );
  assets
}

// ============================================================================
// Grid dimensions
// ============================================================================

/// T01 -- a fresh backend's grid dimensions match ceiling-division of the
/// config's pixel dimensions by the cell size, not floor division.
#[ test ]
fn grid_dimensions_use_ceiling_division()
{
  let backend = term();
  assert_eq!( backend.cols(), 50 );
  assert_eq!( backend.rows(), 19 );
}

// ============================================================================
// Clear
// ============================================================================

/// T02 -- `Clear` fills every cell's background with its color and resets
/// the glyph to `' '`.
#[ test ]
fn clear_fills_every_cell()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let color = [ 0.2, 0.4, 0.6, 1.0 ];
  backend.submit( &[ RenderCommand::Clear( Clear { color } ) ] ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( color ) );
  assert_eq!( backend.cell_glyph( 0, 0 ), Some( ' ' ) );
  assert_eq!( backend.cell_bg( 49, 18 ), Some( color ) );
}

// ============================================================================
// Sprite
// ============================================================================

/// T03 -- a `Sprite` referencing a loaded sprite asset paints its tint at
/// the cell its transform's position resolves to. World `[0, 600]` is the
/// top-left corner of an 800x600 viewport, which is cell `(0, 0)` after the
/// Y-flip (`row_f = (600 - 600) / 32 = 0`).
#[ test ]
fn sprite_paints_tint_at_resolved_cell()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let tint = [ 0.0, 1.0, 0.0, 1.0 ];
  let cmd = RenderCommand::Sprite( Sprite
  {
    transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() },
    sprite : ResourceId::new( 0 ),
    tint,
    blend : BlendMode::default(),
    clip : None,
  });

  backend.submit( &[ cmd ] ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( tint ) );
  assert_eq!( backend.cell_glyph( 0, 0 ), Some( ' ' ) );
}

/// T04 -- a `Sprite` referencing an unloaded sprite id returns
/// `RenderError::MissingAsset`, mirroring `SvgBackend`'s BUG-209 contract.
#[ test ]
fn sprite_missing_asset_returns_error()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let cmd = RenderCommand::Sprite( Sprite
  {
    transform : Transform::default(),
    sprite : ResourceId::new( 99 ),
    tint : [ 1.0, 1.0, 1.0, 1.0 ],
    blend : BlendMode::default(),
    clip : None,
  });

  match backend.submit( &[ cmd ] )
  {
    Err( RenderError::MissingAsset( id ) ) => assert_eq!( id, 99 ),
    other => panic!( "expected Err(MissingAsset(99)), got {other:?}" ),
  }
}

/// T05 -- `ScreenSpaceSprite` shares `Sprite`'s payload and dispatch path
/// (terminal has no camera transform of its own to bypass), so it paints
/// identically to a plain `Sprite` at the same position.
#[ test ]
fn screen_space_sprite_paints_like_sprite()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let tint = [ 1.0, 0.5, 0.0, 1.0 ];
  let cmd = RenderCommand::ScreenSpaceSprite( Sprite
  {
    transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() },
    sprite : ResourceId::new( 0 ),
    tint,
    blend : BlendMode::default(),
    clip : None,
  });

  backend.submit( &[ cmd ] ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( tint ) );
}

// ============================================================================
// Mesh
// ============================================================================

/// T06 -- a `Mesh` with `FillRef::Solid` paints that color at the cell its
/// transform's position resolves to. World `[16, 600]` is cell `(1, 0)`
/// (`col_f = 16 / 16 = 1`).
#[ test ]
fn mesh_solid_fill_paints_expected_cell()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let color = [ 0.2, 0.4, 0.6, 1.0 ];
  let cmd = RenderCommand::Mesh( Mesh
  {
    transform : Transform { position : [ 16.0, 600.0 ], ..Default::default() },
    geometry : ResourceId::new( 0 ),
    fill : FillRef::Solid( color ),
    texture : None,
    topology : Topology::TriangleList,
    blend : BlendMode::default(),
    clip : None,
  });

  backend.submit( &[ cmd ] ).unwrap();

  assert_eq!( backend.cell_bg( 1, 0 ), Some( color ) );
}

/// T07 -- a `Mesh` referencing an unloaded geometry id returns
/// `RenderError::MissingAsset`.
#[ test ]
fn mesh_missing_asset_returns_error()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let cmd = RenderCommand::Mesh( Mesh
  {
    transform : Transform::default(),
    geometry : ResourceId::new( 42 ),
    fill : FillRef::Solid( [ 1.0, 1.0, 1.0, 1.0 ] ),
    texture : None,
    topology : Topology::TriangleList,
    blend : BlendMode::default(),
    clip : None,
  });

  match backend.submit( &[ cmd ] )
  {
    Err( RenderError::MissingAsset( id ) ) => assert_eq!( id, 42 ),
    other => panic!( "expected Err(MissingAsset(42)), got {other:?}" ),
  }
}

/// T08 -- a `Mesh` with a non-`Solid` fill (`Gradient`/`Pattern`) is
/// accepted (`Ok`, matching `capabilities().gradients == false` meaning
/// "not honored", not "rejected") but paints nothing, since the coarse
/// rasterizer only resolves a flat `Solid` color to a cell.
#[ test ]
fn mesh_gradient_fill_paints_nothing()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let cmd = RenderCommand::Mesh( Mesh
  {
    transform : Transform { position : [ 16.0, 600.0 ], ..Default::default() },
    geometry : ResourceId::new( 0 ),
    fill : FillRef::Gradient( ResourceId::new( 0 ) ),
    texture : None,
    topology : Topology::TriangleList,
    blend : BlendMode::default(),
    clip : None,
  });

  let background = RenderConfig::default().background;
  assert!( backend.submit( &[ cmd ] ).is_ok() );
  assert_eq!( backend.cell_bg( 1, 0 ), Some( background ) );
}

// ============================================================================
// Text
// ============================================================================

/// T09 -- `BeginText`/`Char`/`EndText` with `TextAnchor::TopLeft` places
/// each character starting at the resolved base cell, one column per
/// character, on the base row.
#[ test ]
fn text_left_anchor_places_glyphs_left_to_right()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let color = [ 1.0, 1.0, 1.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginText( BeginText
    {
      font : ResourceId::new( 0 ),
      size : 12.0,
      color,
      anchor : TextAnchor::TopLeft,
      position : [ 0.0, 600.0 ],
      along_path : None,
      clip : None,
    }),
    RenderCommand::Char( Char( 'H' ) ),
    RenderCommand::Char( Char( 'i' ) ),
    RenderCommand::EndText( EndText ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_glyph( 0, 0 ), Some( 'H' ) );
  assert_eq!( backend.cell_glyph( 1, 0 ), Some( 'i' ) );
  assert_eq!( backend.cell_fg( 0, 0 ), Some( color ) );
}

/// T10 -- `TextAnchor::Center` shifts the starting column left by
/// `len / 2` (integer division): a 2-character string based at column 10
/// starts at column 9. Uses `position.y = 300.0` (row 8 once `Center`'s own
/// vertical nudge applies -- see T35), not `config.height` like T09/T34's
/// `Top` cases: `Center`'s vertical component pushes the resolved point up
/// by half a cell, which would fall outside the grid at the very top edge.
#[ test ]
fn text_center_anchor_shifts_start_column()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let commands =
  [
    RenderCommand::BeginText( BeginText
    {
      font : ResourceId::new( 0 ),
      size : 12.0,
      color : [ 1.0, 1.0, 1.0, 1.0 ],
      anchor : TextAnchor::Center,
      position : [ 160.0, 300.0 ],
      along_path : None,
      clip : None,
    }),
    RenderCommand::Char( Char( 'A' ) ),
    RenderCommand::Char( Char( 'B' ) ),
    RenderCommand::EndText( EndText ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_glyph( 9, 8 ), Some( 'A' ) );
  assert_eq!( backend.cell_glyph( 10, 8 ), Some( 'B' ) );
}

/// T34 -- vertical anchor now affects row placement: `TextAnchor::TopLeft`
/// and `TextAnchor::BottomLeft` at the same `position` resolve to adjacent
/// rows -- Bottom is always exactly one row above Top, per the
/// `CELL_PX_HEIGHT` row-nudge `cmd_end_text` derives from SVG's own
/// `dominant-baseline` "hanging"/"baseline" split -- proving vertical
/// anchor is no longer collapsed to a single row regardless of
/// `style.anchor`.
#[ test ]
fn text_vertical_top_and_bottom_anchor_differ_by_one_row()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let commands =
  [
    RenderCommand::BeginText( BeginText
    {
      font : ResourceId::new( 0 ),
      size : 12.0,
      color : [ 1.0, 1.0, 1.0, 1.0 ],
      anchor : TextAnchor::TopLeft,
      position : [ 0.0, 304.0 ],
      along_path : None,
      clip : None,
    }),
    RenderCommand::Char( Char( 'T' ) ),
    RenderCommand::EndText( EndText ),
    RenderCommand::BeginText( BeginText
    {
      font : ResourceId::new( 0 ),
      size : 12.0,
      color : [ 1.0, 1.0, 1.0, 1.0 ],
      anchor : TextAnchor::BottomLeft,
      position : [ 0.0, 304.0 ],
      along_path : None,
      clip : None,
    }),
    RenderCommand::Char( Char( 'B' ) ),
    RenderCommand::EndText( EndText ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_glyph( 0, 9 ), Some( 'T' ) );
  assert_eq!( backend.cell_glyph( 0, 8 ), Some( 'B' ) );
}

/// T35 -- `TextAnchor::CenterLeft`'s vertical component resolves to the
/// row whose exact world-space vertical center equals `position`'s Y: at
/// `position = [0.0, 296.0]`, row 9 spans world Y `(280.0, 312.0]` (cell
/// height 32), whose midpoint is exactly 296.0.
#[ test ]
fn text_vertical_center_anchor_resolves_row_midpoint()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let commands =
  [
    RenderCommand::BeginText( BeginText
    {
      font : ResourceId::new( 0 ),
      size : 12.0,
      color : [ 1.0, 1.0, 1.0, 1.0 ],
      anchor : TextAnchor::CenterLeft,
      position : [ 0.0, 296.0 ],
      along_path : None,
      clip : None,
    }),
    RenderCommand::Char( Char( 'C' ) ),
    RenderCommand::EndText( EndText ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_glyph( 0, 9 ), Some( 'C' ) );
}

// ============================================================================
// Paths
// ============================================================================

/// T11 -- `BeginPath`/`MoveTo`/`LineTo`/`EndPath` rasterizes a straight
/// line across every cell it crosses. A horizontal line from world
/// `[0, 0]` to `[32, 0]` (bottom-left corner, row `18` after the Y-flip:
/// `row_f = (600 - 0) / 32 = 18.75 -> floor 18`) crosses cells `(0, 18)`,
/// `(1, 18)`, `(2, 18)`.
#[ test ]
fn path_horizontal_line_paints_crossed_cells()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let stroke_color = [ 1.0, 0.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::None,
      stroke_color,
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 0.0, 0.0 ) ),
    RenderCommand::LineTo( LineTo( 32.0, 0.0 ) ),
    RenderCommand::EndPath( EndPath ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 0, 18 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 1, 18 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 2, 18 ), Some( stroke_color ) );
}

/// T12 -- a `BeginPath` with no `MoveTo`/`LineTo` at all (immediately
/// `EndPath`) never panics and paints nothing -- `cells.windows(2)` on a
/// zero/one-point list simply yields no pairs.
#[ test ]
fn path_with_no_points_paints_nothing_and_does_not_panic()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let commands =
  [
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::None,
      stroke_color : [ 1.0, 0.0, 0.0, 1.0 ],
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::EndPath( EndPath ),
  ];

  let background = RenderConfig::default().background;
  assert!( backend.submit( &commands ).is_ok() );
  assert_eq!( backend.cell_bg( 0, 0 ), Some( background ) );
}

// ============================================================================
// Groups
// ============================================================================

/// T13 -- `BeginGroup`'s `transform` shifts every command inside it: a
/// `Sprite` nominally at world `[0, 600]` (cell `(0, 0)`) lands at cell
/// `(1, 0)` under a `translate(16, 0)` group, and the untranslated cell
/// stays at the frame's background.
#[ test ]
fn group_transform_shifts_sprite()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let tint = [ 1.0, 0.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginGroup( BeginGroup
    {
      transform : Transform { position : [ 16.0, 0.0 ], ..Default::default() },
      clip : None,
      effect : None,
    }),
    RenderCommand::Sprite( Sprite
    {
      transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() },
      sprite : ResourceId::new( 0 ),
      tint,
      blend : BlendMode::default(),
      clip : None,
    }),
    RenderCommand::EndGroup( EndGroup ),
  ];

  let background = RenderConfig::default().background;
  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 1, 0 ), Some( tint ) );
  assert_eq!( backend.cell_bg( 0, 0 ), Some( background ) );
}

/// T14 -- an unmatched `EndGroup` (no preceding `BeginGroup`) never panics
/// -- `Vec::pop()` on an empty stack is a safe no-op.
#[ test ]
fn unmatched_end_group_does_not_panic()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  assert!( backend.submit( &[ RenderCommand::EndGroup( EndGroup ) ] ).is_ok() );
}

// ============================================================================
// Batches
// ============================================================================

/// T15 -- a bound-and-drawn sprite batch composes the instance's local
/// position through the batch's own parent `transform` before resolving a
/// cell, the same as `group_transform_shifts_sprite` above.
#[ test ]
fn batch_sprite_draw_composes_parent_transform()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let batch = ResourceId::new( 0 );
  let tint = [ 0.0, 0.0, 1.0, 1.0 ];
  let commands =
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch
    {
      batch,
      params : SpriteBatchParams
      {
        transform : Transform { position : [ 16.0, 0.0 ], ..Default::default() },
        sheet : ResourceId::new( 0 ),
        blend : BlendMode::default(),
        clip : None,
        alpha_clip : 0.0,
        occlude_overlap : false,
      },
    }),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance
    {
      transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() },
      sprite : ResourceId::new( 0 ),
      tint,
    }),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 1, 0 ), Some( tint ) );
}

/// T16 -- a bound-and-drawn mesh batch with `FillRef::Solid` paints that
/// color at the composed cell; the per-instance `tint` is not multiplied
/// in (the coarse rasterizer uses the batch's own fill directly -- see the
/// module docs).
#[ test ]
fn batch_mesh_draw_solid_fill_paints_expected_cell()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let batch = ResourceId::new( 0 );
  let fill_color = [ 0.5, 0.5, 0.5, 1.0 ];
  let commands =
  [
    RenderCommand::CreateMeshBatch( CreateMeshBatch
    {
      batch,
      params : MeshBatchParams
      {
        transform : Transform { position : [ 16.0, 0.0 ], ..Default::default() },
        geometry : ResourceId::new( 0 ),
        fill : FillRef::Solid( fill_color ),
        texture : None,
        topology : Topology::TriangleList,
        blend : BlendMode::default(),
        clip : None,
      },
    }),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddMeshInstance( AddMeshInstance
    {
      transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() },
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
    }),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 1, 0 ), Some( fill_color ) );
}

/// T17 -- `DrawBatch` referencing a nonexistent batch id is a graceful
/// no-op (`Ok`, nothing painted), never a panic or error.
#[ test ]
fn draw_batch_missing_id_is_noop()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let background = RenderConfig::default().background;

  assert!( backend.submit( &[ RenderCommand::DrawBatch( DrawBatch { batch : ResourceId::new( 7 ) } ) ] ).is_ok() );
  assert_eq!( backend.cell_bg( 0, 0 ), Some( background ) );
}

/// T18 -- `SetSpriteInstance` with an out-of-bounds `index` returns
/// `RenderError::BackendError`, mirroring `SvgBackend`'s BUG-211 contract
/// (out-of-bounds is a hard error, not a silent no-op).
#[ test ]
fn set_sprite_instance_out_of_bounds_returns_error()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let batch = ResourceId::new( 0 );
  let commands =
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch
    {
      batch,
      params : SpriteBatchParams { transform : Transform::default(), sheet : ResourceId::new( 0 ), blend : BlendMode::default(), clip : None, alpha_clip : 0.0, occlude_overlap : false },
    }),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance { transform : Transform::default(), sprite : ResourceId::new( 0 ), tint : [ 1.0, 1.0, 1.0, 1.0 ] } ),
    RenderCommand::SetSpriteInstance( SetSpriteInstance { index : 5, transform : Transform::default(), sprite : ResourceId::new( 0 ), tint : [ 1.0, 1.0, 1.0, 1.0 ] } ),
  ];

  match backend.submit( &commands )
  {
    Err( RenderError::BackendError( _ ) ) => {}
    other => panic!( "expected Err(BackendError(_)), got {other:?}" ),
  }
}

/// T19 -- `RemoveInstance` swap-removes the target: with two instances,
/// removing index 0 moves the last instance (unchanged data) into its
/// place. The survivor still draws at its own world position afterward;
/// the removed instance's position is never painted.
#[ test ]
fn remove_instance_drops_target_and_keeps_remaining()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let batch = ResourceId::new( 0 );
  let red = [ 1.0, 0.0, 0.0, 1.0 ];
  let green = [ 0.0, 1.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch
    {
      batch,
      params : SpriteBatchParams { transform : Transform::default(), sheet : ResourceId::new( 0 ), blend : BlendMode::default(), clip : None, alpha_clip : 0.0, occlude_overlap : false },
    }),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance { transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() }, sprite : ResourceId::new( 0 ), tint : red } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance { transform : Transform { position : [ 16.0, 600.0 ], ..Default::default() }, sprite : ResourceId::new( 0 ), tint : green } ),
    RenderCommand::RemoveInstance( RemoveInstance { index : 0 } ),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch } ),
  ];

  let background = RenderConfig::default().background;
  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 1, 0 ), Some( green ) );
  assert_eq!( backend.cell_bg( 0, 0 ), Some( background ) );
}

/// T20 -- `RemoveInstance` on an unbound batch (no `BindBatch` beforehand)
/// is a graceful no-op, never a panic.
#[ test ]
fn remove_instance_without_bound_batch_does_not_panic()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  assert!( backend.submit( &[ RenderCommand::RemoveInstance( RemoveInstance { index : 0 } ) ] ).is_ok() );
}

// ============================================================================
// assets_load contract
// ============================================================================

/// T21 -- `assets_load` destroys previously created batches (per
/// `Backend::assets_load`'s documented contract): a batch created and
/// populated before a second `assets_load` call no longer exists
/// afterward, so drawing it is a no-op.
#[ test ]
fn assets_load_destroys_existing_batches()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let batch = ResourceId::new( 0 );
  let commands =
  [
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch
    {
      batch,
      params : SpriteBatchParams { transform : Transform::default(), sheet : ResourceId::new( 0 ), blend : BlendMode::default(), clip : None, alpha_clip : 0.0, occlude_overlap : false },
    }),
    RenderCommand::BindBatch( BindBatch { batch } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance { transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() }, sprite : ResourceId::new( 0 ), tint : [ 1.0, 0.0, 0.0, 1.0 ] } ),
    RenderCommand::UnbindBatch( UnbindBatch ),
  ];
  backend.submit( &commands ).unwrap();

  backend.assets_load( &empty_assets() ).unwrap();

  let background = RenderConfig::default().background;
  backend.submit( &[ RenderCommand::DrawBatch( DrawBatch { batch } ) ] ).unwrap();
  assert_eq!( backend.cell_bg( 0, 0 ), Some( background ) );
}

// ============================================================================
// resize
// ============================================================================

/// T22 -- `resize` recomputes `cols`/`rows` for the new dimensions and
/// reallocates the grid, discarding any previously painted content.
#[ test ]
fn resize_recomputes_grid_and_clears_content()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  backend.submit( &[ RenderCommand::Clear( Clear { color : [ 1.0, 0.0, 0.0, 1.0 ] } ) ] ).unwrap();
  assert_eq!( backend.cell_bg( 0, 0 ), Some( [ 1.0, 0.0, 0.0, 1.0 ] ) );

  backend.resize( 16, 32 );

  assert_eq!( backend.cols(), 1 );
  assert_eq!( backend.rows(), 1 );
  assert_eq!( backend.cell_bg( 0, 0 ), Some( RenderConfig::default().background ) );
  assert_eq!( backend.cell_bg( 1, 0 ), None );
}

// ============================================================================
// capabilities
// ============================================================================

/// T23 -- `capabilities()` honestly reports coarse support: every command
/// family the backend processes is `true`, every family it cannot express
/// at cell resolution (gradients, patterns, clip masks, effects, text-on-path)
/// is `false`, and blending is limited to exactly `[BlendMode::Normal]`
/// (source-over) -- `blend_modes` stays `false` since the other 4 variants
/// silently fall back rather than rendering correctly.
#[ test ]
fn capabilities_reports_coarse_support()
{
  let backend = term();
  let caps = backend.capabilities();

  assert!( caps.paths );
  assert!( caps.text );
  assert!( caps.meshes );
  assert!( caps.sprites );
  assert!( caps.batches );
  assert!( !caps.gradients );
  assert!( !caps.patterns );
  assert!( !caps.clip_masks );
  assert!( !caps.effects );
  assert!( !caps.blend_modes );
  assert_eq!( caps.supported_blend_modes, &[ BlendMode::Normal ] );
  assert!( !caps.text_on_path );
  assert_eq!( caps.max_texture_size, 0 );
}

// ============================================================================
// output() -- raw ANSI bytes
// ============================================================================

/// T24 -- `output()` on a 1x1 grid (config exactly one cell wide/tall)
/// encodes a background-only cell as an SGR 48;2 truecolor space, followed
/// by a reset and newline -- the exact byte sequence documented in the
/// module docs, not just "contains an escape code".
#[ test ]
fn output_encodes_background_cell_as_exact_ansi_bytes()
{
  let mut backend = TerminalBackend::new( RenderConfig { width : 16, height : 32, ..Default::default() } );
  backend.assets_load( &empty_assets() ).unwrap();
  backend.submit( &[ RenderCommand::Clear( Clear { color : [ 1.0, 0.0, 0.0, 1.0 ] } ) ] ).unwrap();

  let Output::String( frame ) = backend.output().unwrap() else { panic!( "expected Output::String" ) };
  assert_eq!( frame, "\x1b[48;2;255;0;0m \x1b[0m\n" );
}

/// T25 -- `output()` on a 1x1 grid encodes a glyph cell as an SGR 38;2
/// truecolor foreground code carrying the glyph itself, not a space.
#[ test ]
fn output_encodes_glyph_cell_as_exact_ansi_bytes()
{
  let mut backend = TerminalBackend::new( RenderConfig { width : 16, height : 32, ..Default::default() } );
  backend.assets_load( &empty_assets() ).unwrap();
  let commands =
  [
    RenderCommand::BeginText( BeginText
    {
      font : ResourceId::new( 0 ),
      size : 12.0,
      color : [ 1.0, 1.0, 1.0, 1.0 ],
      anchor : TextAnchor::TopLeft,
      position : [ 0.0, 32.0 ],
      along_path : None,
      clip : None,
    }),
    RenderCommand::Char( Char( 'H' ) ),
    RenderCommand::EndText( EndText ),
  ];
  backend.submit( &commands ).unwrap();

  let Output::String( frame ) = backend.output().unwrap() else { panic!( "expected Output::String" ) };
  assert_eq!( frame, "\x1b[38;2;255;255;255mH\x1b[0m\n" );
}

// ============================================================================
// Curve flattening
// ============================================================================

/// T28 -- `QuadTo` flattens into `CURVE_SEGMENTS` points along the actual
/// quadratic Bezier rather than drawing straight to the endpoint. World
/// `[0, 600]` -> control `[128, 280]` -> `[256, 600]` has a straight chord
/// that stays at row 0 throughout (both endpoints share `y = 600`), so
/// painting cell `(8, 5)` -- hand-traced from the Bezier formula at
/// `t = 0.5`: `x = 0.25*0 + 0.5*128 + 0.25*256 = 128`, `y = 0.25*600 +
/// 0.5*280 + 0.25*600 = 440`, giving `col_f = 8`, `row_f = (600-440)/32 =
/// 5` -- proves the curve actually bowed away from the chord.
#[ test ]
fn path_quad_to_paints_bowed_bezier_cells()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let stroke_color = [ 1.0, 0.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::None,
      stroke_color,
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 0.0, 600.0 ) ),
    RenderCommand::QuadTo( QuadTo { cx : 128.0, cy : 280.0, x : 256.0, y : 600.0 } ),
    RenderCommand::EndPath( EndPath ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 8, 5 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 16, 0 ), Some( stroke_color ) );
}

/// T29 -- `CubicTo` flattens into `CURVE_SEGMENTS` points along the actual
/// cubic Bezier. Same chord as T28 (`[0, 600]` -> `[256, 600]`, both
/// controls pulled to `y = 280`), so painting cell `(8, 7)` -- hand-traced
/// at `t = 0.5`: `x = 0.25*0 + 0.75*128*... = 128`
/// (`3*mt^2*t = 3*mt*t^2 = 0.375` each contribute `0.375*128`), `y =
/// 0.25*600 + 0.75*280 = 360`, giving `col_f = 8`, `row_f = (600-360)/32 =
/// 7.5 -> 7` -- again off the chord's constant row 0.
#[ test ]
fn path_cubic_to_paints_bowed_bezier_cells()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let stroke_color = [ 1.0, 0.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::None,
      stroke_color,
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 0.0, 600.0 ) ),
    RenderCommand::CubicTo( CubicTo { c1x : 0.0, c1y : 280.0, c2x : 256.0, c2y : 280.0, x : 256.0, y : 600.0 } ),
    RenderCommand::EndPath( EndPath ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 8, 7 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 16, 0 ), Some( stroke_color ) );
}

/// T30 -- `ArcTo` flattens a true elliptical arc via SVG endpoint-to-center
/// parameterization, not a straight chord. A quarter circle centered at
/// world `[0, 600]`, radius 256, from `[256, 600]` to `[0, 344]`
/// (`large_arc: false, sweep: false`) hand-traces to center `(0, 600)`,
/// `theta1 = 0`, `delta = -90 deg`; at `t = 0.5` (`theta = -45 deg`):
/// `x = 256*cos(-45deg) = 181.02`, `y = 600 + 256*sin(-45deg) = 418.98`,
/// giving cell `(11, 5)`. A Bresenham-straight chord between the start/end
/// cells `(16, 0)` and `(0, 8)` never visits column 11 at row 5 (it visits
/// `(11, 3)` instead), so this cell distinguishes the true arc from a
/// straight-line fallback.
#[ test ]
fn path_arc_to_paints_true_ellipse_cells()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let stroke_color = [ 1.0, 0.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::None,
      stroke_color,
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 256.0, 600.0 ) ),
    RenderCommand::ArcTo( ArcTo { rx : 256.0, ry : 256.0, rotation : 0.0, large_arc : false, sweep : false, x : 0.0, y : 344.0 } ),
    RenderCommand::EndPath( EndPath ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 16, 0 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 11, 5 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 0, 8 ), Some( stroke_color ) );
}

/// T31 -- a degenerate `ArcTo` (`rx == 0`) falls back to a single point at
/// the endpoint (matching the SVG spec's degenerate-arc-as-line handling)
/// instead of panicking on the division by `ry`/`rx` in the center-
/// parameterization math. Same horizontal chord as T11, so the same three
/// cells end up painted.
#[ test ]
fn path_arc_to_degenerate_radius_falls_back_to_straight_line()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let stroke_color = [ 1.0, 0.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::None,
      stroke_color,
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 0.0, 0.0 ) ),
    RenderCommand::ArcTo( ArcTo { rx : 0.0, ry : 10.0, rotation : 0.0, large_arc : false, sweep : false, x : 32.0, y : 0.0 } ),
    RenderCommand::EndPath( EndPath ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 0, 18 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 1, 18 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 2, 18 ), Some( stroke_color ) );
}

/// T26 -- a non-axis-aligned line exercises the true Bresenham rasterizer
/// (the horizontal case in T11 can't distinguish it from a naive
/// interpolation). World `[0, 600]` -> `[48, 536]` crosses cells `(0, 0)`,
/// `(1, 1)`, `(2, 1)`, `(3, 2)` -- hand-traced from `line_cells`' symmetric
/// integer algorithm (dx=3, dy=-2, err starts at 1).
#[ test ]
fn path_diagonal_line_paints_bresenham_cells()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let stroke_color = [ 1.0, 0.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::None,
      stroke_color,
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 0.0, 600.0 ) ),
    RenderCommand::LineTo( LineTo( 48.0, 536.0 ) ),
    RenderCommand::EndPath( EndPath ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 1, 1 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 2, 1 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 3, 2 ), Some( stroke_color ) );
}

/// T27 -- `line_cells` is direction-symmetric (see its doc comment): tracing
/// the same two points in reverse paints the identical cell set as T26,
/// never a different diagonal "staircase" pattern.
#[ test ]
fn path_diagonal_line_is_symmetric_regardless_of_direction()
{
  let mut backend = term();
  backend.assets_load( &empty_assets() ).unwrap();
  let stroke_color = [ 1.0, 0.0, 0.0, 1.0 ];
  let commands =
  [
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::None,
      stroke_color,
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 48.0, 536.0 ) ),
    RenderCommand::LineTo( LineTo( 0.0, 600.0 ) ),
    RenderCommand::EndPath( EndPath ),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 1, 1 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 2, 1 ), Some( stroke_color ) );
  assert_eq!( backend.cell_bg( 3, 2 ), Some( stroke_color ) );
}

// ============================================================================
// Compositing
// ============================================================================

/// T32 -- a semi-transparent `Sprite` drawn over an opaque background blends
/// via source-over (Porter-Duff "over") on straight RGBA, per `composite_over`'s
/// doc comment: `out_a = sa + da*(1-sa)`, `out_rgb = (src*sa + dst*da*(1-sa))/out_a`.
/// Opaque blue (`da = 1.0`) under 50%-alpha red (`sa = 0.5`) yields exact halves
/// of each channel (`0.5/1.0 = 0.5`, no rounding), so the result is asserted
/// exactly rather than within an epsilon.
#[ test ]
fn sprite_alpha_blends_over_opaque_background_via_source_over()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let blue = [ 0.0, 0.0, 1.0, 1.0 ];
  let red_half = [ 1.0, 0.0, 0.0, 0.5 ];
  let commands =
  [
    RenderCommand::Clear( Clear { color : blue } ),
    RenderCommand::Sprite( Sprite
    {
      transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() },
      sprite : ResourceId::new( 0 ),
      tint : red_half,
      blend : BlendMode::default(),
      clip : None,
    }),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( [ 0.5, 0.0, 0.5, 1.0 ] ) );
}

/// T33 -- compositing a fully transparent `Sprite` (`sa = 0.0`) onto an
/// already fully transparent cell (`da = 0.0`) exercises `composite_over`'s
/// near-zero `out_a` guard documented on the function: rather than dividing
/// by `out_a ~ 0`, it short-circuits to transparent black.
#[ test ]
fn sprite_zero_alpha_over_transparent_background_avoids_division_by_zero()
{
  let mut backend = term();
  backend.assets_load( &loaded_assets() ).unwrap();
  let transparent = [ 0.0, 0.0, 0.0, 0.0 ];
  let commands =
  [
    RenderCommand::Clear( Clear { color : transparent } ),
    RenderCommand::Sprite( Sprite
    {
      transform : Transform { position : [ 0.0, 600.0 ], ..Default::default() },
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 0.5, 0.25, 0.0 ],
      blend : BlendMode::default(),
      clip : None,
    }),
  ];

  backend.submit( &commands ).unwrap();

  assert_eq!( backend.cell_bg( 0, 0 ), Some( [ 0.0, 0.0, 0.0, 0.0 ] ) );
}
