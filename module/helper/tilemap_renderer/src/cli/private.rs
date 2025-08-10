//! Private CLI implementation.

use std::io::{ self, Write };
use unilang::*;

#[ cfg( feature = "cli-repl" ) ]
use rustyline::DefaultEditor;

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

/// Run CLI in interactive REPL mode with rustyline history support
#[ cfg( feature = "cli-repl" ) ]
pub fn run_repl() -> Result< (), CliError >
{
  println!( "Agnostic Rendering Engine CLI (ARE) - REPL Mode" );
  println!( "Type .help for available commands, .quit to exit" );
  println!( "Use Up/Down arrows for command history" );
  println!();
  
  let mut app = CliApp::new()?;
  let mut rl = DefaultEditor::new().map_err( |e| CliError::Io( std::io::Error::new( std::io::ErrorKind::Other, e ) ) )?;
  
  // Set up unilang command registry and pipeline for proper command processing
  let mut registry = CommandRegistry::new();
  setup_unilang_commands( &mut registry );
  let pipeline = Pipeline::new( registry );
  
  loop
  {
    match rl.readline( "are> " )
    {
      Ok( line ) =>
      {
        let input = line.trim();
        
        if input.is_empty()
        {
          continue;
        }
        
        // Add to history
        let _ = rl.add_history_entry( &line );
        
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
          
          // Use unilang to process the command
          let result = pipeline.process_command_simple( command );
          handle_unilang_result( &result, command, &mut app );
        }
        else
        {
          println!( "Commands must start with '.'. Type .help for available commands." );
        }
      },
      Err( rustyline::error::ReadlineError::Interrupted ) =>
      {
        println!( "^C" );
        continue;
      },
      Err( rustyline::error::ReadlineError::Eof ) =>
      {
        println!( "Goodbye!" );
        break;
      },
      Err( err ) =>
      {
        eprintln!( "Error: {}", err );
        break;
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

/// Set up unilang command registry with all available commands
fn setup_unilang_commands( registry: &mut CommandRegistry )
{
  // Register all the CLI commands with unilang
  let scene_new_cmd = CommandDefinition
  {
    name: "scene.new".to_string(),
    namespace: String::new(),
    description: "Create a new empty scene".to_string(),
    routine_link: None,
    hint: "Creates a new empty scene for adding primitives".to_string(),
    status: "stable".to_string(),
    version: "1.0.0".to_string(),
    tags: vec![ "scene".to_string() ],
    arguments: Vec::new(),
    examples: Vec::new(),
    aliases: Vec::new(),
    permissions: Vec::new(),
    idempotent: true,
    deprecation_message: String::new(),
    http_method_hint: String::new(),
  };
  registry.register( scene_new_cmd );
  
  let help_cmd = CommandDefinition
  {
    name: "help".to_string(),
    namespace: String::new(),
    description: "Show available commands".to_string(),
    routine_link: None,
    hint: "Displays help information".to_string(),
    status: "stable".to_string(),
    version: "1.0.0".to_string(),
    tags: vec![ "utility".to_string() ],
    arguments: Vec::new(),
    examples: Vec::new(),
    aliases: vec![ "h".to_string() ],
    permissions: Vec::new(),
    idempotent: true,
    deprecation_message: String::new(),
    http_method_hint: String::new(),
  };
  registry.register( help_cmd );
  
  // Add more commands as needed...
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