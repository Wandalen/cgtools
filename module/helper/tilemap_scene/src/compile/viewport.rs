//! Viewport-anchored rendering — screen-space transforms for
//! [`crate::source::SpriteSource::ViewportTiled`].
//!
//! Slice 4 covers `ViewportTiling::{ Center, Stretch, Fit }` — each produces
//! exactly one [`tilemap_renderer::commands::RenderCommand::ScreenSpaceSprite`] per
//! viewport instance. The tiled variants (`Repeat2D` / `RepeatX` / `RepeatY`)
//! need a `Mesh` command with wrap-enabled UVs and are deferred.

mod private
{
  use crate::source::{ ViewportAnchorPoint, ViewportTiling };
  use tilemap_renderer::types::Transform;

  /// Compute the screen-space [`Transform`] for a viewport-anchored sprite.
  ///
  /// - `sprite_size` is the sprite's natural pixel dimensions (from its atlas
  ///   region).
  /// - `viewport_size` is the screen viewport dimensions in pixels.
  ///
  /// Returns `None` only when the input is malformed (currently always
  /// succeeds for supported tilings). For tiled modes (`Repeat*`) this
  /// returns the transform of the **first** tile; the caller is responsible
  /// for calling [`tiled_positions`] to enumerate the rest.
  #[ must_use ]
  pub fn viewport_transform
  (
    tiling : ViewportTiling,
    anchor : ViewportAnchorPoint,
    sprite_size : ( f32, f32 ),
    viewport_size : ( u32, u32 ),
  ) -> Option< Transform >
  {
    let vw = viewport_size.0 as f32;
    let vh = viewport_size.1 as f32;

    match tiling
    {
      ViewportTiling::Stretch =>
      {
        // Fill the whole viewport, non-uniform scale allowed.
        let sx = vw / sprite_size.0.max( 1.0 );
        let sy = vh / sprite_size.1.max( 1.0 );
        Some( make_transform( ( 0.0, 0.0 ), ( sx, sy ) ) )
      },
      ViewportTiling::Fit =>
      {
        // Uniform scale so the sprite fits inside the viewport, then anchor.
        let sx = vw / sprite_size.0.max( 1.0 );
        let sy = vh / sprite_size.1.max( 1.0 );
        let s = sx.min( sy );
        let scaled = ( sprite_size.0 * s, sprite_size.1 * s );
        let pos = anchor_position( anchor, scaled, ( vw, vh ) );
        Some( make_transform( pos, ( s, s ) ) )
      },
      ViewportTiling::Center =>
      {
        // Native pixel size, positioned per anchor.
        let pos = anchor_position( anchor, sprite_size, ( vw, vh ) );
        Some( make_transform( pos, ( 1.0, 1.0 ) ) )
      },
      ViewportTiling::Repeat2D
      | ViewportTiling::RepeatX
      | ViewportTiling::RepeatY =>
      {
        // Repeat modes emit multiple sprites — the caller uses
        // `tiled_positions` for per-tile offsets. Returning the first tile
        // at origin is convenient for callers that want the common case.
        Some( make_transform( ( 0.0, 0.0 ), ( 1.0, 1.0 ) ) )
      },
    }
  }

  /// Enumerate screen-space positions for each tile in a `Repeat*` viewport
  /// layout.
  ///
  /// Origin is the viewport's top-left; tiles are emitted in row-major
  /// order at native pixel size. For `RepeatX` only the x-axis repeats;
  /// the y-axis is pinned to the anchor point (same for `RepeatY`).
  ///
  /// Returns an empty `Vec` for non-repeating modes — callers use
  /// [`viewport_transform`] directly in that case.
  #[ must_use ]
  pub fn tiled_positions
  (
    tiling : ViewportTiling,
    anchor : ViewportAnchorPoint,
    sprite_size : ( f32, f32 ),
    viewport_size : ( u32, u32 ),
  ) -> Vec< ( f32, f32 ) >
  {
    let vw = viewport_size.0 as f32;
    let vh = viewport_size.1 as f32;
    let sw = sprite_size.0.max( 1.0 );
    let sh = sprite_size.1.max( 1.0 );

    match tiling
    {
      ViewportTiling::Repeat2D =>
      {
        let cols = ( vw / sw ).ceil() as i32 + 1;
        let rows = ( vh / sh ).ceil() as i32 + 1;
        let mut out = Vec::with_capacity( ( cols * rows ) as usize );
        for row in 0..rows
        {
          for col in 0..cols
          {
            out.push( ( col as f32 * sw, row as f32 * sh ) );
          }
        }
        out
      },
      ViewportTiling::RepeatX =>
      {
        // Tile horizontally; y is pinned at the anchor's y position.
        let ( _, y ) = anchor_position( anchor, sprite_size, ( vw, vh ) );
        let cols = ( vw / sw ).ceil() as i32 + 1;
        ( 0..cols ).map( | col | ( col as f32 * sw, y ) ).collect()
      },
      ViewportTiling::RepeatY =>
      {
        // Tile vertically; x is pinned at the anchor's x position.
        let ( x, _ ) = anchor_position( anchor, sprite_size, ( vw, vh ) );
        let rows = ( vh / sh ).ceil() as i32 + 1;
        ( 0..rows ).map( | row | ( x, row as f32 * sh ) ).collect()
      },
      _ => Vec::new(),
    }
  }

