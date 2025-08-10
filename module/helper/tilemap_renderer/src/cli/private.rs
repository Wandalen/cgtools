//! Private CLI implementation using unilang.

use crate::*;
use unilang::*;
use std::io::{ self, Write };

/// CLI application error type
#[ derive( Debug ) ]
pub enum CliError
{
  Io( io::Error ),
  Unilang( String ),
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
  registry: CommandRegistry,
  scene: scene::Scene,
}

impl CliApp
{
  /// Create new CLI application
  pub fn new() -> Result< Self, CliError >
  {
    let mut registry = CommandRegistry::new();
    setup_commands( &mut registry )?;
    
    Ok( Self
    {
      registry,
      scene: scene::Scene::new(),
    } )
  }
  
  /// Get the command registry
  pub fn registry( &self ) -> &CommandRegistry
  {
    &self.registry
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
  let pipeline = Pipeline::new( app.registry );
  
  // Join arguments starting from index 1 (skip program name)
  let command_line = args[ 1.. ].join( " " );
  
  // Ensure command starts with dot
  let command_line = if command_line.starts_with( '.' )
  {
    command_line[ 1.. ].to_string() // Remove dot prefix for unilang
  }
  else
  {
    eprintln!( "Error: All commands must start with '.' (dot prefix)" );
    eprintln!( "Example: .scene.new or .help" );
    return Ok( () );
  };
  
  // Create execution context
  let context = ExecutionContext::default();
  
  // Process command
  let result = pipeline.process_command( &command_line, context );
  
  if result.success
  {
    handle_command_success( &command_line, &result, &mut app );
  }
  else
  {
    eprintln!( "Error executing command: .{}", command_line );
    if let Some( error ) = &result.error
    {
      eprintln!( "Error: {}", error );
    }
  }
  
  Ok( () )
}

/// Run CLI in interactive REPL mode
pub fn run_repl() -> Result< (), CliError >
{
  println!( "Agnostic Rendering Engine CLI (ARE) - REPL Mode" );
  println!( "Type .help for available commands, .quit to exit" );
  println!();
  
  let mut app = CliApp::new()?;
  let pipeline = Pipeline::new( app.registry );
  
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
          let command = &input[ 1.. ]; // Remove dot prefix for unilang
          
          // Create execution context
          let context = ExecutionContext::default();
          
          // Process command
          let result = pipeline.process_command( command, context );
          
          if result.success
          {
            handle_command_success( command, &result, &mut app );
          }
          else
          {
            println!( "Error executing command: {}", input );
            if let Some( error ) = &result.error
            {
              println!( "Error: {}", error );
            }
          }
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

/// Handle successful command execution
fn handle_command_success( command: &str, result: &CommandResult, app: &mut CliApp )
{
  // Handle specific commands with custom logic
  match command
  {
    "scene.new" =>
    {
      app.scene = scene::Scene::new();
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
    "help" | "h" =>
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
  
  // Print any outputs from unilang
  for output in &result.outputs
  {
    println!( "{:?}", output );
  }
}

/// Set up command registry with all available commands
fn setup_commands( registry: &mut CommandRegistry ) -> Result< (), CliError >
{
  // Scene management commands
  registry.register( CommandDefinition
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
  } );
  
  registry.register( CommandDefinition
  {
    name: "scene.add".to_string(),
    namespace: String::new(),
    description: "Add primitive to current scene".to_string(),
    routine_link: None,
    hint: "Adds a new primitive to the current scene".to_string(),
    status: "stable".to_string(),
    version: "1.0.0".to_string(),
    tags: vec![ "scene".to_string() ],
    arguments: vec![ ArgumentDefinition
    {
      name: "primitive_type".to_string(),
      aliases: Vec::new(),
      description: "Type of primitive to add (line, curve, text)".to_string(),
      hint: "Specify the type of primitive".to_string(),
      kind: Kind::String,
      attributes: ArgumentAttributes
      {
        optional: false,
        multiple: false,
        default: None,
        sensitive: false,
        interactive: false,
      },
      tags: Vec::new(),
      validation_rules: Vec::new(),
    } ],
    examples: Vec::new(),
    aliases: Vec::new(),
    permissions: Vec::new(),
    idempotent: false,
    deprecation_message: String::new(),
    http_method_hint: String::new(),
  } );
  
  registry.register( CommandDefinition
  {
    name: "scene.list".to_string(),
    namespace: String::new(),
    description: "List all primitives in current scene".to_string(),
    routine_link: None,
    hint: "Shows all primitives in the current scene".to_string(),
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
  } );
  
  // Utility commands
  registry.register( CommandDefinition
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
  } );
  
  registry.register( CommandDefinition
  {
    name: "version".to_string(),
    namespace: String::new(),
    description: "Show version information".to_string(),
    routine_link: None,
    hint: "Displays version information".to_string(),
    status: "stable".to_string(),
    version: "1.0.0".to_string(),
    tags: vec![ "utility".to_string() ],
    arguments: Vec::new(),
    examples: Vec::new(),
    aliases: vec![ "v".to_string() ],
    permissions: Vec::new(),
    idempotent: true,
    deprecation_message: String::new(),
    http_method_hint: String::new(),
  } );
  
  Ok( () )
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