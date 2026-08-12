//! `SvgBackend` adapter tests, relocated from inline `src/adapters/svg.rs` by
//! task 071. Behavior tests drive the backend through its public surface --
//! `SvgBackend::new` / `viewport_scale_set` / `viewport_offset_set` plus the `Backend`
//! trait -- and assert on the rendered SVG string from `output()`. The
//! formatting/encoding helper tests at the bottom exercise the helpers the adapter
//! exports for exactly this placement ( the internal ones marked `#[ doc( hidden ) ]` ),
//! completing the all-tests-in-tests/ convention for this module.

#![ cfg( feature = "adapter-svg" ) ]

use tilemap_renderer::assets::*;
use tilemap_renderer::backend::*;
use tilemap_renderer::commands::*;
use tilemap_renderer::types::*;
use tilemap_renderer::adapters::svg::{ SvgBackend, SvgContentManager };

mod helpers;
use helpers::empty_assets;

fn svg800x600() -> SvgBackend
{
  SvgBackend::new( RenderConfig { width : 800, height : 600, ..Default::default() } )
}

fn render( svg : &SvgBackend ) -> String
{
  match svg.output().unwrap()
  {
    Output::String( s ) => s,
    _ => panic!( "expected string output" ),
  }
}

fn body( svg : &SvgBackend ) -> String
{
  let full = render( svg );
  // The frame body is wrapped in a viewport <g transform="...">...</g>.
  // Return the inner content so tests don't need to know about the wrapper.
  let frame_start = full.find( "<!--framebegin-->" ).unwrap() + "<!--framebegin-->".len();
  let frame_end   = full.find( "<!--frameend-->" ).unwrap();
  let frame = &full[ frame_start..frame_end ];
  // Strip the opening <g ...> tag and trailing </g>
  let inner_start = frame.find( '>' ).map_or( 0, | i | i + 1 );
  let inner_end   = frame.rfind( "</" ).unwrap_or( frame.len() );
  frame[ inner_start..inner_end ].to_string()
}

fn defs( svg : &SvgBackend ) -> String
{
  let full = render( svg );
  let start = full.find( "<defs>" ).unwrap() + "<defs>".len();
  let end = full.find( "</defs>" ).unwrap();
  full[ start..end ].to_string()
}

// -- clear --

#[ test ]
fn clear_emits_rect()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[ RenderCommand::Clear( Clear { color : [ 1.0, 0.0, 0.0, 1.0 ] } ) ] ).unwrap();
  let b = body( &svg );
  assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "body: {b}" );
  assert!( b.contains( "width=\"100%\"" ) );
}

/// Zoom is now applied via the viewport `<g>` wrapper, not per-element.
/// Verify that `viewport_scale_set` updates the wrapper transform.
#[ test ]
fn viewport_zoom_updates_wrapper()
{
  let mut svg = svg800x600();
  svg.viewport_scale_set( 2.0 );
  let full = render( &svg );
  assert!( full.contains( "scale(2)" ), "wrapper: {full}" );
}

/// Viewport offset is now applied via the `<g>` wrapper, not per-element.
/// `viewport_offset_set` should update the wrapper transform attribute.
#[ test ]
fn viewport_offset_updates_wrapper()
{
  let mut svg = svg800x600();
  svg.viewport_offset_set( [ 10.0, 20.0 ] );
  let full = render( &svg );
  // In the wrapper: offset Y is negated (Y-up → SVG Y-down flip)
  assert!( full.contains( "translate(10,-20)" ), "wrapper: {full}" );
}

// -- path --

#[ test ]
fn path_emits_svg_path()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::Solid( [ 0.0, 0.0, 1.0, 1.0 ] ),
      stroke_color : [ 1.0, 1.0, 1.0, 1.0 ],
      stroke_width : 2.0,
      stroke_cap : LineCap::Round,
      stroke_join : LineJoin::Round,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 10.0, 20.0 ) ),
    RenderCommand::LineTo( LineTo( 100.0, 200.0 ) ),
    RenderCommand::ClosePath( ClosePath ),
    RenderCommand::EndPath( EndPath ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "<path" ), "body: {b}" );
  assert!( b.contains( "M 10 20" ), "body: {b}" );
  assert!( b.contains( "L 100 200" ), "body: {b}" );
  assert!( b.contains( 'Z' ), "body: {b}" );
  assert!( b.contains( "fill=\"rgb(0,0,255)\"" ), "body: {b}" );
  assert!( b.contains( "stroke-linecap=\"round\"" ), "body: {b}" );
  assert!( b.contains( "stroke-linejoin=\"round\"" ), "body: {b}" );
}

#[ test ]
fn path_emits_quad_cubic_arc()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::Solid( [ 0.0, 0.0, 0.0, 1.0 ] ),
      stroke_color : [ 0.0, 0.0, 0.0, 1.0 ],
      stroke_width : 1.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Normal,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 0.0, 0.0 ) ),
    RenderCommand::QuadTo( QuadTo { cx : 10.0, cy : 20.0, x : 30.0, y : 40.0 } ),
    RenderCommand::CubicTo( CubicTo { c1x : 50.0, c1y : 60.0, c2x : 70.0, c2y : 80.0, x : 90.0, y : 100.0 } ),
    // rotation is radians in the command; SVG A takes degrees —
    // FRAC_PI_2 (~1.5708 rad) should serialize as 90.
    RenderCommand::ArcTo( ArcTo
    {
      rx : 5.0, ry : 6.0, rotation : core::f32::consts::FRAC_PI_2,
      large_arc : true, sweep : false, x : 110.0, y : 120.0,
    }),
    RenderCommand::EndPath( EndPath ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "Q 10 20 30 40" ), "body: {b}" );
  assert!( b.contains( "C 50 60 70 80 90 100" ), "body: {b}" );
  // Arc flags serialize as 1 / 0 integers.
  assert!( b.contains( "A 5 6 90 1 0 110 120" ), "body: {b}" );
}

// -- image loading viewBox --

#[ test ]
fn image_viewbox_origin_zero()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap { bytes : vec![ 0u8; 64 * 32 * 4 ], width : 64, height : 32, format : PixelFormat::Rgba8 },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  let d = defs( &svg );
  // Should use "0 0 w h" viewBox, not center-origin
  assert!( d.contains( "viewBox=\"0 0 64 32\"" ), "defs: {d}" );
  // Should not have negative offsets
  assert!( !d.contains( "x=\"-" ), "defs: {d}" );
  assert!( !d.contains( "y=\"-" ), "defs: {d}" );
}

// -- gradients --

