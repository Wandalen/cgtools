//! Test suite for rendering commands and primitives.
//!
//! ## Test Matrix for Commands Module

#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::clone_on_copy ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]
//!
//! ### Test Factors:
//! - **Command Type**: Line, Curve, Text, Tilemap, ParticleEmitter, RenderCommand
//! - **Data Validation**: Structure creation, serialization, POD compliance
//! - **Edge Cases**: Text truncation, tilemap truncation, default values
//! - **Implementation**: Helper functions, data extraction
//!
//! ### Test Combinations:
//!
//! | ID   | Aspect Tested | Command Type | Test Focus | Expected Behavior |
//! |------|---------------|--------------|------------|-------------------|
//! | T1.1 | POD Compliance | LineCommand | Copy/Clone/Serialize traits | All traits implemented correctly |
//! | T1.2 | POD Compliance | CurveCommand | Copy/Clone/Serialize traits | All traits implemented correctly |
//! | T1.3 | POD Compliance | TextCommand | Copy/Clone/Serialize traits | All traits implemented correctly |
//! | T1.4 | POD Compliance | TilemapCommand | Copy/Clone/Serialize traits | All traits implemented correctly |
//! | T1.5 | POD Compliance | ParticleEmitterCommand | Copy/Clone/Serialize traits | All traits implemented correctly |
//! | T1.6 | POD Compliance | RenderCommand | Copy/Clone/Serialize traits | All traits implemented correctly |
//! | T2.1 | Structure Creation | LineCommand | Basic creation | Valid command created |
//! | T2.2 | Structure Creation | CurveCommand | Basic creation | Valid command created |
//! | T2.3 | Structure Creation | TextCommand | Basic creation via new() | Valid command with proper text handling |
//! | T2.4 | Structure Creation | TilemapCommand | Basic creation via new() | Valid command with proper tile handling |
//! | T2.5 | Structure Creation | ParticleEmitterCommand | Basic creation | Valid command created |
//! | T3.1 | Edge Cases | TextCommand | Text truncation | Text > 255 chars truncated properly |
//! | T3.2 | Edge Cases | TilemapCommand | Tile truncation | Tiles > 1024 truncated properly |
//! | T3.3 | Default Values | StrokeStyle | Default creation | Sensible default values |
//! | T3.4 | Default Values | FontStyle | Default creation | Sensible default values |
//! | T3.5 | Default Values | Point2D | Default creation | Zero coordinates |
//! | T4.1 | Helper Functions | TextCommand | Text extraction | Proper string extraction from bytes |
//! | T4.2 | Helper Functions | TilemapCommand | Tile extraction | Proper slice extraction from array |
//! | T5.1 | Enum Wrapper | RenderCommand | Line variant | Proper wrapping of LineCommand |
//! | T5.2 | Enum Wrapper | RenderCommand | Curve variant | Proper wrapping of CurveCommand |
//! | T5.3 | Enum Wrapper | RenderCommand | Text variant | Proper wrapping of TextCommand |
//! | T5.4 | Enum Wrapper | RenderCommand | Tilemap variant | Proper wrapping of TilemapCommand |
//! | T5.5 | Enum Wrapper | RenderCommand | ParticleEmitter variant | Proper wrapping of ParticleEmitterCommand |

#![ allow( unused_imports ) ]
use tilemap_renderer as the_module;
use the_module::commands::*;

/// Tests that LineCommand implements POD traits correctly.
/// Test Combination: T1.1
#[ test ]
fn test_line_command_pod_compliance()
{
  let cmd = LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  };

  // Test Copy
  let _copied = cmd;
  let _also_copied = cmd; // Should work due to Copy

  // Test Clone
  let _cloned = cmd.clone();

  // Test serialization
  let _serialized = serde_json::to_string( &cmd ).expect( "Should serialize" );
}

/// Tests that CurveCommand implements POD traits correctly.
/// Test Combination: T1.2
#[ test ]
fn test_curve_command_pod_compliance()
{
  let cmd = CurveCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    control1: Point2D { x: 5.0, y: 5.0 },
    control2: Point2D { x: 15.0, y: 5.0 },
    end: Point2D { x: 20.0, y: 0.0 },
    style: StrokeStyle::default(),
  };

  // Test Copy and Clone
  let _copied = cmd;
  let _cloned = cmd.clone();

  // Test serialization
  let _serialized = serde_json::to_string( &cmd ).expect( "Should serialize" );
}

