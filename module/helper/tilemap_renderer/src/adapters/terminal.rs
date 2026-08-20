//! Terminal backend adapter.
//!
//! Renders a `RenderCommand` stream onto a coarse character-cell grid and
//! encodes it as 24-bit ("truecolor") ANSI escape sequences — one line per
//! row, ready to `print!` directly to a terminal that understands SGR
//! `38;2;r;g;b` / `48;2;r;g;b` truecolor codes. Each cell carries a
//! background color (painted by `Clear`, path strokes, `Mesh`, `Sprite`)
//! and, optionally, a single glyph with its own foreground color (painted
//! by `BeginText`/`Char`/`EndText`) — terminal is the one backend where
//! *text* is the native medium rather than an approximation.
//!
//! ## Resolution
//!
//! A terminal cannot address individual world-space pixels, so world
//! coordinates are downsampled onto a fixed cell grid: one cell covers a
//! `CELL_PX_WIDTH x CELL_PX_HEIGHT` block of world units (16x32, chosen to
//! approximate a typical monospace glyph's ~1:2 width:height aspect ratio
//! so a world-space square doesn't look badly stretched once downsampled).
//! `resize` reallocates the grid for the new dimensions.
//!
//! ## Known simplifications (see `TerminalBackend::capabilities`)
//!
//! - Curves (`QuadTo`/`CubicTo`/`ArcTo`) flatten into a fixed 16 straight-line
//!   segments each (see `CURVE_SEGMENTS`) rather than adaptive
//!   arc-length/curvature-based subdivision — good enough at this adapter's
//!   coarse cell resolution.
//! - All points accumulated between one `BeginPath`/`EndPath` pair are
//!   connected in emission order, including across `MoveTo`-separated
//!   subpaths — there is no per-subpath break in the coarse rasterizer.
//! - `Mesh`/`Sprite` paint a single cell at their transform's position
//!   rather than resolving actual geometry vertices or image pixels —
//!   `Mesh` only draws when its fill is `FillRef::Solid` (gradients/patterns
//!   are skipped, matching `capabilities().gradients == false`); `Sprite`
//!   uses `tint` directly as the cell color.
//! - Only `BlendMode::Normal` (source-over/Porter-Duff "over") is evaluated:
//!   `capabilities().supported_blend_modes` is `&[BlendMode::Normal]` and
//!   `blend_modes` is `false`. Every command's `blend` field is otherwise
//!   ignored — non-`Normal` variants fall back to source-over compositing
//!   the same way `adapters::webgl` falls its own unsupported `Overlay`
//!   back to `Normal`. Compositing uses straight (non-premultiplied) RGBA;
//!   see `composite_over`.
//! - Clip masks and group effects (blur/shadow/color-matrix/opacity) are
//!   accepted but ignored; only a `BeginGroup`'s `transform` is honored, via
//!   a transform stack composed through `Transform::to_mat3`.
//!
//! This closes a previously untracked implementation gap: the adapter used
//! to be a 7-line stub (`mod private {}`, an empty `mod_interface!{}`, and a
//! "Status: stub only" doc comment) with no `Backend` impl anywhere and no
//! tracking in `task/`/`bug/`, despite the `adapter-terminal` feature gate
//! already existing and compiling. See
//! `docs/feature/003_terminal_backend_adapter.md`.

mod private
{
  use crate::assets::Assets;
  use crate::backend::{ Backend, Capabilities, Output, RenderError };
  use crate::commands::
  {
    AddMeshInstance,
    AddSpriteInstance,
    ArcTo,
    BeginGroup,
    BeginPath,
    BeginText,
    BindBatch,
    Char,
    Clear,
    CreateMeshBatch,
    CreateSpriteBatch,
    CubicTo,
    DeleteBatch,
    DrawBatch,
    LineTo,
    Mesh,
    MeshBatchParams,
    MoveTo,
    QuadTo,
    RemoveInstance,
    RenderCommand,
    SetMeshBatchParams,
    SetMeshInstance,
    SetSpriteBatchParams,
    SetSpriteInstance,
    Sprite,
    SpriteBatchParams,
  };
  use crate::types::{ asset, Batch, BlendMode, FillRef, RenderConfig, ResourceId, TextAnchor, Transform };
  use core::fmt::Write as _;
  use nohash_hasher::{ IntMap, IntSet };

  // ============================================================================
  // Cell grid
  // ============================================================================

  /// One character cell in the terminal grid.
  #[ derive( Debug, Clone, Copy ) ]
  struct TerminalCell
  {
    /// Glyph occupying this cell. `' '` (space) means "no text here" — the
    /// cell then renders as a solid color block via `bg`.
    glyph : char,
    /// Background color. Always meaningful — `Clear`/path/`Mesh`/`Sprite`
    /// paint this, and it is what renders when `glyph == ' '`.
    bg : [ f32; 4 ],
    /// Foreground color. Meaningful only when `glyph != ' '` (set by text
    /// commands); otherwise carries a stale default and is not rendered.
    fg : [ f32; 4 ],
  }

