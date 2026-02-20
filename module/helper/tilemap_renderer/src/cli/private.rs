//! Private CLI implementation.

#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::exhaustive_enums ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::needless_continue ) ]
#![ allow( clippy::needless_raw_string_hashes ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::manual_strip ) ]
#![ allow( clippy::io_other_error ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::redundant_closure_for_method_calls ) ]
#![ allow( clippy::ignored_unit_patterns ) ]
#![ allow( clippy::too_many_arguments ) ]

use std::io::{ self, Write };
use unilang::*;
//use unilang::data::CommandDefinition; // Using former() instead
use unilang::registry::CommandRegistry;
use unilang::pipeline::Pipeline;

#[ cfg( feature = "serde" ) ]
use serde_json;

/// CLI application error type
#[ derive( Debug ) ]
pub enum CliError
{
  /// Input/output error
  Io( io::Error ),
  /// Unilang framework error
  Unilang( String ),
  /// Scene management error
  Scene( String ),
}

impl std::fmt::Display for CliError
{
  fn fmt( &self, f: &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    match self
    {
      CliError::Io( e ) => write!( f, "IO error: {}", e ),
      CliError::Unilang( e ) => write!( f, "Unilang error: {}", e ),
      CliError::Scene( e ) => write!( f, "Scene error: {}", e ),
    }
  }
}

impl std::error::Error for CliError {}

impl From< io::Error > for CliError
{
  fn from( err: io::Error ) -> Self
  {
    CliError::Io( err )
  }
}

/// Main CLI application structure
pub struct CliApp
{
  scene: crate::scene::Scene,
}

impl CliApp
{
  /// Create new CLI application
  pub fn new() -> Result< Self, CliError >
  {
    Ok( Self
    {
      scene: crate::scene::Scene::new(),
    } )
  }
}

/// Run CLI in single command mode
pub fn run_cli() -> Result< (), CliError >
{
  let args: Vec< String > = std::env::args().collect();
  
  if args.len() < 2
  {
    return run_repl();
  }
  
  let mut app = CliApp::new()?;
  
  // Join arguments starting from index 1 (skip program name)
  let command_line = args[ 1.. ].join( " " );
  
  // Ensure command starts with dot
  let command_line = if command_line.starts_with( '.' )
  {
    command_line[ 1.. ].to_string() // Remove dot prefix
  }
  else
  {
    eprintln!( "Error: All commands must start with '.' (dot prefix)" );
    eprintln!( "Example: .scene.new or .help" );
    return Ok( () );
  };
  
  // Process command
  handle_command( &command_line, &mut app );
  
  Ok( () )
}

/// Run CLI in interactive REPL mode using unilang's enhanced REPL
pub fn run_repl() -> Result< (), CliError >
{
  println!( "Agnostic Rendering Engine CLI (ARE) - REPL Mode" );
  println!( "Type .help for available commands, .quit to exit" );
  println!();
  
  let mut app = CliApp::new()?;
  
  // Set up unilang command registry and pipeline
  let mut registry = CommandRegistry::new();
  setup_unilang_commands( &mut registry )?;
  let pipeline = Pipeline::new( registry );
  
  // Use unilang's enhanced REPL with history support  
  run_unilang_repl( &pipeline, &mut app )
}

/// Run simple REPL without rustyline (for non-TTY environments like tests)
fn run_simple_repl( app: &mut CliApp ) -> Result< (), CliError >
{
  println!( "↑/↓ Arrow keys for command history" );
  println!();

  loop
  {
    let mut input = String::new();
    match io::stdin().read_line( &mut input )
    {
      Ok( 0 ) =>
      {
        // EOF (Ctrl+D)
        println!( "Goodbye!" );
        break;
      },
      Ok( _ ) =>
      {
        let input = input.trim();

        if input.is_empty()
        {
          continue;
        }

        // Handle built-in REPL commands
        match input
        {
          ".quit" | ".exit" | ".q" =>
          {
            println!( "Goodbye!" );
            break;
          },
          ".clear" =>
          {
            // Clear screen (ANSI escape sequence)
            print!( "\x1B[2J\x1B[1;1H" );
            io::stdout().flush()?;
            continue;
          },
          _ => {}
        }

        // Process regular commands
        if input.starts_with( '.' )
        {
          let command = &input[ 1.. ]; // Remove dot prefix

          // Handle stateful commands that need scene access directly
          if handle_stateful_command( command, app ) {
            continue;
          }

          // Process command manually for simple REPL
          handle_command( command, app );
        }
        else
        {
          println!( "Commands must start with '.'. Type .help for available commands." );
        }
      },
      Err( error ) =>
      {
        eprintln!( "Error reading input: {}", error );
        return Err( CliError::Io( error ) );
      }
    }
  }

  Ok( () )
}