/// Tests that TextCommand implements POD traits correctly.
/// Test Combination: T1.3
#[ test ]
fn test_text_command_pod_compliance()
{
  let cmd = TextCommand::new(
    "Hello",
    Point2D { x: 10.0, y: 20.0 },
    FontStyle::default(),
    TextAnchor::Center
  );

  // Test Copy and Clone
  let _copied = cmd;
  let _cloned = cmd.clone();

  // Test serialization
  let _serialized = serde_json::to_string( &cmd ).expect( "Should serialize" );
}

/// Tests that TilemapCommand implements POD traits correctly.
/// Test Combination: T1.4
#[ test ]
fn test_tilemap_command_pod_compliance()
{
  let tiles = vec![ 1, 2, 3, 4 ];
  let cmd = TilemapCommand::new(
    Point2D { x: 0.0, y: 0.0 },
    32.0,
    32.0,
    2,
    2,
    0,
    &tiles
  );

  // Test Copy and Clone
  let _copied = cmd;
  let _cloned = cmd.clone();

  // Test serialization
  let _serialized = serde_json::to_string( &cmd ).expect( "Should serialize" );
}

/// Tests that ParticleEmitterCommand implements POD traits correctly.
/// Test Combination: T1.5
#[ test ]
fn test_particle_emitter_command_pod_compliance()
{
  let cmd = ParticleEmitterCommand
  {
    position: Point2D { x: 0.0, y: 0.0 },
    emission_rate: 10.0,
    particle_lifetime: 2.0,
    initial_velocity: Point2D { x: 1.0, y: -1.0 },
    velocity_variance: Point2D { x: 0.5, y: 0.5 },
    particle_size: 4.0,
    size_variance: 1.0,
    particle_color: [ 1.0, 0.0, 0.0, 1.0 ],
    color_variance: [ 0.1, 0.1, 0.1, 0.0 ],
  };

  // Test Copy and Clone
  let _copied = cmd;
  let _cloned = cmd.clone();

  // Test serialization
  let _serialized = serde_json::to_string( &cmd ).expect( "Should serialize" );
}

/// Tests that RenderCommand implements POD traits correctly.
/// Test Combination: T1.6
#[ test ]
fn test_render_command_pod_compliance()
{
  let line_cmd = LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  };
  let cmd = RenderCommand::Line( line_cmd );

  // Test Copy and Clone
  let _copied = cmd;
  let _cloned = cmd.clone();

  // Test serialization
  let _serialized = serde_json::to_string( &cmd ).expect( "Should serialize" );
}

/// Tests basic LineCommand creation.
/// Test Combination: T2.1
#[ test ]
fn test_line_command_creation()
{
  let start = Point2D { x: 1.0, y: 2.0 };
  let end = Point2D { x: 3.0, y: 4.0 };
  let style = StrokeStyle { width: 2.0, color: [ 1.0, 0.0, 0.0, 1.0 ], cap_style: LineCap::Round, join_style: LineJoin::Round };

  let cmd = LineCommand { start, end, style };

  assert_eq!( cmd.start.x, 1.0 );
  assert_eq!( cmd.start.y, 2.0 );
  assert_eq!( cmd.end.x, 3.0 );
  assert_eq!( cmd.end.y, 4.0 );
  assert_eq!( cmd.style.width, 2.0 );
}

/// Tests basic CurveCommand creation.
/// Test Combination: T2.2
#[ test ]
fn test_curve_command_creation()
{
  let start = Point2D { x: 0.0, y: 0.0 };
  let control1 = Point2D { x: 10.0, y: 10.0 };
  let control2 = Point2D { x: 20.0, y: 10.0 };
  let end = Point2D { x: 30.0, y: 0.0 };
  let style = StrokeStyle::default();

  let cmd = CurveCommand { start, control1, control2, end, style };

  assert_eq!( cmd.start.x, 0.0 );
  assert_eq!( cmd.control1.x, 10.0 );
  assert_eq!( cmd.control2.x, 20.0 );
  assert_eq!( cmd.end.x, 30.0 );
}

/// Tests TextCommand creation via new() method.
/// Test Combination: T2.3
#[ test ]
fn test_text_command_creation()
{
  let position = Point2D { x: 50.0, y: 100.0 };
  let font_style = FontStyle { size: 16.0, color: [ 0.0, 1.0, 0.0, 1.0 ], weight: 600, italic: true, family_id: 1 };
  let cmd = TextCommand::new( "Test Text", position, font_style, TextAnchor::TopLeft );

  assert_eq!( cmd.position.x, 50.0 );
  assert_eq!( cmd.position.y, 100.0 );
  assert_eq!( cmd.text(), "Test Text" );
  assert_eq!( cmd.font_style.size, 16.0 );
  assert_eq!( cmd.font_style.weight, 600 );
  assert!( cmd.font_style.italic );
  assert_eq!( cmd.anchor, TextAnchor::TopLeft );
}