  impl TerminalCell
  {
    fn blank( bg : [ f32; 4 ] ) -> Self
    {
      Self { glyph : ' ', bg, fg : [ 1.0, 1.0, 1.0, 1.0 ] }
    }
  }

  /// Internal storage for a batch's instances + shared params.
  enum TerminalBatch
  {
    /// A sprite batch.
    Sprite
    {
      /// Instances currently in the batch.
      instances : Vec< AddSpriteInstance >,
      /// Parameters common to all instances.
      params : SpriteBatchParams,
    },
    /// A mesh batch.
    Mesh
    {
      /// Instances currently in the batch.
      instances : Vec< AddMeshInstance >,
      /// Parameters common to all instances.
      params : MeshBatchParams,
    },
  }

  /// Applies a 2D affine [`Transform`] to a single point via
  /// `Transform::to_mat3`'s column-major `[ m00, m10, 0, m01, m11, 0, tx, ty, 1 ]`
  /// layout: `x' = m00*x + m01*y + tx`, `y' = m10*x + m11*y + ty`.
  fn affine_apply( t : &Transform, p : [ f32; 2 ] ) -> [ f32; 2 ]
  {
    let m = t.to_mat3();
    [ m[ 0 ] * p[ 0 ] + m[ 3 ] * p[ 1 ] + m[ 6 ], m[ 1 ] * p[ 0 ] + m[ 4 ] * p[ 1 ] + m[ 7 ] ]
  }

  // ============================================================================
  // Compositing
  // ============================================================================

  /// Composites `src` over `dst` using source-over (Porter-Duff "over")
  /// alpha blending on straight (non-premultiplied) RGBA — the only
  /// [`crate::types::BlendMode`] variant this backend evaluates per-pixel;
  /// every other variant currently falls back to this, matching the
  /// "fall back to Normal" convention `adapters::webgl` already uses for
  /// its own unsupported `Overlay` variant. Returns transparent black when
  /// both alphas are ~0 (avoids a near-zero division).
  fn composite_over( dst : [ f32; 4 ], src : [ f32; 4 ] ) -> [ f32; 4 ]
  {
    let sa = src[ 3 ].clamp( 0.0, 1.0 );
    let da = dst[ 3 ].clamp( 0.0, 1.0 );
    let out_a = sa + da * ( 1.0 - sa );
    if out_a < f32::EPSILON
    {
      return [ 0.0, 0.0, 0.0, 0.0 ];
    }
    [
      ( src[ 0 ] * sa + dst[ 0 ] * da * ( 1.0 - sa ) ) / out_a,
      ( src[ 1 ] * sa + dst[ 1 ] * da * ( 1.0 - sa ) ) / out_a,
      ( src[ 2 ] * sa + dst[ 2 ] * da * ( 1.0 - sa ) ) / out_a,
      out_a,
    ]
  }

  // ============================================================================
  // Curve flattening
  // ============================================================================

  /// Straight-line segments used to flatten one `QuadTo`/`CubicTo`/`ArcTo`
  /// into `path_points`. Fixed rather than adaptive (arc-length- or
  /// curvature-based) — good enough at this adapter's coarse cell
  /// resolution, and avoids the extra complexity adaptive subdivision would
  /// add for no visible benefit here.
  const CURVE_SEGMENTS : u32 = 16;

  /// Flattens a quadratic Bezier (start `p0`, control `c`, end `p1`) into
  /// [`CURVE_SEGMENTS`] points, excluding `p0` (already the last point in
  /// `path_points`) but including `p1`.
  fn flatten_quad( p0 : [ f32; 2 ], c : [ f32; 2 ], p1 : [ f32; 2 ] ) -> Vec< [ f32; 2 ] >
  {
    ( 1..=CURVE_SEGMENTS ).map( | i |
    {
      let t = i as f32 / CURVE_SEGMENTS as f32;
      let mt = 1.0 - t;
      [
        mt * mt * p0[ 0 ] + 2.0 * mt * t * c[ 0 ] + t * t * p1[ 0 ],
        mt * mt * p0[ 1 ] + 2.0 * mt * t * c[ 1 ] + t * t * p1[ 1 ],
      ]
    }).collect()
  }

  /// Flattens a cubic Bezier (start `p0`, controls `c1`/`c2`, end `p1`) into
  /// [`CURVE_SEGMENTS`] points, excluding `p0` but including `p1`.
  fn flatten_cubic( p0 : [ f32; 2 ], c1 : [ f32; 2 ], c2 : [ f32; 2 ], p1 : [ f32; 2 ] ) -> Vec< [ f32; 2 ] >
  {
    ( 1..=CURVE_SEGMENTS ).map( | i |
    {
      let t = i as f32 / CURVE_SEGMENTS as f32;
      let mt = 1.0 - t;
      let ( mt2, t2 ) = ( mt * mt, t * t );
      let ( mt3, t3 ) = ( mt2 * mt, t2 * t );
      [
        mt3 * p0[ 0 ] + 3.0 * mt2 * t * c1[ 0 ] + 3.0 * mt * t2 * c2[ 0 ] + t3 * p1[ 0 ],
        mt3 * p0[ 1 ] + 3.0 * mt2 * t * c1[ 1 ] + 3.0 * mt * t2 * c2[ 1 ] + t3 * p1[ 1 ],
      ]
    }).collect()
  }

