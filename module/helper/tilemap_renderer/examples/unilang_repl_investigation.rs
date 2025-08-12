//! Investigation of unilang REPL capabilities
//! This example attempts to discover unilang's built-in REPL and history features

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::unnecessary_wraps ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::uninlined_format_args ) ]

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  #[ cfg( feature = "cli" ) ]
  {
    use unilang::*;
    
    println!( "Investigating unilang REPL capabilities..." );
    
    // Create registry and pipeline
    let registry = CommandRegistry::new();
    println!( "✓ CommandRegistry created" );
    
    let pipeline = Pipeline::new( registry );
    println!( "✓ Pipeline created" );
    
    // Test basic command processing
    println!( "\n=== Testing Command Processing ===" );
    
    let result = pipeline.process_command_simple("help");
    println!( "Command 'help':" );
    println!( "  Success: {}", result.success );
    println!( "  Outputs: {:?}", result.outputs );
    if let Some( error ) = result.error
    {
      println!( "  Error: {}", error );
    }
    
    // Test listing available commands
    let result = pipeline.process_command_simple("");
    println!( "\nCommand '' (list commands):" );
    println!( "  Success: {}", result.success );
    println!( "  Outputs: {:?}", result.outputs );
    if let Some( error ) = result.error
    {
      println!( "  Error: {}", error );
    }
    
    // Test the '.' command mentioned in the error
    let result = pipeline.process_command_simple(".");
    println!( "\nCommand '.' (list commands):" );
    println!( "  Success: {}", result.success );
    println!( "  Outputs: {:?}", result.outputs );
    if let Some( error ) = result.error
    {
      println!( "  Error: {}", error );
    }
    
    println!( "\n=== Testing Available Commands ===" );
    
    // Test the known working command
    let result = pipeline.process_command_simple("version");
    println!( "\nCommand 'version':" );
    println!( "  Success: {}", result.success );
    println!( "  Outputs: {:?}", result.outputs );
    if let Some( error ) = result.error
    {
      println!( "  Error: {}", error );
    }
    
    // Test with help syntax
    let result = pipeline.process_command_simple("version ?");
    println!( "\nCommand 'version ?':" );
    println!( "  Success: {}", result.success );
    println!( "  Outputs: {:?}", result.outputs );
    if let Some( error ) = result.error
    {
      println!( "  Error: {}", error );
    }
    
    println!( "\n=== Understanding REPL Pattern ===" );
    println!( "Based on investigation:" );
    println!( "1. Unilang provides CommandRegistry and Pipeline for command processing" );
    println!( "2. Commands are processed via process_command_simple()" );
    println!( "3. Commands appear to be dot-prefixed (e.g. .version)" );
    println!( "4. Built-in help system with '?' suffix" );
    println!( "5. Command results include success flag, outputs, and optional errors" );
    println!( "6. This provides the foundation for a REPL but does NOT include:" );
    println!( "   - Input reading (no built-in readline)" );
    println!( "   - Command history" );
    println!( "   - Interactive loop management" );
    
    println!( "Unilang REPL investigation complete!" );
  }
  
  #[ cfg( not( feature = "cli" ) ) ]
  {
    println!( "CLI feature not enabled" );
  }
  
  Ok( () )
}