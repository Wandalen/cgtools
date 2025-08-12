//! Basic CLI Integration Tests
//! Focused subset of the comprehensive test plan for immediate validation

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::uninlined_format_args ) ]

use std::process::Command;
use std::fs;

/// Execute multiple CLI commands via REPL simulation with working directory control
fn execute_cli_sequence_with_output( commands: &[ &str ] ) -> Result< String, Box< dyn std::error::Error > > {
  let mut command_sequence = String::new();
  for cmd in commands {
    command_sequence.push_str( cmd );
    command_sequence.push( '\n' );
  }
  command_sequence.push_str( ".quit\n" );
  
  let output = Command::new( "sh" )
    .arg( "-c" )
    .arg( format!( "cd {} && echo '{}' | cargo run --bin are --features cli", 
                   std::env::current_dir().unwrap().display(),
                   command_sequence ) )
    .output()?;
  
  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );
  
  Ok( format!( "STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr ) )
}

#[ test ]
fn test_basic_cli_help() {
  let result = execute_cli_sequence_with_output( &[ ".help" ] );
  assert!( result.is_ok() );
  let output = result.unwrap();
  
  assert!( output.contains( "Agnostic Rendering Engine CLI" ) );
  assert!( output.contains( ".scene.new" ) );
}

#[ test ]
fn test_basic_scene_operations() {
  let result = execute_cli_sequence_with_output( &[ 
    ".scene.new", 
    ".scene.add line 0 0 100 100",
    ".scene.list"
  ] );
  assert!( result.is_ok() );
  let output = result.unwrap();
  
  assert!( output.contains( "Added line from (0, 0) to (100, 100)" ) );
  assert!( output.contains( "Scene contains 1 primitive(s)" ) );
}

#[ test ]
fn test_basic_file_operations() {
  // Use absolute path in current directory  
  let test_file = std::env::current_dir().unwrap().join( "basic_test_scene.json" );
  let test_file_str = test_file.to_string_lossy();
  
  let result = execute_cli_sequence_with_output( &[ 
    ".scene.new",
    ".scene.add line 10 10 90 90",
    &format!( ".scene.save {}", test_file_str ),
    ".scene.clear",
    ".scene.list",
    &format!( ".scene.load {}", test_file_str ),
    ".scene.list"
  ] );
  
  println!( "Test output: {}", result.as_ref().unwrap_or( &"ERROR".to_string() ) );
  
  assert!( result.is_ok() );
  let output = result.unwrap();
  
  // Check that file operations worked
  assert!( output.contains( "Scene saved to file" ) );
  assert!( output.contains( "Scene contains 0 primitive(s)" ) ); // After clear
  assert!( output.contains( "Scene loaded from file" ) );
  assert!( output.contains( "Scene contains 1 primitive(s)" ) || 
          output.contains( "(1 primitive(s))" ) ); // After load
  
  // Verify file exists  
  assert!( test_file.exists(), "JSON file should exist at {}", test_file_str );
  
  // Cleanup
  let _ = fs::remove_file( test_file );
}

#[ test ]
fn test_basic_rendering() {
  let svg_file = std::env::current_dir().unwrap().join( "basic_test_output.svg" );
  let svg_file_str = svg_file.to_string_lossy();
  
  let result = execute_cli_sequence_with_output( &[ 
    ".scene.new",
    ".scene.add line 0 0 50 50",
    ".scene.add text 25 25 Test",
    &format!( ".render svg {}", svg_file_str ),
  ] );
  
  println!( "Render test output: {}", result.as_ref().unwrap_or( &"ERROR".to_string() ) );
  
  assert!( result.is_ok() );
  let output = result.unwrap();
  
  assert!( output.contains( "Scene rendered to SVG file" ) );
  
  // Verify SVG file exists
  assert!( svg_file.exists(), "SVG file should exist at {}", svg_file_str );
  
  if svg_file.exists() {
    let svg_content = fs::read_to_string( &svg_file ).expect( "Should read SVG file" );
    assert!( svg_content.contains( "<svg" ) );
    assert!( svg_content.contains( "<line" ) );
    assert!( svg_content.contains( "<text" ) );
  }
  
  // Cleanup
  let _ = fs::remove_file( svg_file );
}

#[ test ]
fn test_error_handling() {
  let result = execute_cli_sequence_with_output( &[ 
    ".scene.add",  // Missing args
    ".scene.add invalid_type", // Invalid type
    ".scene.load nonexistent.json", // File not found
  ] );
  
  assert!( result.is_ok() );
  let output = result.unwrap();
  
  assert!( output.contains( "Error:" ) );
  assert!( output.contains( "requires primitive type" ) );
  assert!( output.contains( "Unknown primitive type" ) );
  assert!( output.contains( "No such file or directory" ) );
}