  /// Flattens an SVG-style elliptical arc (start `p0`, per-command `a`) into
  /// [`CURVE_SEGMENTS`] points, excluding `p0` but including the endpoint.
  /// Implements the endpoint-to-center parameterization from the SVG 1.1
  /// spec (Appendix F.6.5) in `f64` for numerical stability, then walks the
  /// resulting ellipse arc parametrically. A degenerate arc (`rx`/`ry`
  /// zero, or `p0` == endpoint) falls back to a single point at the
  /// endpoint, matching the spec's own degenerate-arc-as-line handling.
  fn flatten_arc( p0 : [ f32; 2 ], a : &ArcTo ) -> Vec< [ f32; 2 ] >
  {
    let ( x1, y1 ) = ( f64::from( p0[ 0 ] ), f64::from( p0[ 1 ] ) );
    let ( x2, y2 ) = ( f64::from( a.x ), f64::from( a.y ) );
    let ( mut rx, mut ry ) = ( f64::from( a.rx ).abs(), f64::from( a.ry ).abs() );
    let phi = f64::from( a.rotation );

    if rx < f64::EPSILON || ry < f64::EPSILON
      || ( ( x1 - x2 ).abs() < f64::EPSILON && ( y1 - y2 ).abs() < f64::EPSILON )
    {
      return vec![ [ a.x, a.y ] ];
    }

    let ( sin_phi, cos_phi ) = phi.sin_cos();
    let ( dx2, dy2 ) = ( ( x1 - x2 ) / 2.0, ( y1 - y2 ) / 2.0 );
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    let lambda = ( x1p * x1p ) / ( rx * rx ) + ( y1p * y1p ) / ( ry * ry );
    if lambda > 1.0
    {
      let s = lambda.sqrt();
      rx *= s;
      ry *= s;
    }

    let sign = if a.large_arc == a.sweep { -1.0 } else { 1.0 };
    let num = ( rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p ).max( 0.0 );
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    let co = sign * ( num / den ).sqrt();
    let cxp = co * ( rx * y1p / ry );
    let cyp = co * ( -ry * x1p / rx );

    let cx = cos_phi * cxp - sin_phi * cyp + f64::midpoint( x1, x2 );
    let cy = sin_phi * cxp + cos_phi * cyp + f64::midpoint( y1, y2 );

    let angle = | ux : f64, uy : f64, vx : f64, vy : f64 | -> f64
    {
      let dot = ux * vx + uy * vy;
      let len = ( ux * ux + uy * uy ).sqrt() * ( vx * vx + vy * vy ).sqrt();
      let sign = if ux * vy - uy * vx < 0.0 { -1.0 } else { 1.0 };
      sign * ( dot / len ).clamp( -1.0, 1.0 ).acos()
    };

    let theta1 = angle( 1.0, 0.0, ( x1p - cxp ) / rx, ( y1p - cyp ) / ry );
    let mut delta = angle
    (
      ( x1p - cxp ) / rx, ( y1p - cyp ) / ry,
      ( -x1p - cxp ) / rx, ( -y1p - cyp ) / ry,
    );
    if !a.sweep && delta > 0.0 { delta -= core::f64::consts::TAU; }
    if a.sweep && delta < 0.0 { delta += core::f64::consts::TAU; }

    ( 1..=CURVE_SEGMENTS ).map( | i |
    {
      let t = f64::from( i ) / f64::from( CURVE_SEGMENTS );
      let theta = theta1 + t * delta;
      let ( sin_t, cos_t ) = theta.sin_cos();
      let x = cx + rx * cos_phi * cos_t - ry * sin_phi * sin_t;
      let y = cy + rx * sin_phi * cos_t + ry * cos_phi * sin_t;
      [ x as f32, y as f32 ]
    }).collect()
  }

  // ============================================================================
  // Backend struct
  // ============================================================================

  /// Terminal renderer backend — renders a command stream to a coarse,
  /// ANSI-colored character-cell grid. See the module docs for the exact
  /// world-to-cell mapping and known simplifications.
  ///
  /// ```ignore
  /// let mut term = TerminalBackend::new( config );
  /// term.assets_load( &assets )?;
  /// term.submit( &commands )?;
  /// let Output::String( frame ) = term.output()? else { unreachable!() };
  /// print!( "{frame}" );
  /// ```
  pub struct TerminalBackend
  {
    config : RenderConfig,
    cols : u32,
    rows : u32,
    cells : Vec< TerminalCell >,
    sprite_ids : IntSet< ResourceId< asset::Sprite > >,
    geometry_ids : IntSet< ResourceId< asset::Geometry > >,
    batches : IntMap< ResourceId< Batch >, TerminalBatch >,
    recording_batch : Option< ResourceId< Batch > >,
    group_stack : Vec< Transform >,
    path_style : Option< BeginPath >,
    path_points : Vec< [ f32; 2 ] >,
    subpath_start : Option< [ f32; 2 ] >,
    text_style : Option< BeginText >,
    text_buf : String,
  }

