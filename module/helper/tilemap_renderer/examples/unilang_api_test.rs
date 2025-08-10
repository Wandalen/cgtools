//! Test to explore unilang API capabilities including REPL methods.

#[ cfg( feature = "cli" ) ]
fn main() -> Result< (), Box< dyn std::error::Error > >
{
  use unilang::*;
  
  println!( "Testing unilang API capabilities..." );
  
  let registry = CommandRegistry::new();
  let pipeline = Pipeline::new( registry );
  
  // Check what methods are available on Pipeline
  println!( "Pipeline methods:" );
  
  // Try process_command_simple - this should exist
  let result = pipeline.process_command_simple( "help" );
  println!( "process_command_simple result: success={}, outputs={:?}", result.success, result.outputs );
  
  // Check if there are any REPL-related methods by trying to call them
  // (This will cause compile errors if they don't exist, which is what we want to see)
  
  println!( "Checking for REPL methods..." );
  
  // Try to see if these methods exist (will cause compile error if not):
  // Uncomment one at a time to test:
  // let _repl = pipeline.start_repl(); // CONFIRMED: doesn't exist
  // let _interactive = pipeline.run_interactive(); // CONFIRMED: doesn't exist
  
  // Check if there's a REPL struct or module:
  // let _repl = unilang::Repl::new(); // CONFIRMED: doesn't exist
  
  println!( "Testing complete - no REPL methods found to test" );
  
  println!( "Basic unilang test completed" );
  
  Ok( () )
}

#[ cfg( not( feature = "cli" ) ) ]
fn main()
{
  println!( "CLI feature not enabled" );
}