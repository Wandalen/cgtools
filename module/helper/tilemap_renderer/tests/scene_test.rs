//! Test suite for scene management system.
//!
//! ## Test Matrix for Scene Module
//!
//! ### Test Factors:
//! - **Scene Operations**: Creation, command addition, querying, management
//! - **Data Integrity**: Command ordering, scene state, statistics
//! - **Query Capabilities**: Type filtering, predicate filtering, result handling
//! - **Edge Cases**: Empty scenes, large command counts, invalid operations

#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::needless_return ) ]

use tilemap_renderer as the_module;
use the_module::scene::*;
use the_module::commands::*;

/// Tests basic scene creation and properties.
/// Test Focus: FR-A1 - Scene object creation
#[ test ]
fn test_scene_creation()
{
  let scene = Scene::new();
  
  assert!( scene.is_empty() );
  assert_eq!( scene.len(), 0 );
  assert!( scene.id().is_none() );
}

/// Tests scene creation with identifier.
/// Test Focus: Scene identification functionality
#[ test ]
fn test_scene_with_id()
{
  let scene = Scene::with_id( "test_scene" );
  
  assert!( scene.is_empty() );
  assert_eq!( scene.id(), Some( "test_scene" ) );
  
  let mut scene2 = Scene::new();
  scene2.set_id( "another_scene" );
  assert_eq!( scene2.id(), Some( "another_scene" ) );
}

/// Tests adding single commands to scene.
/// Test Focus: FR-A3 - Scene.add method, FR-A2 - Ordered list
#[ test ]
fn test_scene_add_single_command()
{
  let mut scene = Scene::new();
  
  let line_cmd = RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } );
  
  scene.add( line_cmd );
  
  assert_eq!( scene.len(), 1 );
  assert!( !scene.is_empty() );
  assert!( matches!( scene.get( 0 ), Some( RenderCommand::Line( _ ) ) ) );
}

/// Tests adding multiple commands maintains order.
/// Test Focus: FR-A2 - Ordered list of commands
#[ test ]
fn test_scene_command_ordering()
{
  let mut scene = Scene::new();
  
  let line_cmd = RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } );
  
  let curve_cmd = RenderCommand::Curve( CurveCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    control1: Point2D { x: 5.0, y: 5.0 },
    control2: Point2D { x: 15.0, y: 5.0 },
    end: Point2D { x: 20.0, y: 0.0 },
    style: StrokeStyle::default(),
  } );
  
  let text_cmd = RenderCommand::Text( 
    TextCommand::new( "Hello", Point2D::default(), FontStyle::default(), TextAnchor::Center )
  );
  
  scene.add( line_cmd );
  scene.add( curve_cmd );
  scene.add( text_cmd );
  
  assert_eq!( scene.len(), 3 );
  assert!( matches!( scene.get( 0 ), Some( RenderCommand::Line( _ ) ) ) );
  assert!( matches!( scene.get( 1 ), Some( RenderCommand::Curve( _ ) ) ) );
  assert!( matches!( scene.get( 2 ), Some( RenderCommand::Text( _ ) ) ) );
}

/// Tests adding multiple commands at once.
/// Test Focus: Bulk command addition
#[ test ]
fn test_scene_add_many_commands()
{
  let mut scene = Scene::new();
  
  let commands = vec![
    RenderCommand::Line( LineCommand
    {
      start: Point2D { x: 0.0, y: 0.0 },
      end: Point2D { x: 10.0, y: 10.0 },
      style: StrokeStyle::default(),
    } ),
    RenderCommand::Line( LineCommand
    {
      start: Point2D { x: 5.0, y: 5.0 },
      end: Point2D { x: 15.0, y: 15.0 },
      style: StrokeStyle::default(),
    } ),
  ];
  
  scene.add_many( commands );
  
  assert_eq!( scene.len(), 2 );
  assert!( matches!( scene.get( 0 ), Some( RenderCommand::Line( _ ) ) ) );
  assert!( matches!( scene.get( 1 ), Some( RenderCommand::Line( _ ) ) ) );
}