  impl TerminalBackend
  {
    /// World-space width of one character cell, in the same units as
    /// `RenderConfig::width`.
    const CELL_PX_WIDTH : u32 = 16;
    /// World-space height of one character cell. Taller than
    /// `CELL_PX_WIDTH` to approximate a monospace glyph's ~1:2 aspect ratio.
    const CELL_PX_HEIGHT : u32 = 32;

    /// Creates a new terminal backend from a render config. Grid dimensions
    /// are derived from `config.width`/`config.height` (rounded up so the
    /// grid always fully covers the viewport — see `cells_for`).
    #[ inline ]
    #[ must_use ]
    pub fn new( config : RenderConfig ) -> Self
    {
      let cols = Self::cells_for( config.width, Self::CELL_PX_WIDTH );
      let rows = Self::cells_for( config.height, Self::CELL_PX_HEIGHT );
      Self
      {
        config,
        cols,
        rows,
        cells : vec![ TerminalCell::blank( config.background ); ( cols * rows ) as usize ],
        sprite_ids : IntSet::default(),
        geometry_ids : IntSet::default(),
        batches : IntMap::default(),
        recording_batch : None,
        group_stack : Vec::new(),
        path_style : None,
        path_points : Vec::new(),
        subpath_start : None,
        text_style : None,
        text_buf : String::new(),
      }
    }

    /// Ceiling-divides `pixels` by `cell_px` so the resulting cell count
    /// always fully covers `pixels` (never truncates a trailing partial
    /// cell), clamped to a minimum of 1.
    fn cells_for( pixels : u32, cell_px : u32 ) -> u32
    {
      pixels.div_ceil( cell_px ).max( 1 )
    }

    /// Grid width in character cells.
    ///
    /// `#[doc(hidden)]`: implementation detail, public only so its tests can
    /// live in `tests/` (mirrors `SvgContentManager`'s test-support fields).
    #[ doc( hidden ) ]
    #[ inline ]
    #[ must_use ]
    pub fn cols( &self ) -> u32 { self.cols }

    /// Grid height in character cells. See [`Self::cols`].
    #[ doc( hidden ) ]
    #[ inline ]
    #[ must_use ]
    pub fn rows( &self ) -> u32 { self.rows }

    /// Background color of the cell at `(col, row)` — row 0 is the top row
    /// (screen convention). `None` if out of bounds. See [`Self::cols`].
    #[ doc( hidden ) ]
    #[ inline ]
    #[ must_use ]
    pub fn cell_bg( &self, col : u32, row : u32 ) -> Option< [ f32; 4 ] >
    {
      self.cell( col, row ).map( | c | c.bg )
    }

    /// Foreground color of the cell at `(col, row)`. See [`Self::cell_bg`].
    #[ doc( hidden ) ]
    #[ inline ]
    #[ must_use ]
    pub fn cell_fg( &self, col : u32, row : u32 ) -> Option< [ f32; 4 ] >
    {
      self.cell( col, row ).map( | c | c.fg )
    }

    /// Glyph occupying the cell at `(col, row)`; `' '` means "no text here".
    /// See [`Self::cell_bg`].
    #[ doc( hidden ) ]
    #[ inline ]
    #[ must_use ]
    pub fn cell_glyph( &self, col : u32, row : u32 ) -> Option< char >
    {
      self.cell( col, row ).map( | c | c.glyph )
    }

    fn cell( &self, col : u32, row : u32 ) -> Option< &TerminalCell >
    {
      if col >= self.cols || row >= self.rows { return None; }
      self.cells.get( ( row * self.cols + col ) as usize )
    }

    fn cell_mut( &mut self, col : u32, row : u32 ) -> Option< &mut TerminalCell >
    {
      if col >= self.cols || row >= self.rows { return None; }
      self.cells.get_mut( ( row * self.cols + col ) as usize )
    }

    fn plot_bg( &mut self, col : u32, row : u32, color : [ f32; 4 ] )
    {
      if let Some( cell ) = self.cell_mut( col, row )
      {
        cell.glyph = ' ';
        cell.bg = composite_over( cell.bg, color );
      }
    }

    fn plot_glyph( &mut self, col : u32, row : u32, glyph : char, color : [ f32; 4 ] )
    {
      if let Some( cell ) = self.cell_mut( col, row )
      {
        cell.glyph = glyph;
        cell.fg = color;
      }
    }

    /// Maps a world-space point (Y-up, per [invariant/001](../../docs/invariant/001_y_up_coordinate_system.md))
    /// to a grid cell, flipping Y the same way `SvgBackend`'s transform
    /// conversion does (`height - y`) before dividing into cell units.
    /// `None` if the point falls outside the grid or is non-finite.
    fn world_to_cell( &self, p : [ f32; 2 ] ) -> Option< ( u32, u32 ) >
    {
      let col_f = p[ 0 ] / Self::CELL_PX_WIDTH as f32;
      let row_f = ( self.config.height as f32 - p[ 1 ] ) / Self::CELL_PX_HEIGHT as f32;
      if !( col_f.is_finite() && row_f.is_finite() ) || col_f < 0.0 || row_f < 0.0
      {
        return None;
      }
      let col = col_f.floor() as u32;
      let row = row_f.floor() as u32;
      if col >= self.cols || row >= self.rows { return None; }
      Some( ( col, row ) )
    }