#[ test ]
fn linear_gradient_emits_userspace_coords_and_stops()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    gradients : vec![ GradientAsset
    {
      id : ResourceId::new( 0 ),
      kind : GradientKind::Linear { start : [ 10.0, 20.0 ], end : [ 100.0, 200.0 ] },
      stops : vec!
      [
        GradientStop { offset : 0.0, color : [ 1.0, 0.0, 0.0, 1.0 ] },
        GradientStop { offset : 1.0, color : [ 0.0, 0.0, 1.0, 0.5 ] },
      ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  let d = defs( &svg );
  assert!( d.contains( "<linearGradient id=\"grad_0\"" ), "defs: {d}" );
  // Regression guard for the userSpaceOnUse fix (commit ab4699e3) —
  // without this, SVG reinterprets pixel coords as 0..1 bbox fractions.
  assert!( d.contains( "gradientUnits=\"userSpaceOnUse\"" ), "defs: {d}" );
  assert!
  (
    d.contains( "x1=\"10\"" ) && d.contains( "y1=\"20\"" )
      && d.contains( "x2=\"100\"" ) && d.contains( "y2=\"200\"" ),
    "defs: {d}"
  );
  assert!( d.contains( "offset=\"0\"" ) && d.contains( "offset=\"1\"" ), "defs: {d}" );
  assert!( d.contains( "stop-color=\"rgb(255,0,0)\"" ), "defs: {d}" );
  assert!( d.contains( "stop-color=\"rgb(0,0,255)\"" ), "defs: {d}" );
  // Alpha != 1 should emit stop-opacity on that stop.
  assert!( d.contains( "stop-opacity=" ), "defs: {d}" );
  assert!( d.contains( "</linearGradient>" ), "defs: {d}" );
}

#[ test ]
fn radial_gradient_emits_center_radius_focal()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    gradients : vec![ GradientAsset
    {
      id : ResourceId::new( 7 ),
      kind : GradientKind::Radial
      {
        center : [ 50.0, 60.0 ],
        radius : 40.0,
        focal  : [ 55.0, 65.0 ],
      },
      stops : vec!
      [
        GradientStop { offset : 0.0, color : [ 1.0, 1.0, 1.0, 1.0 ] },
        GradientStop { offset : 1.0, color : [ 0.0, 0.0, 0.0, 1.0 ] },
      ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  let d = defs( &svg );
  assert!( d.contains( "<radialGradient id=\"grad_7\"" ), "defs: {d}" );
  assert!( d.contains( "gradientUnits=\"userSpaceOnUse\"" ), "defs: {d}" );
  assert!
  (
    d.contains( "cx=\"50\"" ) && d.contains( "cy=\"60\"" ) && d.contains( "r=\"40\"" )
      && d.contains( "fx=\"55\"" ) && d.contains( "fy=\"65\"" ),
    "defs: {d}"
  );
  assert!( d.contains( "</radialGradient>" ), "defs: {d}" );
}

// -- patterns --

#[ test ]
fn pattern_emits_userspace_tile_size_and_image_ref()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 3 ),
      source : ImageSource::Bitmap { bytes : vec![ 0u8; 32 * 32 * 4 ], width : 32, height : 32, format : PixelFormat::Rgba8 },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    patterns : vec![ PatternAsset
    {
      id : ResourceId::new( 9 ),
      content : ResourceId::new( 3 ),
      width : 32.0,
      height : 16.0,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  let d = defs( &svg );
  assert!( d.contains( "<pattern id=\"pat_9\"" ), "defs: {d}" );
  assert!( d.contains( "width=\"32\"" ) && d.contains( "height=\"16\"" ), "defs: {d}" );
  // userSpaceOnUse keeps the tile at its declared pixel size rather than
  // scaling to a 0..1 fraction of the filled element's bbox.
  assert!( d.contains( "patternUnits=\"userSpaceOnUse\"" ), "defs: {d}" );
  assert!( d.contains( "href=\"#img_3\"" ), "defs: {d}" );
  assert!( d.contains( "</pattern>" ), "defs: {d}" );
}

// -- clip masks --

#[ test ]
fn clip_mask_emits_clip_path_with_path_segments()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    clip_masks : vec![ ClipMaskAsset
    {
      id : ResourceId::new( 4 ),
      segments : vec!
      [
        PathSegment::MoveTo( 10.0, 20.0 ),
        PathSegment::LineTo( 30.0, 20.0 ),
        PathSegment::LineTo( 30.0, 40.0 ),
        PathSegment::Close,
      ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  let d = defs( &svg );
  assert!( d.contains( "<clipPath id=\"clip_4\">" ), "defs: {d}" );
  // Segments should be joined into one d= attribute in emission order.
  assert!
  (
    d.contains( "d=\"M 10 20 L 30 20 L 30 40 Z\"" ),
    "defs: {d}"
  );
  assert!( d.contains( "</clipPath>" ), "defs: {d}" );
}

// -- sprite tint --

#[ test ]
fn sprite_white_tint_no_filter()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 16, height : 16, format : PixelFormat::Rgba8 },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    sprites : vec![ SpriteAsset
    {
      id : ResourceId::new( 0 ),
      sheet : ResourceId::new( 0 ),
      region : [ 0.0, 0.0, 16.0, 16.0 ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  svg.submit( &[
    RenderCommand::Sprite( Sprite
    {
      transform : Transform::default(),
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();

  let b = body( &svg );
  assert!( !b.contains( "filter=" ), "white tint should not create filter, body: {b}" );
}

/// Closes Polish §6 in `tilemap_scene/roadmap.md`: `ScreenSpaceSprite`
/// shares the `Sprite` payload, and SVG's user-space is already
/// screen-space, so the adapter dispatches both through the same
/// `cmd_sprite` path.
#[ test ]
fn screen_space_sprite_renders_through_sprite_path()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 16, height : 16, format : PixelFormat::Rgba8 },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    sprites : vec![ SpriteAsset
    {
      id : ResourceId::new( 0 ),
      sheet : ResourceId::new( 0 ),
      region : [ 0.0, 0.0, 16.0, 16.0 ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  svg.submit( &[
    RenderCommand::ScreenSpaceSprite( Sprite
    {
      transform : Transform::default(),
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();

  let b = body( &svg );
  assert!
  (
    b.contains( "<use href=\"#sprite_0\"" ),
    "ScreenSpaceSprite must render via the same <use> path as Sprite; body: {b}",
  );
}

#[ test ]
fn sprite_colored_tint_creates_filter()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 16, height : 16, format : PixelFormat::Rgba8 },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    sprites : vec![ SpriteAsset
    {
      id : ResourceId::new( 0 ),
      sheet : ResourceId::new( 0 ),
      region : [ 0.0, 0.0, 16.0, 16.0 ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  svg.submit( &[
    RenderCommand::Sprite( Sprite
    {
      transform : Transform::default(),
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 0.0, 0.0, 1.0 ],
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();

  let b = body( &svg );
  let d = defs( &svg );
  assert!( b.contains( "filter=\"url(#tint_0)\"" ), "body: {b}" );
  assert!( d.contains( "<filter id=\"tint_0\">" ), "defs: {d}" );
  assert!( d.contains( "feColorMatrix" ), "defs: {d}" );
}

#[ test ]
fn two_tinted_sprites_get_distinct_filter_ids()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 16, height : 16, format : PixelFormat::Rgba8 },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    sprites : vec![ SpriteAsset
    {
      id : ResourceId::new( 0 ),
      sheet : ResourceId::new( 0 ),
      region : [ 0.0, 0.0, 16.0, 16.0 ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  let s = Sprite
  {
    transform : Transform::default(),
    sprite : ResourceId::new( 0 ),
    tint : [ 1.0, 0.0, 0.0, 1.0 ],
    blend : BlendMode::Normal,
    clip : None,
  };
  svg.submit( &[ RenderCommand::Sprite( s ), RenderCommand::Sprite( Sprite { tint : [ 0.0, 1.0, 0.0, 1.0 ], ..s } ) ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "url(#tint_0)" ), "body: {b}" );
  assert!( b.contains( "url(#tint_1)" ), "body: {b}" );
}

// -- batch lifecycle --

#[ test ]
fn sprite_batch_create_draw()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap { bytes : vec![ 0u8; 4 ], width : 32, height : 32, format : PixelFormat::Rgba8 },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    sprites : vec![ SpriteAsset
    {
      id : ResourceId::new( 0 ),
      sheet : ResourceId::new( 0 ),
      region : [ 0.0, 0.0, 32.0, 32.0 ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  let batch_id : ResourceId< Batch > = ResourceId::new( 0 );
  svg.submit( &[
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch
    {
      batch : batch_id,
      params : SpriteBatchParams
      {
        transform : Transform::default(),
        sheet : ResourceId::new( 0 ),
        blend : BlendMode::Normal,
        clip : None,
      },
    }),
    RenderCommand::BindBatch( BindBatch { batch : batch_id } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance
    {
      transform : Transform { position : [ 10.0, 20.0 ], ..Default::default() },
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
    }),
    RenderCommand::AddSpriteInstance( AddSpriteInstance
    {
      transform : Transform { position : [ 50.0, 60.0 ], ..Default::default() },
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
    }),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch : batch_id } ),
  ]).unwrap();

  let b = body( &svg );
  // Should have a group wrapper
  assert!( b.contains( "<g" ), "body: {b}" );
  assert!( b.contains( "</g>" ), "body: {b}" );
  // Should have two sprite instances with local transforms
  assert_eq!( b.matches( "#sprite_0" ).count(), 2, "body: {b}" );
  // Local transforms should use raw positions (no Y-flip)
  assert!( b.contains( "translate(10,20)" ), "body: {b}" );
  assert!( b.contains( "translate(50,60)" ), "body: {b}" );
}

// -- mesh batch --

#[ test ]
fn mesh_batch_create_draw()
{
  let mut svg = svg800x600();
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
  let assets = Assets
  {
    geometries : vec![ GeometryAsset
    {
      id : ResourceId::new( 0 ),
      positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
      uvs : None,
      indices : None,
      data_type : DataType::U16,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  let batch_id : ResourceId< Batch > = ResourceId::new( 0 );
  svg.submit( &[
    RenderCommand::CreateMeshBatch( CreateMeshBatch
    {
      batch : batch_id,
      params : MeshBatchParams
      {
        transform : Transform::default(),
        geometry : ResourceId::new( 0 ),
        fill : FillRef::Solid( [ 0.0, 1.0, 0.0, 1.0 ] ),
        texture : None,
        topology : Topology::TriangleList,
        blend : BlendMode::Normal,
        clip : None,
      },
    }),
    RenderCommand::BindBatch( BindBatch { batch : batch_id } ),
    RenderCommand::AddMeshInstance( AddMeshInstance
    {
      transform : Transform { position : [ 5.0, 10.0 ], ..Default::default() },
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
    }),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch : batch_id } ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "<g" ), "body: {b}" );
  assert!( b.contains( "fill=\"rgb(0,255,0)\"" ), "body: {b}" );
  assert!( b.contains( "translate(5,10)" ), "body: {b}" );
}

// -- batch instance update and remove --

#[ test ]
fn batch_set_and_remove_instance()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();

  let batch_id : ResourceId< Batch > = ResourceId::new( 0 );
  // First submit: create batch with 2 instances
  svg.submit( &[
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch
    {
      batch : batch_id,
      params : SpriteBatchParams
      {
        transform : Transform::default(),
        sheet : ResourceId::new( 0 ),
        blend : BlendMode::Normal,
        clip : None,
      },
    }),
    RenderCommand::BindBatch( BindBatch { batch : batch_id } ),
    RenderCommand::AddSpriteInstance( AddSpriteInstance
    {
      transform : Transform { position : [ 1.0, 2.0 ], ..Default::default() },
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
    }),
    RenderCommand::AddSpriteInstance( AddSpriteInstance
    {
      transform : Transform { position : [ 3.0, 4.0 ], ..Default::default() },
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
    }),
    RenderCommand::UnbindBatch( UnbindBatch ),
  ]).unwrap();

  // Second submit: remove first instance, draw
  svg.submit( &[
    RenderCommand::BindBatch( BindBatch { batch : batch_id } ),
    RenderCommand::RemoveInstance( RemoveInstance { index : 0 } ),
    RenderCommand::UnbindBatch( UnbindBatch ),
    RenderCommand::DrawBatch( DrawBatch { batch : batch_id } ),
  ]).unwrap();

  let b = body( &svg );
  // Should have only 1 instance (the one at 3,4)
  assert_eq!( b.matches( "#sprite_0" ).count(), 1, "body: {b}" );
  assert!( b.contains( "translate(3,4)" ), "body: {b}" );
  assert!( !b.contains( "translate(1,2)" ), "body: {b}" );
}

// -- delete batch --

#[ test ]
fn delete_batch()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();

  let batch_id : ResourceId< Batch > = ResourceId::new( 0 );
  svg.submit( &[
    RenderCommand::CreateSpriteBatch( CreateSpriteBatch
    {
      batch : batch_id,
      params : SpriteBatchParams
      {
        transform : Transform::default(),
        sheet : ResourceId::new( 0 ),
        blend : BlendMode::Normal,
        clip : None,
      },
    }),
    RenderCommand::DeleteBatch( DeleteBatch { batch : batch_id } ),
    RenderCommand::DrawBatch( DrawBatch { batch : batch_id } ),
  ]).unwrap();

  let b = body( &svg );
  // Draw after delete should produce nothing
  assert!( !b.contains( "<g" ), "body: {b}" );
}

// -- effects --

#[ test ]
fn effect_blur()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    RenderCommand::BeginGroup( BeginGroup
    {
      transform : Transform::default(),
      clip : None,
      effect : Some( Effect::Blur { radius : 5.0 } ),
    }),
    RenderCommand::EndGroup( EndGroup ),
  ]).unwrap();

  let b = body( &svg );
  let d = defs( &svg );
  assert!( b.contains( "filter=\"url(#fx_0)\"" ), "body: {b}" );
  assert!( d.contains( "feGaussianBlur" ), "defs: {d}" );
  assert!( d.contains( "stdDeviation=\"5\"" ), "defs: {d}" );
}

#[ test ]
fn effect_drop_shadow_y_flipped()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    RenderCommand::BeginGroup( BeginGroup
    {
      transform : Transform::default(),
      clip : None,
      effect : Some( Effect::DropShadow { dx : 2.0, dy : 3.0, blur : 4.0, color : [ 0.0, 0.0, 0.0, 0.5 ] } ),
    }),
    RenderCommand::EndGroup( EndGroup ),
  ]).unwrap();

  let d = defs( &svg );
  // SVG 1.1 composite drop shadow: blur -> offset -> flood+composite -> merge
  assert!( !d.contains( "feDropShadow" ), "feDropShadow is SVG 2, should be lowered: {d}" );
  assert!( d.contains( "feGaussianBlur" ), "defs: {d}" );
  assert!( d.contains( "stdDeviation=\"4\"" ), "defs: {d}" );
  assert!( d.contains( "<feOffset" ), "defs: {d}" );
  assert!( d.contains( "dx=\"2\"" ), "defs: {d}" );
  // dy should be negated: 3.0 → -3.0
  assert!( d.contains( "dy=\"-3\"" ), "defs: {d}" );
  assert!( d.contains( "<feFlood" ), "defs: {d}" );
  assert!( d.contains( "<feComposite" ), "defs: {d}" );
  assert!( d.contains( "operator=\"in\"" ), "defs: {d}" );
  assert!( d.contains( "<feMerge>" ), "defs: {d}" );
}

#[ test ]
fn effect_color_matrix()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  let mut values = [ 0.0f32; 20 ];
  values[ 0 ] = 1.0; // r->r
  values[ 6 ] = 1.0; // g->g
  values[ 12 ] = 1.0; // b->b
  values[ 18 ] = 1.0; // a->a
  svg.submit( &[
    RenderCommand::BeginGroup( BeginGroup
    {
      transform : Transform::default(),
      clip : None,
      effect : Some( Effect::ColorMatrix( values ) ),
    }),
    RenderCommand::EndGroup( EndGroup ),
  ]).unwrap();

  let d = defs( &svg );
  assert!( d.contains( "feColorMatrix" ), "defs: {d}" );
  assert!( d.contains( "type=\"matrix\"" ), "defs: {d}" );
}

#[ test ]
fn effect_opacity()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    RenderCommand::BeginGroup( BeginGroup
    {
      transform : Transform::default(),
      clip : None,
      effect : Some( Effect::Opacity( 0.5 ) ),
    }),
    RenderCommand::EndGroup( EndGroup ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "opacity=\"0.5\"" ), "body: {b}" );
}

// -- groups --

#[ test ]
fn nested_groups()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    RenderCommand::BeginGroup( BeginGroup { transform : Transform::default(), clip : None, effect : None } ),
    RenderCommand::BeginGroup( BeginGroup { transform : Transform::default(), clip : None, effect : None } ),
    RenderCommand::EndGroup( EndGroup ),
    RenderCommand::EndGroup( EndGroup ),
  ]).unwrap();

  let b = body( &svg );
  assert_eq!( b.matches( "<g" ).count(), 2 );
  assert_eq!( b.matches( "</g>" ).count(), 2 );
}

#[ test ]
fn unmatched_end_group_does_not_emit_closing_tag()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[ RenderCommand::EndGroup( EndGroup ) ]).unwrap();

  let b = body( &svg );
  assert_eq!( b.matches( "</g>" ).count(), 0, "unmatched EndGroup should not emit </g>: {b}" );
}

// -- geometry mesh --

#[ test ]
fn mesh_triangle_list()
{
  let mut svg = svg800x600();
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
  let assets = Assets
  {
    geometries : vec![ GeometryAsset
    {
      id : ResourceId::new( 0 ),
      positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
      uvs : None,
      indices : None,
      data_type : DataType::U16,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  svg.submit( &[
    RenderCommand::Mesh( Mesh
    {
      transform : Transform::default(),
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
      texture : None,
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();

  let b = body( &svg );
  let d = defs( &svg );
  assert!( d.contains( "<polygon" ), "defs: {d}" );
  assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "body: {b}" );
}

/// Verifies that `DataType::U8` index buffers are correctly loaded and used
/// so geometry with U8 indices renders polygons rather than being silently dropped.
#[ test ]
fn geometry_u8_indices_loaded()
{
  let mut svg = svg800x600();
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
  let indices_u8 : &[ u8 ] = &[ 0, 1, 2 ];
  let assets = Assets
  {
    geometries : vec![ GeometryAsset
    {
      id : ResourceId::new( 0 ),
      positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
      uvs : None,
      indices : Some( Source::Bytes( indices_u8.to_vec() ) ),
      data_type : DataType::U8,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  svg.submit( &[
    RenderCommand::Mesh( Mesh
    {
      transform : Transform::default(),
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
      texture : None,
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();

  let d = defs( &svg );
  assert!( d.contains( "<polygon" ), "U8 indices not used — polygon missing from defs: {d}" );
}

/// Verifies that out-of-bounds indices in geometry do not cause a panic.
/// The out-of-range polygon is silently skipped; valid polygons still render.
#[ test ]
fn geometry_oob_index_no_panic()
{
  let mut svg = svg800x600();
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ]; // 3 vertices
  // Triangle 0: valid (0,1,2). Triangle 1: index 99 is out of bounds.
  let indices : Vec< u32 > = vec![ 0, 1, 2, 0, 1, 99 ];
  let assets = Assets
  {
    geometries : vec![ GeometryAsset
    {
      id : ResourceId::new( 0 ),
      positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
      uvs : None,
      indices : Some( Source::Bytes( bytemuck::cast_slice( &indices ).to_vec() ) ),
      data_type : DataType::U32,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  // Must not panic
  svg.submit( &[
    RenderCommand::Mesh( Mesh
    {
      transform : Transform::default(),
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
      texture : None,
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();

  let d = defs( &svg );
  // The valid first triangle should still appear
  assert!( d.contains( "<polygon" ), "valid polygon missing from defs: {d}" );
}

fn mesh_svg( topology : Topology, positions : &[ f32 ] ) -> ( String, String )
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    geometries : vec![ GeometryAsset
    {
      id : ResourceId::new( 0 ),
      positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
      uvs : None,
      indices : None,
      data_type : DataType::U16,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  svg.submit( &[
    RenderCommand::Mesh( Mesh
    {
      transform : Transform::default(),
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( [ 1.0, 1.0, 1.0, 1.0 ] ),
      texture : None,
      topology,
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();
  ( body( &svg ), defs( &svg ) )
}

/// `TriangleStrip` with 4 vertices produces 2 triangles (n − 2 = 2 polygons).
#[ test ]
fn mesh_triangle_strip_polygon_count()
{
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 0.0, 100.0, 100.0, 100.0 ];
  let ( _b, d ) = mesh_svg( Topology::TriangleStrip, positions );
  assert_eq!( d.matches( "<polygon" ).count(), 2, "defs: {d}" );
}

/// `TriangleStrip` with exactly 3 vertices produces exactly 1 triangle.
#[ test ]
fn mesh_triangle_strip_min_count()
{
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
  let ( _b, d ) = mesh_svg( Topology::TriangleStrip, positions );
  assert_eq!( d.matches( "<polygon" ).count(), 1, "defs: {d}" );
}

/// `TriangleStrip` alternates winding on odd triangles — for strip v0..v3,
/// triangle 0 is (v0,v1,v2) and triangle 1 is (v2,v1,v3), preserving CCW order
/// (swapping the first two would flip winding; the second triangle in a raw
/// strip is (v1,v2,v3) which has opposite winding from (v0,v1,v2)).
#[ test ]
fn mesh_triangle_strip_alternates_winding()
{
  // Four distinct vertices so we can identify the emitted order.
  let positions : &[ f32 ] = &[ 0.0, 0.0, 10.0, 0.0, 0.0, 10.0, 10.0, 10.0 ];
  let ( _b, d ) = mesh_svg( Topology::TriangleStrip, positions );
  // First triangle: v0,v1,v2 => "0,0 10,0 0,10"
  assert!( d.contains( "points=\"0,0 10,0 0,10\"" ), "first tri wrong: {d}" );
  // Second triangle: order swapped to v2,v1,v3 => "0,10 10,0 10,10"
  assert!( d.contains( "points=\"0,10 10,0 10,10\"" ), "second tri winding not alternated: {d}" );
  // Raw (un-alternated) order would have been v1,v2,v3 => "10,0 0,10 10,10" — ensure it's absent.
  assert!( !d.contains( "points=\"10,0 0,10 10,10\"" ), "strip emitted raw order: {d}" );
}

/// `TriangleStrip` with fewer than 3 vertices produces no geometry — degenerate input is silently skipped.
#[ test ]
fn mesh_triangle_strip_degenerate_no_output()
{
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0 ]; // 2 vertices
  let ( b, _d ) = mesh_svg( Topology::TriangleStrip, positions );
  // No <use> in body — the mesh def was not created
  assert!( !b.contains( "<use" ), "body: {b}" );
}

/// `LineList` with 4 vertices (2 pairs) produces 2 `<polyline>` elements.
#[ test ]
fn mesh_line_list_even()
{
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 100.0, 200.0, 0.0, 300.0, 100.0 ];
  let ( _b, d ) = mesh_svg( Topology::LineList, positions );
  assert_eq!( d.matches( "<polyline" ).count(), 2, "defs: {d}" );
}

/// `LineList` with 3 vertices (odd) emits only 1 `<polyline>` — the trailing vertex is ignored.
#[ test ]
fn mesh_line_list_odd_vertex_count()
{
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 100.0, 200.0, 0.0 ];
  let ( _b, d ) = mesh_svg( Topology::LineList, positions );
  assert_eq!( d.matches( "<polyline" ).count(), 1, "defs: {d}" );
}

/// `LineStrip` with 4 vertices produces a single `<polyline>` with all points.
#[ test ]
fn mesh_line_strip_single_polyline()
{
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 100.0, 100.0, 0.0, 100.0 ];
  let ( _b, d ) = mesh_svg( Topology::LineStrip, positions );
  assert_eq!( d.matches( "<polyline" ).count(), 1, "defs: {d}" );
}

/// Line meshes inherit stroke color from the <use>, not from
/// `currentColor` (which would resolve to black regardless of fill).
#[ test ]
fn mesh_line_stroke_cascades_from_use()
{
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0 ];
  let ( b, d ) = mesh_svg( Topology::LineList, positions );
  // Polyline in the symbol does NOT set stroke (it would block cascade).
  assert!( !d.contains( "stroke=\"currentColor\"" ), "defs: {d}" );
  // <use> in the body carries stroke equal to fill so it cascades.
  assert!( b.contains( "stroke=\"rgb(255,255,255)\"" ), "body: {b}" );
}

// -- resize --

#[ test ]
fn resize_updates_viewbox()
{
  let mut svg = svg800x600();
  svg.resize( 1024, 768 );
  let full = render( &svg );
  assert!( full.contains( "width=\"1024\"" ), "full: {full}" );
  assert!( full.contains( "height=\"768\"" ), "full: {full}" );
  assert!( full.contains( "viewBox=\"0 0 1024 768\"" ), "full: {full}" );
}

// -- capabilities --

#[ test ]
fn capabilities_all_true()
{
  let svg = svg800x600();
  let caps = svg.capabilities();
  assert!( caps.paths );
  assert!( caps.text );
  assert!( caps.meshes );
  assert!( caps.sprites );
  assert!( caps.batches );
  assert!( caps.gradients );
  assert!( caps.patterns );
  assert!( caps.clip_masks );
  assert!( caps.effects );
  assert!( caps.blend_modes );
  assert!( caps.text_on_path );
  assert_eq!( caps.max_texture_size, 0 );
}

// -- blend modes --

#[ test ]
fn blend_mode_multiply()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    RenderCommand::BeginPath( BeginPath
    {
      transform : Transform::default(),
      fill : FillRef::Solid( [ 1.0, 1.0, 1.0, 1.0 ] ),
      stroke_color : [ 0.0, 0.0, 0.0, 0.0 ],
      stroke_width : 0.0,
      stroke_cap : LineCap::Butt,
      stroke_join : LineJoin::Miter,
      stroke_dash : DashStyle::default(),
      blend : BlendMode::Multiply,
      clip : None,
    }),
    RenderCommand::MoveTo( MoveTo( 0.0, 0.0 ) ),
    RenderCommand::EndPath( EndPath ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "mix-blend-mode:multiply" ), "body: {b}" );
}

/// Verifies that an `ImageSource::Path` containing XML-special characters
/// cannot break out of the href attribute and inject event handlers like
/// onload="alert(1)". Filenames with double-quotes are legal on Linux.
#[ test ]
fn image_path_escapes_attribute_injection()
{
  let mut svg = svg800x600();
  let malicious = r#"foo" onload="alert(1)"#;
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Path( std::path::PathBuf::from( malicious ) ),
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  let d = defs( &svg );
  // Raw unescaped injection must not appear.
  assert!( !d.contains( r#"onload="alert(1)""# ), "event handler leaked: {d}" );
  // Percent-encoded form must appear (quote => %22).
  assert!( d.contains( "%22" ), "expected percent-encoded quote in href: {d}" );
}

/// Verifies that a sprite referencing an `ImageSource::Path` sheet
/// (which has unknown dimensions) is skipped and a diagnostic HTML
/// comment is emitted instead of producing an invisible sprite symbol.
#[ test ]
fn sprite_on_path_sheet_is_skipped_with_comment()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Path( "does_not_matter.png".into() ),
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    sprites : vec![ SpriteAsset
    {
      id : ResourceId::new( 7 ),
      sheet : ResourceId::new( 0 ),
      region : [ 0.0, 0.0, 4.0, 4.0 ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  let d = defs( &svg );
  // No sprite_7 symbol was emitted.
  assert!( !d.contains( "id=\"sprite_7\"" ), "sprite should be skipped: {d}" );
  // A diagnostic comment was emitted instead.
  assert!( d.contains( "sprite_7 skipped" ), "diagnostic comment missing: {d}" );
}

/// Verifies that a geometry whose positions and indices both come from
/// `Source::Path` files is loaded from disk and renders exactly like a
/// `Source::Bytes` geometry.
#[ test ]
fn geometry_path_source_loads_from_disk()
{
  let dir = std::env::temp_dir();
  let positions_path = dir.join( "-tilemap_renderer_svg_test_geometry_positions.bin" );
  let indices_path = dir.join( "-tilemap_renderer_svg_test_geometry_indices.bin" );
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
  let indices : &[ u16 ] = &[ 0, 1, 2 ];
  std::fs::write( &positions_path, bytemuck::cast_slice( positions ) ).unwrap();
  std::fs::write( &indices_path, bytemuck::cast_slice( indices ) ).unwrap();

  let mut svg = svg800x600();
  let assets = Assets
  {
    geometries : vec![ GeometryAsset
    {
      id : ResourceId::new( 0 ),
      positions : Source::Path( positions_path.clone() ),
      uvs : None,
      indices : Some( Source::Path( indices_path.clone() ) ),
      data_type : DataType::U16,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  svg.submit( &[
    RenderCommand::Mesh( Mesh
    {
      transform : Transform::default(),
      geometry : ResourceId::new( 0 ),
      fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
      texture : None,
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();

  std::fs::remove_file( &positions_path ).ok();
  std::fs::remove_file( &indices_path ).ok();

  let b = body( &svg );
  let d = defs( &svg );
  assert!( d.contains( "<polygon" ), "mesh def missing — geometry did not load from disk: {d}" );
  assert!( !d.contains( "geometry_0 skipped" ), "geometry was wrongly skipped: {d}" );
  assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "body: {b}" );
}

/// Verifies that a geometry whose `Source::Path` cannot be read is skipped
/// loudly — a diagnostic HTML comment in the defs — and that a mesh
/// referencing it is absent rather than the whole submit failing.
#[ test ]
fn geometry_on_missing_path_is_skipped_with_comment()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    geometries : vec![ GeometryAsset
    {
      id : ResourceId::new( 3 ),
      positions : Source::Path( "/nonexistent/-tilemap_renderer_svg_test_missing.bin".into() ),
      uvs : None,
      indices : None,
      data_type : DataType::U16,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();

  svg.submit( &[
    RenderCommand::Mesh( Mesh
    {
      transform : Transform::default(),
      geometry : ResourceId::new( 3 ),
      fill : FillRef::Solid( [ 1.0, 0.0, 0.0, 1.0 ] ),
      texture : None,
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    }),
  ]).unwrap();

  let d = defs( &svg );
  assert!( d.contains( "geometry_3 skipped" ), "diagnostic comment missing: {d}" );
  assert!( !d.contains( "<polygon" ), "no mesh def should exist for a skipped geometry: {d}" );
}

/// Verifies that a geometry whose index `Source::Path` cannot be read is
/// skipped whole — partial fallback to unindexed drawing would silently
/// render different topology.
#[ test ]
fn geometry_on_missing_index_path_is_skipped_whole()
{
  let positions : &[ f32 ] = &[ 0.0, 0.0, 100.0, 0.0, 50.0, 100.0 ];
  let mut svg = svg800x600();
  let assets = Assets
  {
    geometries : vec![ GeometryAsset
    {
      id : ResourceId::new( 4 ),
      positions : Source::Bytes( bytemuck::cast_slice( positions ).to_vec() ),
      uvs : None,
      indices : Some( Source::Path( "/nonexistent/-tilemap_renderer_svg_test_missing_indices.bin".into() ) ),
      data_type : DataType::U16,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  let d = defs( &svg );
  assert!( d.contains( "geometry_4 skipped" ), "diagnostic comment missing: {d}" );
}

/// Verifies that JPEG-encoded bytes produce a `data:image/jpeg` URI.
#[ test ]
fn image_encoded_jpeg_emits_jpeg_mime()
{
  let jpeg_bytes : Vec< u8 > = vec![ 0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46 ];
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Encoded( jpeg_bytes ),
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  let d = defs( &svg );
  assert!( d.contains( "data:image/jpeg;base64," ), "defs: {d}" );
  assert!( !d.contains( "data:image/png;base64," ), "should not emit PNG mime: {d}" );
}

/// End-to-end: load a 2×2 Rgba8 Bitmap image asset and verify that `<defs>`
/// contains a `data:image/png;base64,` URI — the full encode path ran.
#[ test ]
fn image_bitmap_emits_png_data_uri()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap
      {
        bytes : vec![ 255u8; 2 * 2 * 4 ],
        width : 2,
        height : 2,
        format : PixelFormat::Rgba8,
      },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  let d = defs( &svg );
  assert!( d.contains( "data:image/png;base64," ), "defs: {d}" );
}

/// When the byte buffer is too small for the declared dimensions,
/// `bitmap_to_png` returns None and no image def is emitted.
#[ test ]
fn image_bitmap_bad_dimensions_emits_nothing()
{
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap
      {
        bytes : vec![ 255u8; 4 ], // too small for 4×4
        width : 4,
        height : 4,
        format : PixelFormat::Rgba8,
      },
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  let d = defs( &svg );
  assert!( !d.contains( "data:image/png;base64," ), "expected no image def, defs: {d}" );
}

fn begin_text_cmd( anchor : TextAnchor, position : [ f32; 2 ] ) -> RenderCommand
{
  RenderCommand::BeginText( BeginText
  {
    font : ResourceId::new( 0 ),
    size : 16.0,
    color : [ 1.0, 1.0, 1.0, 1.0 ],
    anchor,
    position,
    along_path : None,
    clip : None,
  })
}

/// Verifies that `BeginText` / `Char` / `EndText` produces a `<text>` element
/// containing the submitted characters.
#[ test ]
fn text_basic_flow_emits_text_element()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    begin_text_cmd( TextAnchor::TopLeft, [ 10.0, 20.0 ] ),
    RenderCommand::Char( Char( 'H' ) ),
    RenderCommand::Char( Char( 'i' ) ),
    RenderCommand::EndText( EndText ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "<text" ), "body: {b}" );
  assert!( b.contains( "Hi" ), "body: {b}" );
}

/// Verifies SVG 1.1 conformance: translucent colors emit `rgb()` plus a
/// separate `*-opacity` attribute, never the CSS-Color-Level-4 `rgba()`
/// notation (which Inkscape / strict SVG parsers may reject).
#[ test ]
fn color_emits_svg11_rgb_plus_opacity_not_rgba()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[ RenderCommand::Clear( Clear { color : [ 1.0, 0.0, 0.0, 0.5 ] } ) ] ).unwrap();
  let b = body( &svg );
  assert!( !b.contains( "rgba(" ), "rgba() notation leaked (not SVG 1.1): {b}" );
  assert!( b.contains( "fill=\"rgb(255,0,0)\"" ), "expected rgb() fill: {b}" );
  assert!( b.contains( "fill-opacity=\"0.5\"" ), "expected fill-opacity attr: {b}" );
}

/// Opaque colors (alpha = 1.0) emit no opacity attribute at all.
#[ test ]
fn opaque_color_omits_opacity_attribute()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[ RenderCommand::Clear( Clear { color : [ 0.0, 1.0, 0.0, 1.0 ] } ) ] ).unwrap();
  let b = body( &svg );
  assert!( b.contains( "fill=\"rgb(0,255,0)\"" ), "expected opaque rgb: {b}" );
  assert!( !b.contains( "fill-opacity" ), "opaque color should not emit opacity attr: {b}" );
}

/// Verifies that XML-special characters in the Char stream are escaped
/// so they cannot break out of the <text> element and inject markup.
#[ test ]
fn text_escapes_xml_special_characters()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  let injection = "</text><script>x</script>";
  let mut cmds : Vec< RenderCommand > = vec![ begin_text_cmd( TextAnchor::TopLeft, [ 0.0, 0.0 ] ) ];
  cmds.extend( injection.chars().map( | c | RenderCommand::Char( Char( c ) ) ) );
  cmds.push( RenderCommand::EndText( EndText ) );
  svg.submit( &cmds ).unwrap();

  let b = body( &svg );
  // The raw injection must NOT appear — the </text> and <script> tags must be escaped.
  assert!( !b.contains( "</text><script>" ), "injection not escaped: {b}" );
  assert!( !b.contains( "<script>" ), "script tag leaked: {b}" );
  // The escaped form must be present.
  assert!( b.contains( "&lt;/text&gt;&lt;script&gt;" ), "expected escaped form: {b}" );
}

/// Verifies that `EndText` without `BeginText` is silently ignored (no panic, no output).
#[ test ]
fn text_end_without_begin_is_noop()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[ RenderCommand::EndText( EndText ) ] ).unwrap();
  assert!( !body( &svg ).contains( "<text" ) );
}

/// Verifies font-size is emitted in the `<text>` element.
#[ test ]
fn text_emits_font_size()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    RenderCommand::BeginText( BeginText
    {
      font : ResourceId::new( 0 ),
      size : 24.0,
      color : [ 0.0, 0.0, 0.0, 1.0 ],
      anchor : TextAnchor::Center,
      position : [ 0.0, 0.0 ],
      along_path : None,
      clip : None,
    }),
    RenderCommand::Char( Char( 'A' ) ),
    RenderCommand::EndText( EndText ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "font-size=\"24\"" ), "body: {b}" );
}

/// Verifies that anchor attributes from `BeginText` are written into the `<text>` element.
#[ test ]
fn text_anchor_attrs_in_output()
{
  let mut svg = svg800x600();
  svg.assets_load( &empty_assets() ).unwrap();
  svg.submit( &[
    begin_text_cmd( TextAnchor::BottomRight, [ 0.0, 0.0 ] ),
    RenderCommand::Char( Char( 'X' ) ),
    RenderCommand::EndText( EndText ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "text-anchor=\"end\"" ), "body: {b}" );
  assert!( b.contains( "dominant-baseline=\"baseline\"" ), "body: {b}" );
}

/// Verifies that text with `along_path` produces a `<textPath href="#path_N">` element.
#[ test ]
fn text_along_path_emits_text_path()
{
  use tilemap_renderer::assets::{ PathAsset, PathSegment };

  let mut svg = svg800x600();
  let assets = Assets
  {
    paths : vec![ PathAsset
    {
      id : ResourceId::new( 3 ),
      segments : vec![ PathSegment::MoveTo( 0.0, 0.0 ), PathSegment::LineTo( 200.0, 0.0 ) ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  svg.submit( &[
    RenderCommand::BeginText( BeginText
    {
      font : ResourceId::new( 0 ),
      size : 12.0,
      color : [ 0.0, 0.0, 0.0, 1.0 ],
      anchor : TextAnchor::Center,
      position : [ 0.0, 0.0 ],
      along_path : Some( ResourceId::new( 3 ) ),
      clip : None,
    }),
    RenderCommand::Char( Char( 'A' ) ),
    RenderCommand::Char( Char( 'B' ) ),
    RenderCommand::EndText( EndText ),
  ]).unwrap();

  let b = body( &svg );
  assert!( b.contains( "<textPath" ), "body: {b}" );
  assert!( b.contains( "href=\"#path_3\"" ), "body: {b}" );
  assert!( b.contains( "AB" ), "body: {b}" );
}

// ============================================================================
// Formatting/encoding helper tests, relocated from inline `src/adapters/svg.rs`
// per the all-tests-in-tests/ convention ( the helpers are exported for exactly
// this purpose; `#[ doc( hidden ) ]` marks the internal ones ).
// ============================================================================

// -- transform Y-up --

#[ test ]
fn transform_y_up_bottom_left_origin()
{
  // Position (0,0) in Y-up should map to SVG (0, height=600)
  let s = SvgBackend::transform_to_svg_static(
    &Transform { position : [ 0.0, 0.0 ], ..Default::default() },
    600,
  );
  assert!( s.contains( "translate(0,600)" ), "got: {s}" );
}

#[ test ]
fn transform_y_up_top_right()
{
  // Position (800,600) should map to SVG (800, 0)
  let s = SvgBackend::transform_to_svg_static(
    &Transform { position : [ 800.0, 600.0 ], ..Default::default() },
    600,
  );
  assert!( s.contains( "translate(800,0)" ), "got: {s}" );
}

#[ test ]
fn transform_y_up_center()
{
  // Position (400,300) should map to SVG (400, 300)
  let s = SvgBackend::transform_to_svg_static(
    &Transform { position : [ 400.0, 300.0 ], ..Default::default() },
    600,
  );
  assert!( s.contains( "translate(400,300)" ), "got: {s}" );
}

#[ test ]
fn transform_rotation_negated()
{
  let angle = core::f32::consts::FRAC_PI_4; // 45° CCW in Y-up
  let s = SvgBackend::transform_to_svg_static(
    &Transform { rotation : angle, ..Default::default() },
    600,
  );
  // Should emit negative degrees in SVG
  assert!( s.contains( "rotate(-45" ), "got: {s}" );
}

#[ test ]
fn transform_scale_y_negated()
{
  let s = SvgBackend::transform_to_svg_static(
    &Transform { scale : [ 2.0, 3.0 ], ..Default::default() },
    600,
  );
  // scale Y should be negated: 3.0 → -3.0
  assert!( s.contains( "scale(2,-3)" ), "got: {s}" );
}

#[ test ]
fn transform_identity_scale_emits_y_flip()
{
  // Default scale (1,1) should still emit scale(1,-1) for Y-flip
  let s = SvgBackend::transform_to_svg_static(
    &Transform::default(),
    600,
  );
  assert!( s.contains( "scale(1,-1)" ), "got: {s}" );
}

/// Verify that zoom=1.0 does NOT inject scale(1) noise into per-element transforms.
#[ test ]
fn transform_no_zoom_in_per_element_transform()
{
  let s = SvgBackend::transform_to_svg_static(
    &Transform::default(),
    600,
  );
  // Only scale(1,-1) for Y-flip should be present; no zoom prefix
  assert!( !s.contains( "scale(1) " ), "got: {s}" );
}

#[ test ]
fn transform_skew_negated()
{
  let angle = core::f32::consts::FRAC_PI_6; // 30°
  let s = SvgBackend::transform_to_svg_static(
    &Transform { skew : [ angle, 0.0 ], ..Default::default() },
    600,
  );
  assert!( s.contains( "skewX(-30" ), "got: {s}" );
}

// -- local transform (for batch instances inside Y-flipped group) --

#[ test ]
fn local_transform_no_y_flip()
{
  let s = SvgBackend::transform_to_svg_local( &Transform
  {
    position : [ 10.0, 20.0 ],
    rotation : 0.5,
    scale : [ 2.0, 3.0 ],
    ..Default::default()
  });
  // Position is raw, no Y-flip
  assert!( s.contains( "translate(10,20)" ), "got: {s}" );
  // Rotation is raw (positive), not negated
  let deg = 0.5_f32.to_degrees();
  assert!( s.contains( &format!( "rotate({deg})" ) ), "got: {s}" );
  // Scale is raw, no Y negation
  assert!( s.contains( "scale(2,3)" ), "got: {s}" );
}

// -- content manager --

#[ test ]
fn content_manager_push_clear_cycle()
{
  let mut cm = SvgContentManager::new( 100, 100, "" );
  cm.asset_def_push( "<test-def/>" );
  cm.body_push( "<test-body/>" );

  let buf = cm.buffer();
  assert!( buf.contains( "<test-def/>" ) );
  assert!( buf.contains( "<test-body/>" ) );

  cm.body_clear();
  let buf = cm.buffer();
  assert!( buf.contains( "<test-def/>" ) );
  assert!( !buf.contains( "<test-body/>" ) );

  cm.defs_clear();
  let buf = cm.buffer();
  assert!( !buf.contains( "<test-def/>" ) );
}

// -- png_dimensions --

/// Verifies that `png_dimensions` extracts correct width/height from valid PNG bytes.
#[ test ]
fn png_dimensions_valid()
{
  // Generate a real 3×5 PNG via bitmap_to_png, then extract dimensions from its header.
  let bytes = vec![ 0u8; 3 * 5 * 4 ];
  let png = SvgBackend::bitmap_to_png( &bytes, 3, 5, PixelFormat::Rgba8 ).unwrap();
  assert_eq!( SvgBackend::png_dimensions( &png ), Some( ( 3, 5 ) ) );
}

/// Verifies MIME type detection from magic bytes.
#[ test ]
fn detect_image_mime_by_magic()
{
  // PNG
  assert_eq!( image_mime_detect( &[ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0 ] ), "image/png" );
  // JPEG
  assert_eq!( image_mime_detect( &[ 0xff, 0xd8, 0xff, 0xe0 ] ), "image/jpeg" );
  // GIF
  assert_eq!( image_mime_detect( b"GIF89a..." ), "image/gif" );
  // WebP
  let mut webp = Vec::from( *b"RIFF\0\0\0\0WEBP" );
  webp.push( 0 );
  assert_eq!( image_mime_detect( &webp ), "image/webp" );
  // Unknown falls back to PNG
  assert_eq!( image_mime_detect( &[ 0, 0, 0, 0 ] ), "image/png" );
}

/// Verifies that `path_to_href` produces a valid URI reference:
/// spaces become %20 and Windows backslashes become forward slashes.
#[ test ]
fn image_path_produces_valid_uri_reference()
{
  assert_eq!( SvgBackend::path_to_href( "images/tile set/floor.png" ), "images/tile%20set/floor.png" );
  assert_eq!( SvgBackend::path_to_href( r"images\tiles\floor.png" ), "images/tiles/floor.png" );
  assert_eq!( SvgBackend::path_to_href( "safe-name_1.2.png" ), "safe-name_1.2.png" );
  // All URI-reserved and XML-unsafe characters are percent-encoded.
  let e = SvgBackend::path_to_href( "a\"b<c>d&e#f?g%h" );
  assert!( !e.contains( '"' ) && !e.contains( '<' ) && !e.contains( '>' ) && !e.contains( '&' ), "unsafe char leaked: {e}" );
}

/// Verifies that a short / non-PNG buffer returns None.
#[ test ]
fn png_dimensions_invalid()
{
  assert_eq!( SvgBackend::png_dimensions( &[] ), None );
  assert_eq!( SvgBackend::png_dimensions( &[ 0u8; 24 ] ), None ); // no PNG signature
}

/// Verifies that `assets_load` extracts PNG dimensions from `ImageSource::Encoded`
/// so that a sprite symbol uses the correct sheet size.
#[ test ]
fn image_encoded_png_stores_dimensions()
{
  let png = SvgBackend::bitmap_to_png( &[ 0u8; 8 * 4 * 4 ], 8, 4, PixelFormat::Rgba8 ).unwrap();
  let mut svg = svg800x600();
  let assets = Assets
  {
    images : vec![ ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Encoded( png ),
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
      wrap : WrapMode::Clamp,
    }],
    sprites : vec![ SpriteAsset
    {
      id : ResourceId::new( 0 ),
      sheet : ResourceId::new( 0 ),
      region : [ 0.0, 0.0, 4.0, 4.0 ],
    }],
    ..empty_assets()
  };
  svg.assets_load( &assets ).unwrap();
  let d = defs( &svg );
  // The sprite symbol's <use> must reference width="8" height="4" (the sheet size)
  assert!( d.contains( "width=\"8\"" ), "defs: {d}" );
  assert!( d.contains( "height=\"4\"" ), "defs: {d}" );
}

const PNG_MAGIC : &[ u8 ] = &[ 0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a ];

/// Verifies that a 1×1 Rgba8 pixel buffer produces valid PNG output
/// (starts with the PNG magic bytes).
#[ test ]
fn bitmap_to_png_rgba8_valid()
{
  let png = SvgBackend::bitmap_to_png( &[ 255, 0, 128, 255 ], 1, 1, PixelFormat::Rgba8 );
  let bytes = png.expect( "expected Some for valid 1x1 Rgba8" );
  assert!( bytes.starts_with( PNG_MAGIC ), "not PNG: {:?}", &bytes[ ..8.min( bytes.len() ) ] );
}

/// Verifies that a 1×1 Rgb8 pixel buffer encodes successfully.
#[ test ]
fn bitmap_to_png_rgb8_valid()
{
  let png = SvgBackend::bitmap_to_png( &[ 255, 0, 128 ], 1, 1, PixelFormat::Rgb8 );
  assert!( png.is_some(), "expected Some for valid 1x1 Rgb8" );
}

/// Verifies that a 1×1 Gray8 pixel buffer encodes successfully.
#[ test ]
fn bitmap_to_png_gray8_valid()
{
  let png = SvgBackend::bitmap_to_png( &[ 128 ], 1, 1, PixelFormat::Gray8 );
  assert!( png.is_some(), "expected Some for valid 1x1 Gray8" );
}

/// Verifies that a 1×1 `GrayAlpha8` pixel buffer encodes successfully.
#[ test ]
fn bitmap_to_png_gray_alpha8_valid()
{
  let png = SvgBackend::bitmap_to_png( &[ 128, 255 ], 1, 1, PixelFormat::GrayAlpha8 );
  assert!( png.is_some(), "expected Some for valid 1x1 GrayAlpha8" );
}

/// Verifies that mismatched dimensions (too few bytes for the declared size) return None.
#[ test ]
fn bitmap_to_png_dimension_mismatch_returns_none()
{
  // 2×2 Rgba8 needs 16 bytes; supplying only 4 must return None
  let png = SvgBackend::bitmap_to_png( &[ 255, 0, 0, 255 ], 2, 2, PixelFormat::Rgba8 );
  assert!( png.is_none(), "expected None for undersized buffer" );
}

/// Verifies pixel-for-pixel round-trip fidelity through `bitmap_to_png` for
/// all 4 `PixelFormat` variants — the tests above only check
/// magic-bytes/`is_some()`, not that the encoded pixel data is correct.
#[ test ]
fn bitmap_to_png_round_trip_pixel_fidelity()
{
  let cases : &[ ( PixelFormat, u32, u32, &[ u8 ] ) ] =
  &[
    ( PixelFormat::Rgba8, 2, 2, &[ 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160 ] ),
    ( PixelFormat::Rgb8, 3, 1, &[ 10, 20, 30, 40, 50, 60, 70, 80, 90 ] ),
    ( PixelFormat::Gray8, 2, 2, &[ 10, 20, 30, 40 ] ),
    ( PixelFormat::GrayAlpha8, 3, 1, &[ 10, 20, 30, 40, 50, 60 ] ),
  ];

  for &( format, width, height, pixels ) in cases
  {
    let png = SvgBackend::bitmap_to_png( pixels, width, height, format )
      .unwrap_or_else( || panic!( "expected Some for valid {width}x{height} {format:?}" ) );

    let decoder = png::Decoder::new( std::io::Cursor::new( png ) );
    let mut reader = decoder.read_info().expect( "valid PNG should decode" );
    let mut buf = vec![ 0u8; reader.output_buffer_size().expect( "png output buffer size should fit in usize" ) ];
    let info = reader.next_frame( &mut buf ).expect( "should decode frame" );

    assert_eq!( &buf[ ..info.buffer_size() ], pixels, "pixel mismatch for {format:?}" );
  }
}

// anchor_to_svg — 9 variants

#[ test ]
fn anchor_top_left()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::TopLeft );
  assert_eq!( h, "start" );
  assert_eq!( v, "hanging" );
}

#[ test ]
fn anchor_top_center()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::TopCenter );
  assert_eq!( h, "middle" );
  assert_eq!( v, "hanging" );
}

#[ test ]
fn anchor_top_right()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::TopRight );
  assert_eq!( h, "end" );
  assert_eq!( v, "hanging" );
}

#[ test ]
fn anchor_center_left()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::CenterLeft );
  assert_eq!( h, "start" );
  assert_eq!( v, "central" );
}

#[ test ]
fn anchor_center()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::Center );
  assert_eq!( h, "middle" );
  assert_eq!( v, "central" );
}

#[ test ]
fn anchor_center_right()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::CenterRight );
  assert_eq!( h, "end" );
  assert_eq!( v, "central" );
}

#[ test ]
fn anchor_bottom_left()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::BottomLeft );
  assert_eq!( h, "start" );
  assert_eq!( v, "baseline" );
}

#[ test ]
fn anchor_bottom_center()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::BottomCenter );
  assert_eq!( h, "middle" );
  assert_eq!( v, "baseline" );
}

#[ test ]
fn anchor_bottom_right()
{
  let ( h, v ) = SvgBackend::anchor_to_svg( TextAnchor::BottomRight );
  assert_eq!( h, "end" );
  assert_eq!( v, "baseline" );
}