/// Tests scene clearing functionality.
/// Test Focus: Scene state management
#[ test ]
fn test_scene_clear()
{
  let mut scene = Scene::with_id( "test" );
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  assert_eq!( scene.len(), 1 );
  
  scene.clear();
  
  assert!( scene.is_empty() );
  assert_eq!( scene.len(), 0 );
  assert_eq!( scene.id(), Some( "test" ) ); // ID should remain
}

/// Tests command removal by index.
/// Test Focus: Command management operations
#[ test ]
fn test_scene_remove_command()
{
  let mut scene = Scene::new();
  
  let line1 = RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } );
  
  let line2 = RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 5.0, y: 5.0 },
    end: Point2D { x: 15.0, y: 15.0 },
    style: StrokeStyle::default(),
  } );
  
  scene.add( line1 );
  scene.add( line2 );
  
  assert_eq!( scene.len(), 2 );
  
  let removed = scene.remove( 0 );
  assert!( removed.is_some() );
  assert_eq!( scene.len(), 1 );
  
  // Invalid index should return None
  let removed_invalid = scene.remove( 10 );
  assert!( removed_invalid.is_none() );
  assert_eq!( scene.len(), 1 );
}

/// Tests command insertion at specific index.
/// Test Focus: Command positioning and ordering
#[ test ]
fn test_scene_insert_command()
{
  let mut scene = Scene::new();
  
  let line1 = RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } );
  
  let line2 = RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 20.0, y: 20.0 },
    end: Point2D { x: 30.0, y: 30.0 },
    style: StrokeStyle::default(),
  } );
  
  let line_middle = RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 5.0, y: 5.0 },
    end: Point2D { x: 15.0, y: 15.0 },
    style: StrokeStyle::default(),
  } );
  
  scene.add( line1 );
  scene.add( line2 );
  scene.insert( 1, line_middle );
  
  assert_eq!( scene.len(), 3 );
  
  // Verify order: line1, line_middle, line2
  if let Some( RenderCommand::Line( cmd ) ) = scene.get( 1 )
  {
    assert_eq!( cmd.start.x, 5.0 );
    assert_eq!( cmd.end.x, 15.0 );
  }
  else
  {
    panic!( "Expected Line command at index 1" );
  }
}

/// Tests querying Line commands by type.
/// Test Focus: FR-A6 - Scene querying capabilities
#[ test ]
fn test_scene_query_lines()
{
  let mut scene = Scene::new();
  
  // Add mixed command types
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Text( 
    TextCommand::new( "Hello", Point2D::default(), FontStyle::default(), TextAnchor::Center )
  ) );
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 5.0, y: 5.0 },
    end: Point2D { x: 15.0, y: 15.0 },
    style: StrokeStyle::default(),
  } ) );
  
  let lines = scene.query_lines();
  
  assert_eq!( lines.len(), 2 );
  assert!( !lines.is_empty() );
  
  for line_ref in lines.iter()
  {
    assert!( matches!( line_ref, RenderCommand::Line( _ ) ) );
  }
}

/// Tests querying Text commands by type.
/// Test Focus: FR-A6 - Scene querying capabilities
#[ test ]
fn test_scene_query_text()
{
  let mut scene = Scene::new();
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Text( 
    TextCommand::new( "Hello", Point2D::default(), FontStyle::default(), TextAnchor::Center )
  ) );
  
  scene.add( RenderCommand::Text( 
    TextCommand::new( "World", Point2D { x: 10.0, y: 20.0 }, FontStyle::default(), TextAnchor::TopLeft )
  ) );
  
  let text_commands = scene.query_text();
  
  assert_eq!( text_commands.len(), 2 );
  
  for text_ref in text_commands.iter()
  {
    assert!( matches!( text_ref, RenderCommand::Text( _ ) ) );
  }
}