    /// Folds `p` through the active `BeginGroup`/`EndGroup` transform stack,
    /// innermost (last pushed) first — so a point local to the innermost
    /// group ends up in world space.
    fn group_apply( &self, p : [ f32; 2 ] ) -> [ f32; 2 ]
    {
      let mut p = p;
      for t in self.group_stack.iter().rev()
      {
        p = affine_apply( t, p );
      }
      p
    }

    fn resolve_cell( &self, local : [ f32; 2 ] ) -> Option< ( u32, u32 ) >
    {
      self.world_to_cell( self.group_apply( local ) )
    }

    fn resolve_path_cell( &self, style_transform : &Transform, raw : [ f32; 2 ] ) -> Option< ( u32, u32 ) >
    {
      self.resolve_cell( affine_apply( style_transform, raw ) )
    }

    /// True Bresenham line rasterization between two cells — the symmetric
    /// integer variant that steps whichever axis (or both, on the same
    /// iteration) the accumulated error demands, rather than picking one
    /// "major" axis and stepping it every iteration. This form always
    /// produces the same cell set regardless of direction:
    /// `line_cells(a, b)` equals `line_cells(b, a)` reversed. Always
    /// includes both endpoints.
    fn line_cells( a : ( u32, u32 ), b : ( u32, u32 ) ) -> Vec< ( u32, u32 ) >
    {
      let ( mut x0, mut y0 ) = ( i64::from( a.0 ), i64::from( a.1 ) );
      let ( x1, y1 ) = ( i64::from( b.0 ), i64::from( b.1 ) );

      let dx = ( x1 - x0 ).abs();
      let dy = -( y1 - y0 ).abs();
      let sx : i64 = if x0 < x1 { 1 } else { -1 };
      let sy : i64 = if y0 < y1 { 1 } else { -1 };
      let mut err = dx + dy;

      let mut cells = Vec::new();
      loop
      {
        cells.push( ( x0 as u32, y0 as u32 ) );
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy
        {
          err += dy;
          x0 += sx;
        }
        if e2 <= dx
        {
          err += dx;
          y0 += sy;
        }
      }
      cells
    }

    fn to_rgb8( c : [ f32; 4 ] ) -> ( u8, u8, u8 )
    {
      (
        ( c[ 0 ].clamp( 0.0, 1.0 ) * 255.0 ) as u8,
        ( c[ 1 ].clamp( 0.0, 1.0 ) * 255.0 ) as u8,
        ( c[ 2 ].clamp( 0.0, 1.0 ) * 255.0 ) as u8,
      )
    }

    /// Renders the full grid as ANSI SGR truecolor escape sequences — one
    /// line per row. A `' '` glyph cell emits a background-colored space
    /// (`ESC[48;2;r;g;bm `); any other glyph emits it foreground-colored
    /// (`ESC[38;2;r;g;bmX`). Each row ends with a reset (`ESC[0m`) then
    /// `\n`.
    fn render_ansi( &self ) -> String
    {
      let mut out = String::with_capacity( ( self.cols * self.rows * 12 ) as usize );
      for row in 0..self.rows
      {
        for col in 0..self.cols
        {
          let cell = self.cell( col, row ).copied().unwrap_or_else( || TerminalCell::blank( self.config.background ) );
          if cell.glyph == ' '
          {
            let ( r, g, b ) = Self::to_rgb8( cell.bg );
            let _ = write!( out, "\x1b[48;2;{r};{g};{b}m " );
          }
          else
          {
            let ( r, g, b ) = Self::to_rgb8( cell.fg );
            let _ = write!( out, "\x1b[38;2;{r};{g};{b}m{}", cell.glyph );
          }
        }
        out.push_str( "\x1b[0m\n" );
      }
      out
    }

    // ---- command handlers ----

    fn cmd_clear( &mut self, c : &Clear )
    {
      for cell in &mut self.cells { *cell = TerminalCell::blank( c.color ); }
    }

    fn cmd_begin_path( &mut self, bp : &BeginPath )
    {
      self.path_points.clear();
      self.subpath_start = None;
      self.path_style = Some( *bp );
    }

    fn cmd_move_to( &mut self, m : MoveTo )
    {
      self.path_points.push( [ m.0, m.1 ] );
      self.subpath_start = Some( [ m.0, m.1 ] );
    }

    fn cmd_line_to( &mut self, l : LineTo )
    {
      self.path_points.push( [ l.0, l.1 ] );
    }

    // Curves flatten into `CURVE_SEGMENTS` straight-line points via the
    // free functions above — see "Curve flattening" and the module docs'
    // "Known simplifications".
    fn cmd_quad_to( &mut self, q : &QuadTo )
    {
      let p0 = self.path_points.last().copied().unwrap_or( [ q.x, q.y ] );
      self.path_points.extend( flatten_quad( p0, [ q.cx, q.cy ], [ q.x, q.y ] ) );
    }