  /// Pixel position of a sprite whose natural size is `sprite_size` when
  /// anchored inside a `viewport_size` viewport. Top-left convention —
  /// screen `(0, 0)` is the top-left of the viewport.
  #[ must_use ]
  pub fn anchor_position
  (
    anchor : ViewportAnchorPoint,
    sprite_size : ( f32, f32 ),
    viewport_size : ( f32, f32 ),
  ) -> ( f32, f32 )
  {
    let ( sw, sh ) = sprite_size;
    let ( vw, vh ) = viewport_size;
    match anchor
    {
      ViewportAnchorPoint::TopLeft      => ( 0.0,          0.0 ),
      ViewportAnchorPoint::TopCenter    => ( ( vw - sw ) * 0.5, 0.0 ),
      ViewportAnchorPoint::TopRight     => ( vw - sw,     0.0 ),
      ViewportAnchorPoint::CenterLeft   => ( 0.0,          ( vh - sh ) * 0.5 ),
      ViewportAnchorPoint::Center       => ( ( vw - sw ) * 0.5, ( vh - sh ) * 0.5 ),
      ViewportAnchorPoint::CenterRight  => ( vw - sw,     ( vh - sh ) * 0.5 ),
      ViewportAnchorPoint::BottomLeft   => ( 0.0,          vh - sh ),
      ViewportAnchorPoint::BottomCenter => ( ( vw - sw ) * 0.5, vh - sh ),
      ViewportAnchorPoint::BottomRight  => ( vw - sw,     vh - sh ),
    }
  }

  #[ inline ]
  fn make_transform( pos : ( f32, f32 ), scale : ( f32, f32 ) ) -> Transform
  {
    Transform
    {
      position : [ pos.0, pos.1 ],
      rotation : 0.0,
      scale : [ scale.0, scale.1 ],
      skew : [ 0.0, 0.0 ],
      depth : 0.0,
    }
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::private::*;
  use crate::source::{ ViewportAnchorPoint, ViewportTiling };

  #[ test ]
  fn stretch_fills_viewport()
  {
    let t = viewport_transform(
      ViewportTiling::Stretch,
      ViewportAnchorPoint::Center,
      ( 100.0, 50.0 ),
      ( 800, 600 ),
    ).unwrap();
    assert_eq!( t.position, [ 0.0, 0.0 ] );
    assert!( ( t.scale[ 0 ] - 8.0 ).abs() < 1e-5 );
    assert!( ( t.scale[ 1 ] - 12.0 ).abs() < 1e-5 );
  }

  #[ test ]
  fn center_topleft_is_origin()
  {
    let t = viewport_transform(
      ViewportTiling::Center,
      ViewportAnchorPoint::TopLeft,
      ( 100.0, 50.0 ),
      ( 800, 600 ),
    ).unwrap();
    assert_eq!( t.position, [ 0.0, 0.0 ] );
    assert_eq!( t.scale, [ 1.0, 1.0 ] );
  }

  #[ test ]
  fn center_bottomcenter_positions_sprite()
  {
    let t = viewport_transform(
      ViewportTiling::Center,
      ViewportAnchorPoint::BottomCenter,
      ( 100.0, 50.0 ),
      ( 800, 600 ),
    ).unwrap();
    assert!( ( t.position[ 0 ] - 350.0 ).abs() < 1e-5 ); // (800 - 100) / 2
    assert!( ( t.position[ 1 ] - 550.0 ).abs() < 1e-5 ); // 600 - 50
  }

  #[ test ]
  fn repeat2d_emits_grid_covering_viewport()
  {
    // 32x32 tile, 800x600 viewport → 26 cols × 20 rows (plus safety margin
    // of +1 each side in the implementation).
    let positions = tiled_positions(
      ViewportTiling::Repeat2D,
      ViewportAnchorPoint::TopLeft,
      ( 32.0, 32.0 ),
      ( 800, 600 ),
    );
    assert_eq!( positions.len(), 26 * 20 );
    // First tile at origin.
    assert_eq!( positions[ 0 ], ( 0.0, 0.0 ) );
    // Second tile one sprite-width across.
    assert_eq!( positions[ 1 ], ( 32.0, 0.0 ) );
  }

  #[ test ]
  fn repeatx_emits_single_row()
  {
    let positions = tiled_positions(
      ViewportTiling::RepeatX,
      ViewportAnchorPoint::BottomLeft,
      ( 100.0, 50.0 ),
      ( 800, 600 ),
    );
    assert_eq!( positions.len(), 9 ); // ceil(800/100) + 1 = 9
    // All tiles pinned at y = viewport_h - sprite_h = 550.
    for ( _, y ) in &positions
    {
      assert!( ( y - 550.0 ).abs() < 1e-5 );
    }
  }

  #[ test ]
  fn fit_preserves_aspect()
  {
    // Sprite 100x50, viewport 800x600 → limiting axis is Y (ratio 12 vs 8).
    // Wait — sprite w 100 fits 8x in 800, sprite h 50 fits 12x in 600.
    // min = 8 → scaled sprite = 800 x 400, fits width-first, centred vertically.
    let t = viewport_transform(
      ViewportTiling::Fit,
      ViewportAnchorPoint::Center,
      ( 100.0, 50.0 ),
      ( 800, 600 ),
    ).unwrap();
    assert!( ( t.scale[ 0 ] - 8.0 ).abs() < 1e-5 );
    assert!( ( t.scale[ 1 ] - 8.0 ).abs() < 1e-5 );
    // Centred: scaled sprite is 800x400 → x=0, y=100.
    assert!( ( t.position[ 0 ] - 0.0 ).abs() < 1e-5 );
    assert!( ( t.position[ 1 ] - 100.0 ).abs() < 1e-5 );
  }
}

mod_interface::mod_interface!
{
  exposed use viewport_transform;
  exposed use tiled_positions;
  exposed use anchor_position;
}