/// Tests TilemapCommand creation via new() method.
/// Test Combination: T2.4
#[ test ]
fn test_tilemap_command_creation()
{
  let position = Point2D { x: 100.0, y: 200.0 };
  let tiles = vec![ 1, 2, 3, 4, 5, 6 ];
  let cmd = TilemapCommand::new( position, 32.0, 32.0, 3, 2, 1, &tiles );

  assert_eq!( cmd.position.x, 100.0 );
  assert_eq!( cmd.position.y, 200.0 );
  assert_eq!( cmd.tile_width, 32.0 );
  assert_eq!( cmd.tile_height, 32.0 );
  assert_eq!( cmd.map_width, 3 );
  assert_eq!( cmd.map_height, 2 );
  assert_eq!( cmd.tileset_id, 1 );
  assert_eq!( cmd.tiles(), &[ 1, 2, 3, 4, 5, 6 ] );
  assert_eq!( cmd.tile_count, 6 );
}

/// Tests ParticleEmitterCommand creation.
/// Test Combination: T2.5
#[ test ]
fn test_particle_emitter_command_creation()
{
  let cmd = ParticleEmitterCommand
  {
    position: Point2D { x: 150.0, y: 250.0 },
    emission_rate: 50.0,
    particle_lifetime: 3.0,
    initial_velocity: Point2D { x: 2.0, y: -3.0 },
    velocity_variance: Point2D { x: 1.0, y: 1.0 },
    particle_size: 8.0,
    size_variance: 2.0,
    particle_color: [ 0.0, 0.0, 1.0, 1.0 ],
    color_variance: [ 0.2, 0.2, 0.2, 0.0 ],
  };

  assert_eq!( cmd.position.x, 150.0 );
  assert_eq!( cmd.emission_rate, 50.0 );
  assert_eq!( cmd.particle_lifetime, 3.0 );
  assert_eq!( cmd.particle_size, 8.0 );
}

/// Tests text truncation for long strings.
/// Test Combination: T3.1
#[ test ]
fn test_text_command_truncation()
{
  let long_text = "A".repeat( 100 ); // 100 characters, should be truncated to 63
  let cmd = TextCommand::new( &long_text, Point2D::default(), FontStyle::default(), TextAnchor::Center );

  assert_eq!( cmd.text_len, 63 );
  assert_eq!( cmd.text().len(), 63 );
  assert_eq!( cmd.text(), &"A".repeat( 63 ) );
}

/// Tests tile data truncation for large tilemaps.
/// Test Combination: T3.2
#[ test ]
fn test_tilemap_command_truncation()
{
  let large_tiles: Vec< u16 > = ( 0..100 ).collect(); // 100 tiles, should be truncated to 32
  let cmd = TilemapCommand::new( Point2D::default(), 16.0, 16.0, 50, 40, 0, &large_tiles );

  assert_eq!( cmd.tile_count, 32 );
  assert_eq!( cmd.tiles().len(), 32 );
  assert_eq!( cmd.tiles()[ 0 ], 0 );
  assert_eq!( cmd.tiles()[ 31 ], 31 );
}

/// Tests StrokeStyle default values.
/// Test Combination: T3.3
#[ test ]
fn test_stroke_style_default()
{
  let style = StrokeStyle::default();

  assert_eq!( style.width, 1.0 );
  assert_eq!( style.color, [ 0.0, 0.0, 0.0, 1.0 ] ); // Black
  assert_eq!( style.cap_style, LineCap::Butt );
  assert_eq!( style.join_style, LineJoin::Miter );
}

/// Tests FontStyle default values.
/// Test Combination: T3.4
#[ test ]
fn test_font_style_default()
{
  let style = FontStyle::default();

  assert_eq!( style.size, 12.0 );
  assert_eq!( style.color, [ 0.0, 0.0, 0.0, 1.0 ] ); // Black
  assert_eq!( style.weight, 400 ); // Normal
  assert!( !style.italic );
  assert_eq!( style.family_id, 0 );
}

/// Tests Point2D default values.
/// Test Combination: T3.5
#[ test ]
fn test_point2d_default()
{
  let point = Point2D::default();

  assert_eq!( point.x, 0.0 );
  assert_eq!( point.y, 0.0 );
}