/// Tests querying with custom predicate.
/// Test Focus: FR-A6 - General querying capabilities
#[ test ]
fn test_scene_query_where()
{
  let mut scene = Scene::new();
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 0.0, y: 0.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle { width: 1.0, color: [ 1.0, 0.0, 0.0, 1.0 ], cap_style: LineCap::Butt, join_style: LineJoin::Miter },
  } ) );
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 5.0, y: 5.0 },
    end: Point2D { x: 15.0, y: 15.0 },
    style: StrokeStyle { width: 2.0, color: [ 0.0, 1.0, 0.0, 1.0 ], cap_style: LineCap::Butt, join_style: LineJoin::Miter },
  } ) );
  
  // Query for thick lines (width > 1.5)
  let thick_lines = scene.query_where( |cmd|
  {
    match cmd
    {
      RenderCommand::Line( line_cmd ) => line_cmd.style.width > 1.5,
      _ => false,
    }
  } );
  
  assert_eq!( thick_lines.len(), 1 );
  
  if let Some( RenderCommand::Line( cmd ) ) = thick_lines.get( 0 )
  {
    assert_eq!( cmd.style.width, 2.0 );
  }
  else
  {
    panic!( "Expected thick line command" );
  }
}

/// Tests scene statistics calculation.
/// Test Focus: Scene analysis and reporting
#[ test ]
fn test_scene_stats()
{
  let mut scene = Scene::new();
  
  // Add various command types
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 5.0, y: 5.0 },
    end: Point2D { x: 15.0, y: 15.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Text( 
    TextCommand::new( "Hello", Point2D::default(), FontStyle::default(), TextAnchor::Center )
  ) );
  
  scene.add( RenderCommand::Curve( CurveCommand
  {
    start: Point2D::default(),
    control1: Point2D { x: 5.0, y: 5.0 },
    control2: Point2D { x: 15.0, y: 5.0 },
    end: Point2D { x: 20.0, y: 0.0 },
    style: StrokeStyle::default(),
  } ) );
  
  let stats = scene.stats();
  
  assert_eq!( stats.total_count, 4 );
  assert_eq!( stats.line_count, 2 );
  assert_eq!( stats.text_count, 1 );
  assert_eq!( stats.curve_count, 1 );
  assert_eq!( stats.tilemap_count, 0 );
  assert_eq!( stats.particle_emitter_count, 0 );
}

/// Tests scene iterator functionality.
/// Test Focus: Scene traversal and iteration
#[ test ]
fn test_scene_iteration()
{
  let mut scene = Scene::new();
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Text( 
    TextCommand::new( "Test", Point2D::default(), FontStyle::default(), TextAnchor::Center )
  ) );
  
  let mut count = 0;
  for _command in scene.commands()
  {
    count += 1;
  }
  
  assert_eq!( count, 2 );
  
  // Test mutable iteration
  let mut mut_count = 0;
  for _command in scene.commands_mut()
  {
    mut_count += 1;
  }
  
  assert_eq!( mut_count, 2 );
}

/// Tests empty scene behavior.
/// Test Focus: Edge case handling
#[ test ]
fn test_empty_scene_behavior()
{
  let scene = Scene::new();
  
  assert!( scene.is_empty() );
  assert_eq!( scene.len(), 0 );
  assert!( scene.get( 0 ).is_none() );
  
  let lines = scene.query_lines();
  assert!( lines.is_empty() );
  
  let stats = scene.stats();
  assert_eq!( stats.total_count, 0 );
  assert_eq!( stats.line_count, 0 );
  
  // Empty query results
  let empty_query = scene.query_where( |_| true );
  assert!( empty_query.is_empty() );
}

/// Tests scene cloning and equality.
/// Test Focus: Scene data integrity
#[ test ]
fn test_scene_clone_and_equality()
{
  let mut scene1 = Scene::with_id( "test_scene" );
  
  scene1.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  let scene2 = scene1.clone();
  
  assert_eq!( scene1, scene2 );
  assert_eq!( scene1.id(), scene2.id() );
  assert_eq!( scene1.len(), scene2.len() );
}