    fn cmd_cubic_to( &mut self, c : &CubicTo )
    {
      let p0 = self.path_points.last().copied().unwrap_or( [ c.x, c.y ] );
      self.path_points.extend( flatten_cubic( p0, [ c.c1x, c.c1y ], [ c.c2x, c.c2y ], [ c.x, c.y ] ) );
    }

    fn cmd_arc_to( &mut self, a : &ArcTo )
    {
      let p0 = self.path_points.last().copied().unwrap_or( [ a.x, a.y ] );
      self.path_points.extend( flatten_arc( p0, a ) );
    }

    fn cmd_close_path( &mut self )
    {
      if let Some( start ) = self.subpath_start
      {
        self.path_points.push( start );
      }
    }

    fn cmd_end_path( &mut self )
    {
      let Some( style ) = self.path_style.take() else { return; };
      let points = core::mem::take( &mut self.path_points );
      self.subpath_start = None;

      let cells : Vec< ( u32, u32 ) > = points.iter()
      .filter_map( | &raw | self.resolve_path_cell( &style.transform, raw ) )
      .collect();

      for pair in cells.windows( 2 )
      {
        for ( col, row ) in Self::line_cells( pair[ 0 ], pair[ 1 ] )
        {
          self.plot_bg( col, row, style.stroke_color );
        }
      }
    }

    fn cmd_begin_text( &mut self, bt : &BeginText )
    {
      self.text_buf.clear();
      self.text_style = Some( *bt );
    }

    fn cmd_char( &mut self, ch : Char )
    {
      self.text_buf.push( ch.0 );
    }

    // Single row per BeginText/EndText run: vertical anchor (Top/Center/
    // Bottom) nudges `position`'s Y by a fraction of one cell height before
    // the row lookup, the same way SVG maps it to `dominant-baseline`
    // (hanging/central/baseline) — Top leaves Y unshifted (position is the
    // row's top edge), Center adds half a cell (position is the row's
    // vertical center), Bottom adds a full cell (position is the row's
    // bottom edge). This can't be a row-count shift the way horizontal
    // shift uses whole character columns: text always spans exactly one
    // row, so the anchor must operate at sub-cell (world-unit) precision
    // instead. Horizontal anchor shifts the starting column the same way
    // SVG's own anchor conversion groups its three left/center/right cases.
    fn cmd_end_text( &mut self )
    {
      let Some( style ) = self.text_style.take() else { return; };
      let text = core::mem::take( &mut self.text_buf );

      let dy = match style.anchor
      {
        TextAnchor::TopLeft | TextAnchor::TopCenter | TextAnchor::TopRight => 0.0,
        TextAnchor::CenterLeft | TextAnchor::Center | TextAnchor::CenterRight => Self::CELL_PX_HEIGHT as f32 / 2.0,
        TextAnchor::BottomLeft | TextAnchor::BottomCenter | TextAnchor::BottomRight => Self::CELL_PX_HEIGHT as f32,
      };
      let anchored_position = [ style.position[ 0 ], style.position[ 1 ] + dy ];
      let Some( ( base_col, base_row ) ) = self.resolve_cell( anchored_position ) else { return; };

      let len = text.chars().count() as i64;
      let shift = match style.anchor
      {
        TextAnchor::TopLeft | TextAnchor::CenterLeft | TextAnchor::BottomLeft => 0,
        TextAnchor::TopCenter | TextAnchor::Center | TextAnchor::BottomCenter => len / 2,
        TextAnchor::TopRight | TextAnchor::CenterRight | TextAnchor::BottomRight => len,
      };
      let start_col = i64::from( base_col ) - shift;

      for ( i, ch ) in text.chars().enumerate()
      {
        let col = start_col + i as i64;
        if col < 0 { continue; }
        self.plot_glyph( col as u32, base_row, ch, style.color );
      }
    }

    fn cmd_mesh( &mut self, m : &Mesh ) -> Result< (), RenderError >
    {
      if !self.geometry_ids.contains( &m.geometry )
      {
        return Err( RenderError::MissingAsset( m.geometry.inner() ) );
      }
      if let FillRef::Solid( color ) = m.fill
        && let Some( ( col, row ) ) = self.resolve_cell( m.transform.position )
      {
        self.plot_bg( col, row, color );
      }
      Ok( () )
    }

    fn cmd_sprite( &mut self, s : &Sprite ) -> Result< (), RenderError >
    {
      if !self.sprite_ids.contains( &s.sprite )
      {
        return Err( RenderError::MissingAsset( s.sprite.inner() ) );
      }
      if let Some( ( col, row ) ) = self.resolve_cell( s.transform.position )
      {
        self.plot_bg( col, row, s.tint );
      }
      Ok( () )
    }

    fn cmd_create_sprite_batch( &mut self, cb : &CreateSpriteBatch )
    {
      self.batches.insert( cb.batch, TerminalBatch::Sprite { instances : Vec::new(), params : cb.params } );
    }

    fn cmd_create_mesh_batch( &mut self, cb : &CreateMeshBatch )
    {
      self.batches.insert( cb.batch, TerminalBatch::Mesh { instances : Vec::new(), params : cb.params } );
    }