/// Tests TextCommand text extraction functionality.
/// Test Combination: T4.1
#[ test ]
fn test_text_command_text_extraction()
{
  let test_texts = [ "Hello", "", "🦀 Rust", "Mixed ASCII and émojis" ];

  for text in &test_texts
  {
    let cmd = TextCommand::new( text, Point2D::default(), FontStyle::default(), TextAnchor::Center );
    assert_eq!( cmd.text(), *text );
  }
}

/// Tests TilemapCommand tile extraction functionality.
/// Test Combination: T4.2
#[ test ]
fn test_tilemap_command_tile_extraction()
{
  let tiles_data = [ vec![], vec![ 1 ], vec![ 1, 2, 3, 4, 5 ] ];

  for tiles in &tiles_data
  {
    let cmd = TilemapCommand::new( Point2D::default(), 16.0, 16.0, 1, 1, 0, tiles );
    assert_eq!( cmd.tiles(), tiles.as_slice() );
  }
}

/// Tests RenderCommand Line variant wrapping.
/// Test Combination: T5.1
#[ test ]
fn test_render_command_line_variant()
{
  let line_cmd = LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 1.0, y: 1.0 },
    style: StrokeStyle::default(),
  };
  let render_cmd = RenderCommand::Line( line_cmd );

  match render_cmd
  {
    RenderCommand::Line( cmd ) =>
    {
      assert_eq!( cmd.start.x, 0.0 );
      assert_eq!( cmd.end.x, 1.0 );
    }
    _ => panic!( "Expected Line variant" ),
  }
}

/// Tests RenderCommand Curve variant wrapping.
/// Test Combination: T5.2
#[ test ]
fn test_render_command_curve_variant()
{
  let curve_cmd = CurveCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    control1: Point2D { x: 1.0, y: 1.0 },
    control2: Point2D { x: 2.0, y: 1.0 },
    end: Point2D { x: 3.0, y: 0.0 },
    style: StrokeStyle::default(),
  };
  let render_cmd = RenderCommand::Curve( curve_cmd );

  match render_cmd
  {
    RenderCommand::Curve( cmd ) =>
    {
      assert_eq!( cmd.start.x, 0.0 );
      assert_eq!( cmd.end.x, 3.0 );
    }
    _ => panic!( "Expected Curve variant" ),
  }
}

/// Tests RenderCommand Text variant wrapping.
/// Test Combination: T5.3
#[ test ]
fn test_render_command_text_variant()
{
  let text_cmd = TextCommand::new( "Test", Point2D::default(), FontStyle::default(), TextAnchor::Center );
  let render_cmd = RenderCommand::Text( text_cmd );

  match render_cmd
  {
    RenderCommand::Text( cmd ) =>
    {
      assert_eq!( cmd.text(), "Test" );
    }
    _ => panic!( "Expected Text variant" ),
  }
}

/// Tests RenderCommand Tilemap variant wrapping.
/// Test Combination: T5.4
#[ test ]
fn test_render_command_tilemap_variant()
{
  let tiles = vec![ 1, 2 ];
  let tilemap_cmd = TilemapCommand::new( Point2D::default(), 16.0, 16.0, 1, 2, 0, &tiles );
  let render_cmd = RenderCommand::Tilemap( tilemap_cmd );

  match render_cmd
  {
    RenderCommand::Tilemap( cmd ) =>
    {
      assert_eq!( cmd.tiles().len(), 2 );
    }
    _ => panic!( "Expected Tilemap variant" ),
  }
}

/// Tests RenderCommand ParticleEmitter variant wrapping.
/// Test Combination: T5.5
#[ test ]
fn test_render_command_particle_emitter_variant()
{
  let emitter_cmd = ParticleEmitterCommand
  {
    position: Point2D { x: 10.0, y: 20.0 },
    emission_rate: 5.0,
    particle_lifetime: 1.0,
    initial_velocity: Point2D::default(),
    velocity_variance: Point2D::default(),
    particle_size: 2.0,
    size_variance: 0.0,
    particle_color: [ 1.0, 1.0, 1.0, 1.0 ],
    color_variance: [ 0.0, 0.0, 0.0, 0.0 ],
  };
  let render_cmd = RenderCommand::ParticleEmitter( emitter_cmd );

  match render_cmd
  {
    RenderCommand::ParticleEmitter( cmd ) =>
    {
      assert_eq!( cmd.position.x, 10.0 );
      assert_eq!( cmd.emission_rate, 5.0 );
    }
    _ => panic!( "Expected ParticleEmitter variant" ),
  }
}