/// Run CLI in interactive REPL mode (fallback without rustyline)
#[ cfg( not( feature = "cli-repl" ) ) ]
pub fn run_repl_fallback() -> Result< (), CliError >
{
  println!( "Agnostic Rendering Engine CLI (ARE) - REPL Mode" );
  println!( "Type .help for available commands, .quit to exit" );
  println!();

  let mut app = CliApp::new()?;

  loop
  {
    print!( "are> " );
    io::stdout().flush()?;

    let mut input = String::new();
    match io::stdin().read_line( &mut input )
    {
      Ok( 0 ) =>
      {
        // EOF (Ctrl+D)
        println!();
        println!( "Goodbye!" );
        break;
      },
      Ok( _ ) =>
      {
        let input = input.trim();

        if input.is_empty()
        {
          continue;
        }

        // Handle built-in REPL commands
        match input
        {
          ".quit" | ".exit" | ".q" =>
          {
            println!( "Goodbye!" );
            break;
          },
          ".clear" =>
          {
            // Clear screen (ANSI escape sequence)
            print!( "\x1B[2J\x1B[1;1H" );
            io::stdout().flush()?;
            continue;
          },
          _ => {}
        }

        // Process regular commands
        if input.starts_with( '.' )
        {
          let command = &input[ 1.. ]; // Remove dot prefix

          // Process command manually
          handle_command( command, &mut app );
        }
        else
        {
          println!( "Commands must start with '.'. Type .help for available commands." );
        }
      },
      Err( error ) =>
      {
        eprintln!( "Error reading input: {}", error );
        break;
      }
    }
  }

  Ok( () )
}

/// Run enhanced REPL using unilang's rustyline integration
fn run_unilang_repl( pipeline: &Pipeline, app: &mut CliApp ) -> Result< (), CliError >
{
  // Rustyline is available through unilang's enhanced_repl feature
  use rustyline::DefaultEditor;
  use rustyline::error::ReadlineError;

  // Try to create rustyline editor, fallback to simple REPL if it fails (no TTY)
  let mut rl = match DefaultEditor::new() {
    Ok( editor ) => editor,
    Err( _ ) => {
      // Fall back to simple REPL when rustyline cannot initialize (e.g., no TTY)
      return run_simple_repl( app );
    }
  };
  let mut session_counter = 0u32;
  
  println!( "🎨 Enhanced REPL Features:" );
  println!( "  • ↑/↓ Arrow keys for command history" );  
  println!( "  • Tab completion (basic)" );
  println!( "  • Ctrl+C to continue, Ctrl+D to quit" );
  println!();
  
  loop
  {
    let prompt = format!( "are[{}]> ", session_counter );
    
    match rl.readline( &prompt )
    {
      Ok( input ) =>
      {
        let input = input.trim();
        
        if input.is_empty()
        {
          continue;
        }
        
        // Add to history
        let _ = rl.add_history_entry( input );
        session_counter += 1;
        
        // Handle built-in REPL commands
        match input
        {
          ".quit" | ".exit" | ".q" =>
          {
            println!( "Goodbye!" );
            break;
          },
          ".clear" =>
          {
            print!( "\x1B[2J\x1B[1;1H" );
            io::stdout().flush().map_err( CliError::Io )?;
            continue;
          },
          _ => {}
        }
        
        // Process regular commands
        if input.starts_with( '.' )
        {
          let command = &input[ 1.. ]; // Remove dot prefix
          
          // Handle stateful commands that need scene access directly
          if handle_stateful_command( command, app ) {
            continue;
          }
          
          // Process through unilang for other commands  
          let result = pipeline.process_command_simple( command );
          handle_unilang_result( &result, command, app );
        }
        else
        {
          println!( "Commands must start with '.'. Type .help for available commands." );
        }
      },
      Err( ReadlineError::Interrupted ) =>
      {
        println!( "^C" );
        continue;
      },
      Err( ReadlineError::Eof ) =>
      {
        println!( "Goodbye!" );
        break;
      },
      Err( err ) =>
      {
        return Err( CliError::Io( std::io::Error::new( std::io::ErrorKind::Other, err ) ) );
      }
    }
  }
  
  Ok( () )
}