    fn cmd_bind_batch( &mut self, bb : BindBatch )
    {
      self.recording_batch = Some( bb.batch );
    }

    fn cmd_add_sprite_instance( &mut self, si : &AddSpriteInstance )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( TerminalBatch::Sprite { instances, .. } ) = self.batches.get_mut( &batch_id )
      {
        instances.push( *si );
      }
    }

    fn cmd_add_mesh_instance( &mut self, mi : &AddMeshInstance )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( TerminalBatch::Mesh { instances, .. } ) = self.batches.get_mut( &batch_id )
      {
        instances.push( *mi );
      }
    }

    fn cmd_set_sprite_instance( &mut self, si : &SetSpriteInstance ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ); };
      let Some( TerminalBatch::Sprite { instances, .. } ) = self.batches.get_mut( &batch_id )
      else
      {
        return Ok( () );
      };
      if ( si.index as usize ) >= instances.len()
      {
        return Err( RenderError::BackendError
        (
          format!( "SetSpriteInstance: index {} out of bounds (len {})", si.index, instances.len() )
        ) );
      }
      instances[ si.index as usize ] = AddSpriteInstance { transform : si.transform, sprite : si.sprite, tint : si.tint };
      Ok( () )
    }

    fn cmd_set_mesh_instance( &mut self, mi : &SetMeshInstance ) -> Result< (), RenderError >
    {
      let Some( batch_id ) = self.recording_batch else { return Ok( () ); };
      let Some( TerminalBatch::Mesh { instances, .. } ) = self.batches.get_mut( &batch_id )
      else
      {
        return Ok( () );
      };
      if ( mi.index as usize ) >= instances.len()
      {
        return Err( RenderError::BackendError
        (
          format!( "SetMeshInstance: index {} out of bounds (len {})", mi.index, instances.len() )
        ) );
      }
      instances[ mi.index as usize ] = AddMeshInstance { transform : mi.transform, tint : mi.tint };
      Ok( () )
    }

    fn cmd_remove_instance( &mut self, ri : RemoveInstance )
    {
      let Some( batch_id ) = self.recording_batch else { return; };
      match self.batches.get_mut( &batch_id )
      {
        Some( TerminalBatch::Sprite { instances, .. } ) if ( ri.index as usize ) < instances.len() =>
        {
          instances.swap_remove( ri.index as usize );
        }
        Some( TerminalBatch::Mesh { instances, .. } ) if ( ri.index as usize ) < instances.len() =>
        {
          instances.swap_remove( ri.index as usize );
        }
        _ => {}
      }
    }

    fn cmd_set_sprite_batch_params( &mut self, sp : &SetSpriteBatchParams )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( TerminalBatch::Sprite { params, .. } ) = self.batches.get_mut( &batch_id )
      {
        *params = sp.params;
      }
    }

    fn cmd_set_mesh_batch_params( &mut self, mp : &SetMeshBatchParams )
    {
      if let Some( batch_id ) = self.recording_batch
        && let Some( TerminalBatch::Mesh { params, .. } ) = self.batches.get_mut( &batch_id )
      {
        *params = mp.params;
      }
    }

    fn cmd_unbind_batch( &mut self )
    {
      self.recording_batch = None;
    }

    fn cmd_draw_batch( &mut self, db : DrawBatch )
    {
      // Collected into an owned Vec first so the `self.batches` borrow ends
      // here, before the loop below needs a fresh `&mut self` for
      // `resolve_cell`/`plot_bg`.
      let plots : Vec< ( [ f32; 2 ], [ f32; 4 ] ) > = match self.batches.get( &db.batch )
      {
        Some( TerminalBatch::Sprite { instances, params } ) =>
        {
          instances.iter()
          .map( | inst | ( affine_apply( &params.transform, inst.transform.position ), inst.tint ) )
          .collect()
        }
        Some( TerminalBatch::Mesh { instances, params } ) =>
        {
          if let FillRef::Solid( color ) = params.fill
          {
            instances.iter()
            .map( | inst | ( affine_apply( &params.transform, inst.transform.position ), color ) )
            .collect()
          }
          else
          {
            Vec::new()
          }
        }
        None => Vec::new(),
      };

      for ( pos, color ) in plots
      {
        if let Some( ( col, row ) ) = self.resolve_cell( pos )
        {
          self.plot_bg( col, row, color );
        }
      }
    }

    fn cmd_delete_batch( &mut self, db : DeleteBatch )
    {
      self.batches.remove( &db.batch );
      if self.recording_batch == Some( db.batch )
      {
        self.recording_batch = None;
      }
    }

    fn cmd_begin_group( &mut self, bg : &BeginGroup )
    {
      self.group_stack.push( bg.transform );
    }

    fn cmd_end_group( &mut self )
    {
      self.group_stack.pop();
    }
  }

  // ============================================================================
  // Backend trait impl
  // ============================================================================

  impl Backend for TerminalBackend
  {
    #[ inline ]
    fn assets_load( &mut self, assets : &Assets ) -> Result< (), RenderError >
    {
      self.sprite_ids.clear();
      self.geometry_ids.clear();
      for s in &assets.sprites { self.sprite_ids.insert( s.id ); }
      for g in &assets.geometries { self.geometry_ids.insert( g.id ); }
      // Trait contract ("Each call replaces all previously loaded assets...
      // including any active batches" — see `Backend::assets_load` docs).
      self.batches.clear();
      self.recording_batch = None;
      Ok( () )
    }

    #[ inline ]
    fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
    {
      let bg = self.config.background;
      for cell in &mut self.cells { *cell = TerminalCell::blank( bg ); }
      self.group_stack.clear();
      self.recording_batch = None;
      // An unterminated BeginPath/BeginText from a malformed previous frame
      // would otherwise leak accumulator state into this one.
      self.path_points.clear();
      self.path_style = None;
      self.subpath_start = None;
      self.text_buf.clear();
      self.text_style = None;

      for cmd in commands
      {
        match cmd
        {
          RenderCommand::Clear( c ) => self.cmd_clear( c ),
          RenderCommand::BeginPath( bp ) => self.cmd_begin_path( bp ),
          RenderCommand::MoveTo( m ) => self.cmd_move_to( *m ),
          RenderCommand::LineTo( l ) => self.cmd_line_to( *l ),
          RenderCommand::QuadTo( q ) => self.cmd_quad_to( q ),
          RenderCommand::CubicTo( c ) => self.cmd_cubic_to( c ),
          RenderCommand::ArcTo( a ) => self.cmd_arc_to( a ),
          RenderCommand::ClosePath( _ ) => self.cmd_close_path(),
          RenderCommand::EndPath( _ ) => self.cmd_end_path(),
          RenderCommand::BeginText( bt ) => self.cmd_begin_text( bt ),
          RenderCommand::Char( ch ) => self.cmd_char( *ch ),
          RenderCommand::EndText( _ ) => self.cmd_end_text(),
          RenderCommand::Mesh( m ) => self.cmd_mesh( m )?,
          // ScreenSpaceSprite shares the Sprite payload; this backend has no
          // camera/viewport transform of its own to bypass (unlike SVG's
          // viewport pan/zoom), so both variants already behave identically.
          RenderCommand::Sprite( s ) | RenderCommand::ScreenSpaceSprite( s ) => self.cmd_sprite( s )?,
          RenderCommand::CreateSpriteBatch( cb ) => self.cmd_create_sprite_batch( cb ),
          RenderCommand::CreateMeshBatch( cb ) => self.cmd_create_mesh_batch( cb ),
          RenderCommand::BindBatch( bb ) => self.cmd_bind_batch( *bb ),
          RenderCommand::AddSpriteInstance( si ) => self.cmd_add_sprite_instance( si ),
          RenderCommand::AddMeshInstance( mi ) => self.cmd_add_mesh_instance( mi ),
          RenderCommand::SetSpriteInstance( si ) => self.cmd_set_sprite_instance( si )?,
          RenderCommand::SetMeshInstance( mi ) => self.cmd_set_mesh_instance( mi )?,
          RenderCommand::RemoveInstance( ri ) => self.cmd_remove_instance( *ri ),
          RenderCommand::SetSpriteBatchParams( sp ) => self.cmd_set_sprite_batch_params( sp ),
          RenderCommand::SetMeshBatchParams( mp ) => self.cmd_set_mesh_batch_params( mp ),
          RenderCommand::UnbindBatch( _ ) => self.cmd_unbind_batch(),
          RenderCommand::DrawBatch( db ) => self.cmd_draw_batch( *db ),
          RenderCommand::DeleteBatch( db ) => self.cmd_delete_batch( *db ),
          RenderCommand::BeginGroup( bg ) => self.cmd_begin_group( bg ),
          RenderCommand::EndGroup( _ ) => self.cmd_end_group(),
          // No depth buffer in the terminal backend — the opaque/transparent
          // pass split is a GPU-only optimisation; ignore.
          RenderCommand::SetDepthWrite( _ ) => {},
        }
      }

      Ok( () )
    }

    #[ inline ]
    fn output( &self ) -> Result< Output, RenderError >
    {
      Ok( Output::String( self.render_ansi() ) )
    }

    #[ inline ]
    fn resize( &mut self, width : u32, height : u32 )
    {
      self.config.width = width;
      self.config.height = height;
      self.cols = Self::cells_for( width, Self::CELL_PX_WIDTH );
      self.rows = Self::cells_for( height, Self::CELL_PX_HEIGHT );
      self.cells = vec![ TerminalCell::blank( self.config.background ); ( self.cols * self.rows ) as usize ];
    }

    #[ inline ]
    fn capabilities( &self ) -> Capabilities
    {
      // Coarse/simplified, not absent — see the module docs' "Known
      // simplifications" for exactly what each `true` here does and
      // doesn't cover.
      Capabilities
      {
        paths : true,
        text : true,
        meshes : true,
        sprites : true,
        batches : true,
        gradients : false,
        patterns : false,
        clip_masks : false,
        effects : false,
        blend_modes : false,
        supported_blend_modes : &[ BlendMode::Normal ],
        text_on_path : false,
        max_texture_size : 0,
      }
    }
  }
}

mod_interface::mod_interface!
{
  own use TerminalBackend;
}