/// Tests mutable access to scene commands.
/// Test Focus: Command modification capabilities
#[ test ]
fn test_scene_mutable_access()
{
  let mut scene = Scene::new();
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  // Test mutable access
  if let Some( RenderCommand::Line( ref mut line_cmd ) ) = scene.get_mut( 0 )
  {
    line_cmd.end.x = 20.0;
  }
  
  // Verify modification
  if let Some( RenderCommand::Line( line_cmd ) ) = scene.get( 0 )
  {
    assert_eq!( line_cmd.end.x, 20.0 );
  }
  else
  {
    panic!( "Expected Line command" );
  }
}

/// Tests all query type methods.
/// Test Focus: Complete FR-A6 query capabilities
#[ test ]
fn test_all_query_types()
{
  let mut scene = Scene::new();
  
  // Add one of each command type
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D::default(),
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Curve( CurveCommand
  {
    start: Point2D::default(),
    control1: Point2D { x: 5.0, y: 5.0 },
    control2: Point2D { x: 15.0, y: 5.0 },
    end: Point2D { x: 20.0, y: 0.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Text( 
    TextCommand::new( "Test", Point2D::default(), FontStyle::default(), TextAnchor::Center )
  ) );
  
  let tiles = vec![ 1, 2, 3, 4 ];
  scene.add( RenderCommand::Tilemap( TilemapCommand::new(
    Point2D::default(),
    32.0,
    32.0,
    2,
    2,
    0,
    &tiles
  ) ) );
  
  scene.add( RenderCommand::ParticleEmitter( ParticleEmitterCommand
  {
    position: Point2D::default(),
    emission_rate: 10.0,
    particle_lifetime: 2.0,
    initial_velocity: Point2D::default(),
    velocity_variance: Point2D::default(),
    particle_size: 4.0,
    size_variance: 1.0,
    particle_color: [ 1.0, 1.0, 1.0, 1.0 ],
    color_variance: [ 0.0, 0.0, 0.0, 0.0 ],
  } ) );
  
  assert_eq!( scene.query_lines().len(), 1 );
  assert_eq!( scene.query_curves().len(), 1 );
  assert_eq!( scene.query_text().len(), 1 );
  assert_eq!( scene.query_tilemaps().len(), 1 );
  assert_eq!( scene.query_particle_emitters().len(), 1 );
}

/// Tests large scene performance characteristics.
/// Test Focus: Scalability and performance considerations
#[ test ]
#[ allow( clippy::cast_precision_loss ) ]
fn test_large_scene_handling()
{
  let mut scene = Scene::new();
  
  // Add many commands
  for i in 0..1000
  {
    scene.add( RenderCommand::Line( LineCommand
    {
      start: Point2D { x: i as f32, y: 0.0 },
      end: Point2D { x: i as f32, y: 10.0 },
      style: StrokeStyle::default(),
    } ) );
  }
  
  assert_eq!( scene.len(), 1000 );
  
  let lines = scene.query_lines();
  assert_eq!( lines.len(), 1000 );
  
  let stats = scene.stats();
  assert_eq!( stats.total_count, 1000 );
  assert_eq!( stats.line_count, 1000 );
}

/// Tests query result functionality.
/// Test Focus: QueryResult type behavior
#[ test ]
fn test_query_result_operations()
{
  let mut scene = Scene::new();
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 1.0, y: 1.0 },
    end: Point2D { x: 10.0, y: 10.0 },
    style: StrokeStyle::default(),
  } ) );
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 2.0, y: 2.0 },
    end: Point2D { x: 20.0, y: 20.0 },
    style: StrokeStyle::default(),
  } ) );
  
  let lines = scene.query_lines();
  
  assert_eq!( lines.len(), 2 );
  assert!( !lines.is_empty() );
  
  // Test indexed access
  let first_line = lines.get( 0 );
  assert!( first_line.is_some() );
  assert!( matches!( first_line, Some( RenderCommand::Line( _ ) ) ) );
  
  let invalid_line = lines.get( 10 );
  assert!( invalid_line.is_none() );
  
  // Test iteration
  let mut iter_count = 0;
  for _line_ref in lines.iter()
  {
    iter_count += 1;
  }
  assert_eq!( iter_count, 2 );
}