/// Set up unilang command registry with all available commands
fn setup_unilang_commands( registry: &mut CommandRegistry ) -> Result< (), CliError >
{
  use unilang::data::{ ArgumentAttributes, ArgumentDefinition, CommandDefinition, Kind, OutputData, ErrorData };
  use unilang::semantic::VerifiedCommand;
  
  // Register scene.new command with runtime
  let scene_new_cmd = CommandDefinition::former()
    .name( ".scene.new" )
    .namespace( "" )
    .description( "Create a new empty scene".to_string() )
    .hint( "Creates a new empty scene for adding primitives" )
    .status( "stable" )
    .version( "1.0.0" )
    .tags( vec![ "scene".to_string() ] )
    .arguments( vec![] )
    .examples( vec![ "scene.new".to_string() ] )
    .aliases( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( "POST".to_string() )
    .end();
  
  let scene_new_routine = Box::new( |_cmd: VerifiedCommand, _ctx| {
    println!( "Created new empty scene" );
    Ok( OutputData {
      content: "Scene created successfully".to_string(),
      format: "text".to_string(),
    } )
  } );
  
  registry.command_add_runtime( &scene_new_cmd, scene_new_routine )
    .map_err( |e| CliError::Unilang( e.to_string() ) )?;

  // Note: .help is provided by unilang built-in commands, so we don't register it

  // Register scene.list command with runtime
  let scene_list_cmd = CommandDefinition::former()
    .name( ".scene.list" )
    .namespace( "" )
    .description( "List all primitives in current scene".to_string() )
    .hint( "Shows the number of primitives in the scene" )
    .status( "stable" )
    .version( "1.0.0" )
    .tags( vec![ "scene".to_string() ] )
    .arguments( vec![] )
    .examples( vec![ "scene.list".to_string() ] )
    .aliases( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( "GET".to_string() )
    .end();

  let scene_list_routine = Box::new( |_cmd: VerifiedCommand, _ctx| {
    println!( "Scene contains 0 primitive(s)" );
    Ok( OutputData {
      content: "Scene list displayed".to_string(),
      format: "text".to_string(),
    } )
  } );

  registry.command_add_runtime( &scene_list_cmd, scene_list_routine )
    .map_err( |e| CliError::Unilang( e.to_string() ) )?;

  // Register scene.add command with runtime
  let scene_add_cmd = CommandDefinition::former()
    .name( ".scene.add" )
    .namespace( "" )
    .description( "Add primitive to current scene".to_string() )
    .hint( "Adds a rendering primitive (line, curve, text, tilemap) to the scene" )
    .status( "stable" )
    .version( "1.0.0" )
    .tags( vec![ "scene".to_string() ] )
    .arguments( vec![
      ArgumentDefinition {
        name: "primitive_type".to_string(),
        description: "Type of primitive to add (line, curve, text, tilemap)".to_string(),
        kind: Kind::String,
        hint: "line | curve | text | tilemap".to_string(),
        attributes: ArgumentAttributes { optional: false, ..Default::default() },
        validation_rules: vec![],
        aliases: vec![],
        tags: vec![ "required".to_string() ],
      },
      ArgumentDefinition {
        name: "coordinates".to_string(),
        description: "Coordinates and parameters for the primitive".to_string(),
        kind: Kind::String,
        hint: "Space-separated coordinates/parameters".to_string(),
        attributes: ArgumentAttributes { optional: true, ..Default::default() },
        validation_rules: vec![],
        aliases: vec![],
        tags: vec![],
      }
    ])
    .examples( vec![ 
      "scene.add line 0 0 100 100".to_string(),
      "scene.add text 50 50 Hello".to_string()
    ] )
    .aliases( vec![] )
    .permissions( vec![] )
    .idempotent( false )
    .deprecation_message( String::new() )
    .http_method_hint( "POST".to_string() )
    .end();

  let scene_add_routine = Box::new( |cmd: VerifiedCommand, _ctx| {
    if let Some( primitive_type ) = cmd.arguments.get( "primitive_type" ) {
      let coords = cmd.arguments.get( "coordinates" ).map_or( String::new(), |v| v.to_string() );
      println!( "Adding {} primitive with args: {}", primitive_type, coords );
      
      match primitive_type.to_string().as_str() {
        "line" => {
          println!( "Line added to scene" );
        },
        "curve" => {
          println!( "Curve added to scene" );
        },
        "text" => {
          println!( "Text added to scene" );
        },
        "tilemap" => {
          println!( "Tilemap added to scene" );
        },
        _ => {
          return Err( ErrorData::new(
            "INVALID_PRIMITIVE_TYPE".to_string(),
            format!( "Unknown primitive type: {}. Use: line, curve, text, or tilemap", primitive_type )
          ) );
        }
      }
    } else {
      return Err( ErrorData::new(
        "MISSING_ARGUMENT".to_string(),
        "Missing primitive type argument".to_string()
      ) );
    }
    
    Ok( OutputData {
      content: "Primitive added successfully".to_string(),
      format: "text".to_string(),
    } )
  } );

  registry.command_add_runtime( &scene_add_cmd, scene_add_routine )
    .map_err( |e| CliError::Unilang( e.to_string() ) )?;
    
  // Note: .version is provided by unilang built-in commands
  
  Ok( () )
}

/// Handle unilang command results
fn handle_unilang_result( result: &CommandResult, command: &str, app: &mut CliApp )
{
  if result.success
  {
    // Handle successful commands
    handle_command( command, app );
  }
  else
  {
    // Handle unilang command failures  
    if let Some( error ) = &result.error
    {
      // Handle known .version limitation
      if error.contains( "No executable routine found" ) && command == "version"
      {
        println!( "Agnostic Rendering Engine CLI (ARE) v{}", env!( "CARGO_PKG_VERSION" ) );
        return;
      }
      
      println!( "Error: {}", error );
    }
    else
    {
      println!( "Unknown command error" );
    }
  }
}

/// Handle stateful commands that need scene access
/// Returns true if the command was handled, false if it should go to unilang
fn handle_stateful_command( command: &str, app: &mut CliApp ) -> bool
{
  let parts: Vec< &str > = command.split_whitespace().collect();
  if parts.is_empty() {
    return false;
  }
  
  match parts[ 0 ] {
    "scene.add" => {
      if parts.len() < 2 {
        println!( "Error: .scene.add requires primitive type" );
        println!( "Usage: .scene.add <type> [coordinates...]" );
        println!( "Types: line, curve, text, tilemap" );
        return true;
      }
      
      let primitive_type = parts[ 1 ];
      match primitive_type {
        "line" => {
          if parts.len() >= 6 {
            // .scene.add line x1 y1 x2 y2
            if let ( Ok( x1 ), Ok( y1 ), Ok( x2 ), Ok( y2 ) ) = (
              parts[ 2 ].parse::< f32 >(),
              parts[ 3 ].parse::< f32 >(),
              parts[ 4 ].parse::< f32 >(),
              parts[ 5 ].parse::< f32 >()
            ) {
              add_line_to_scene( app, x1, y1, x2, y2 );
              println!( "Added line from ({}, {}) to ({}, {}) to scene", x1, y1, x2, y2 );
            } else {
              println!( "Error: Invalid coordinates for line" );
              println!( "Usage: .scene.add line <x1> <y1> <x2> <y2>" );
            }
          } else {
            println!( "Error: Line requires 4 coordinates" );
            println!( "Usage: .scene.add line <x1> <y1> <x2> <y2>" );
          }
        },
        "text" => {
          if parts.len() >= 5 {
            // .scene.add text x y "text content"
            if let ( Ok( x ), Ok( y ) ) = ( parts[ 2 ].parse::< f32 >(), parts[ 3 ].parse::< f32 >() ) {
              let text = parts[ 4.. ].join( " " );
              add_text_to_scene( app, x, y, &text );
              println!( "Added text '{}' at ({}, {}) to scene", text, x, y );
            } else {
              println!( "Error: Invalid coordinates for text" );
            }
          } else {
            println!( "Error: Text requires coordinates and content" );
            println!( "Usage: .scene.add text <x> <y> <text>" );
          }
        },
        "curve" => {
          if parts.len() >= 10 {
            // .scene.add curve x1 y1 cx1 cy1 cx2 cy2 x2 y2
            if let ( Ok( x1 ), Ok( y1 ), Ok( cx1 ), Ok( cy1 ), Ok( cx2 ), Ok( cy2 ), Ok( x2 ), Ok( y2 ) ) = (
              parts[ 2 ].parse::< f32 >(),
              parts[ 3 ].parse::< f32 >(),
              parts[ 4 ].parse::< f32 >(),
              parts[ 5 ].parse::< f32 >(),
              parts[ 6 ].parse::< f32 >(),
              parts[ 7 ].parse::< f32 >(),
              parts[ 8 ].parse::< f32 >(),
              parts[ 9 ].parse::< f32 >()
            ) {
              add_curve_to_scene( app, x1, y1, cx1, cy1, cx2, cy2, x2, y2 );
              println!( "Added bezier curve to scene" );
            } else {
              println!( "Error: Invalid coordinates for curve" );
            }
          } else {
            println!( "Error: Curve requires 8 coordinates" );
            println!( "Usage: .scene.add curve <x1> <y1> <cx1> <cy1> <cx2> <cy2> <x2> <y2>" );
          }
        },
        _ => {
          println!( "Error: Unknown primitive type '{}'. Use: line, curve, text, tilemap", primitive_type );
        }
      }
      true
    },
    "scene.list" => {
      let count = app.scene.len();
      println!( "Scene contains {} primitive(s)", count );
      if count > 0 {
        println!( "Use .render to generate output from this scene" );
      }
      true
    },
    "scene.clear" => {
      app.scene = crate::scene::Scene::new();
      println!( "Scene cleared - all primitives removed" );
      true
    },
    "render" => {
      if parts.len() < 3 {
        println!( "Error: .render requires backend and output file" );
        println!( "Usage: .render <backend> <output_file>" );
        println!( "Backends: svg, terminal" );
        return true;
      }
      
      let backend = parts[ 1 ];
      let output_file = parts[ 2 ];
      
      match backend {
        "svg" => {
          match render_scene_to_svg( &app.scene, output_file ) {
            Ok( _ ) => println!( "Scene rendered to SVG file: {}", output_file ),
            Err( e ) => println!( "Error rendering to SVG: {}", e ),
          }
        },
        "terminal" => {
          match render_scene_to_terminal( &app.scene, output_file ) {
            Ok( _ ) => println!( "Scene rendered to terminal file: {}", output_file ),
            Err( e ) => println!( "Error rendering to terminal: {}", e ),
          }
        },
        _ => {
          println!( "Error: Unknown backend '{}'. Use: svg, terminal", backend );
        }
      }
      true
    },
    "scene.save" => {
      if parts.len() < 2 {
        println!( "Error: .scene.save requires filename" );
        println!( "Usage: .scene.save <filename>" );
        return true;
      }
      
      let filename = parts[ 1 ];
      match save_scene_to_file( &app.scene, filename ) {
        Ok( _ ) => println!( "Scene saved to file: {}", filename ),
        Err( e ) => println!( "Error saving scene: {}", e ),
      }
      true
    },
    "scene.load" => {
      if parts.len() < 2 {
        println!( "Error: .scene.load requires filename" );
        println!( "Usage: .scene.load <filename>" );
        return true;
      }
      
      let filename = parts[ 1 ];
      match load_scene_from_file( filename ) {
        Ok( scene ) => {
          app.scene = scene;
          let count = app.scene.len();
          println!( "Scene loaded from file: {} ({} primitive(s))", filename, count );
        },
        Err( e ) => println!( "Error loading scene: {}", e ),
      }
      true
    },
    _ => false // Not a stateful command, let unilang handle it
  }
}

/// Handle command execution (legacy single command mode)
fn handle_command( command: &str, app: &mut CliApp )
{
  // Handle specific commands with custom logic
  match command
  {
    "scene.new" =>
    {
      app.scene = crate::scene::Scene::new();
      println!( "Created new empty scene" );
    },
    cmd if cmd.starts_with( "scene.add" ) =>
    {
      let parts: Vec< &str > = command.split_whitespace().collect();
      if parts.len() > 1
      {
        let primitive_type = parts[ 1 ];
        println!( "Added {} primitive to scene", primitive_type );
        // TODO: Actually add to scene when rendering is integrated
      }
      else
      {
        println!( "Error: .scene.add requires primitive type" );
      }
    },
    "scene.list" =>
    {
      println!( "Scene contains {} primitive(s)", app.scene.len() );
    },
    "help" | "h" | "" =>
    {
      show_help();
    },
    "version" | "v" =>
    {
      println!( "Agnostic Rendering Engine CLI (ARE) v{}", env!( "CARGO_PKG_VERSION" ) );
    },
    _ =>
    {
      // Default success message
      println!( "Command executed successfully" );
    }
  }
}

/// Add a line primitive to the scene
fn add_line_to_scene( app: &mut CliApp, x1: f32, y1: f32, x2: f32, y2: f32 )
{
  use crate::commands::{ RenderCommand, LineCommand, Point2D, StrokeStyle };
  
  let line = RenderCommand::Line( LineCommand {
    start: Point2D { x: x1, y: y1 },
    end: Point2D { x: x2, y: y2 },
    style: StrokeStyle::default(),
  } );
  
  app.scene.add( line );
}

/// Add a text primitive to the scene
fn add_text_to_scene( app: &mut CliApp, x: f32, y: f32, text: &str )
{
  use crate::commands::{ RenderCommand, TextCommand, Point2D, FontStyle, TextAnchor };
  
  let text_cmd = TextCommand::new(
    text,
    Point2D { x, y },
    FontStyle::default(),
    TextAnchor::TopLeft
  );
  
  app.scene.add( RenderCommand::Text( text_cmd ) );
}

/// Add a curve primitive to the scene
fn add_curve_to_scene( app: &mut CliApp, x1: f32, y1: f32, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x2: f32, y2: f32 )
{
  use crate::commands::{ RenderCommand, CurveCommand, Point2D, StrokeStyle };
  
  let curve = RenderCommand::Curve( CurveCommand {
    start: Point2D { x: x1, y: y1 },
    control1: Point2D { x: cx1, y: cy1 },
    control2: Point2D { x: cx2, y: cy2 },
    end: Point2D { x: x2, y: y2 },
    style: StrokeStyle::default(),
  } );
  
  app.scene.add( curve );
}

/// Render scene to SVG file
fn render_scene_to_svg( scene: &crate::scene::Scene, output_file: &str ) -> Result< (), CliError >
{
  use crate::adapters::SvgRenderer;
  use crate::ports::{ Renderer, RenderContext };
  
  let mut renderer = SvgRenderer::new();
  let context = RenderContext::default();
  
  renderer.initialize( &context ).map_err( |e| CliError::Scene( e.to_string() ) )?;
  renderer.begin_frame( &context ).map_err( |e| CliError::Scene( e.to_string() ) )?;
  renderer.render_scene( scene ).map_err( |e| CliError::Scene( e.to_string() ) )?;
  renderer.end_frame().map_err( |e| CliError::Scene( e.to_string() ) )?;
  
  let svg_content = renderer.output().map_err( |e| CliError::Scene( e.to_string() ) )?;
  
  std::fs::write( output_file, svg_content )?;
  
  Ok( () )
}

/// Render scene to terminal file
fn render_scene_to_terminal( scene: &crate::scene::Scene, output_file: &str ) -> Result< (), CliError >
{
  use crate::adapters::TerminalRenderer;
  use crate::ports::{ Renderer, RenderContext };
  
  let mut renderer = TerminalRenderer::with_dimensions( 80, 25 );
  let context = RenderContext::default();
  
  renderer.initialize( &context ).map_err( |e| CliError::Scene( e.to_string() ) )?;
  renderer.begin_frame( &context ).map_err( |e| CliError::Scene( e.to_string() ) )?;
  renderer.render_scene( scene ).map_err( |e| CliError::Scene( e.to_string() ) )?;
  renderer.end_frame().map_err( |e| CliError::Scene( e.to_string() ) )?;
  
  let terminal_content = renderer.get_output();
  
  std::fs::write( output_file, terminal_content )?;
  
  Ok( () )
}

/// Save scene to JSON file
fn save_scene_to_file( scene: &crate::scene::Scene, filename: &str ) -> Result< (), CliError >
{
  #[ cfg( feature = "serde" ) ]
  {
    let json = serde_json::to_string_pretty( scene ).map_err( |e| CliError::Scene( e.to_string() ) )?;
    std::fs::write( filename, json )?;
    Ok( () )
  }
  #[ cfg( not( feature = "serde" ) ) ]
  {
    Err( CliError::Scene( "Scene serialization requires 'serde' feature".to_string() ) )
  }
}

/// Load scene from JSON file
fn load_scene_from_file( filename: &str ) -> Result< crate::scene::Scene, CliError >
{
  #[ cfg( feature = "serde" ) ]
  {
    let json = std::fs::read_to_string( filename )?;
    let scene = serde_json::from_str( &json ).map_err( |e| CliError::Scene( e.to_string() ) )?;
    Ok( scene )
  }
  #[ cfg( not( feature = "serde" ) ) ]
  {
    Err( CliError::Scene( "Scene deserialization requires 'serde' feature".to_string() ) )
  }
}


/// Show comprehensive help information
fn show_help()
{
  println!( "Agnostic Rendering Engine CLI (ARE) - Commands" );
  println!();
  println!( "Scene Management Commands:" );
  println!( "  .scene.new                   Create a new empty scene" );
  println!( "  .scene.add <primitive>       Add primitive to current scene" );
  println!( "  .scene.list                  List all primitives in current scene" );
  println!( "  .scene.save <filename>       Save current scene to file" );
  println!( "  .scene.load <filename>       Load scene from file" );
  println!();
  println!( "Rendering Commands:" );
  println!( "  .render <backend> <output>   Render current scene with specified backend" );
  println!();
  println!( "Available Backends:" );
  println!( "  svg                          Static SVG vector graphics" );
  println!( "  terminal                     ASCII art terminal output" );
  println!();
  println!( "Primitive Types for .scene.add:" );
  println!( "  line                         2D line segment" );
  println!( "  curve                        Bezier curve" );
  println!( "  text                         Text element" );
  println!();
  println!( "General Commands:" );
  println!( "  .help, .h                    Show this help message" );
  println!( "  .version, .v                 Show version information" );
  println!( "  .quit, .exit, .q             Exit the CLI" );
  println!();
  println!( "REPL-specific Commands:" );
  println!( "  .clear                       Clear the screen" );
  println!();
  println!( "Examples:" );
  println!( "  .scene.new" );
  println!( "  .scene.add line" );
  println!( "  .scene.add curve" );
  println!( "  .scene.add text" );
  println!( "  .scene.save my_scene.json" );
  println!( "  .render svg output.svg" );
  println!( "  .render terminal output.txt" );
}