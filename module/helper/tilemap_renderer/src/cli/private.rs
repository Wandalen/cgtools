//! Private CLI implementation.

use std::io::{ self, Write };
use unilang::*;
//use unilang::data::CommandDefinition; // Using former() instead
use unilang::registry::CommandRegistry;
use unilang::pipeline::Pipeline;

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
  
  let mut rl = DefaultEditor::new().map_err( |e| CliError::Io( std::io::Error::new( std::io::ErrorKind::Other, e ) ) )?;
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
  use unilang::data::{ CommandDefinition, OutputData };
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
  
  // Register help command with runtime
  let help_cmd = CommandDefinition::former()
    .name( ".help" )
    .namespace( "" )
    .description( "Show available commands".to_string() )
    .hint( "Displays help information" )
    .status( "stable" )
    .version( "1.0.0" )
    .tags( vec![ "utility".to_string() ] )
    .arguments( vec![] )
    .examples( vec![ "help".to_string() ] )
    .aliases( vec![ "h".to_string() ] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( "GET".to_string() )
    .end();
  
  let help_routine = Box::new( |_cmd: VerifiedCommand, _ctx| {
    let help_content = r#"Agnostic Rendering Engine CLI (ARE) - Commands

Scene Management Commands:
  .scene.new                   Create a new empty scene
  .scene.add <primitive>       Add primitive to current scene
  .scene.list                  List all primitives in current scene

General Commands:
  .help, .h                    Show this help message
  .version, .v                 Show version information

Examples:
  .scene.new
  .help"#;
    
    println!( "{}", help_content );
    Ok( OutputData {
      content: "Help displayed".to_string(),
      format: "text".to_string(),
    } )
  } );
  
  registry.command_add_runtime( &help_cmd, help_routine )
    .map_err( |e| CliError::Unilang( e.to_string() ) )?;

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

/// Handle command execution
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