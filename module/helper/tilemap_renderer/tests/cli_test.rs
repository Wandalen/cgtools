//! CLI integration tests verifying all dot commands work properly.

#![ allow( clippy::needless_borrows_for_generic_args ) ]

#[ cfg( feature = "cli" ) ]
mod cli_tests
{
  use std::process::Command;
  
  #[ test ]
  fn test_cli_help_command()
  {
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", ".help" ] )
      .output()
      .expect( "Failed to execute CLI help command" );
      
    assert!( output.status.success(), "CLI help command failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Agnostic Rendering Engine CLI (ARE) - Commands" ) );
    assert!( stdout.contains( ".scene.new" ) );
    assert!( stdout.contains( ".scene.add" ) );
    assert!( stdout.contains( ".scene.list" ) );
    assert!( stdout.contains( ".help" ) );
    assert!( stdout.contains( ".version" ) );
  }
  
  #[ test ]
  fn test_cli_version_command()
  {
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", ".version" ] )
      .output()
      .expect( "Failed to execute CLI version command" );
      
    assert!( output.status.success(), "CLI version command failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Agnostic Rendering Engine CLI (ARE) v0.1.0" ) );
  }
  
  #[ test ]
  fn test_cli_scene_new_command()
  {
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", ".scene.new" ] )
      .output()
      .expect( "Failed to execute CLI scene.new command" );
      
    assert!( output.status.success(), "CLI scene.new command failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Created new empty scene" ) );
  }
  
  #[ test ]
  fn test_cli_scene_add_command()
  {
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", ".scene.add", "line" ] )
      .output()
      .expect( "Failed to execute CLI scene.add command" );
      
    assert!( output.status.success(), "CLI scene.add command failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Added line primitive to scene" ) );
  }
  
  #[ test ]
  fn test_cli_scene_list_command()
  {
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", ".scene.list" ] )
      .output()
      .expect( "Failed to execute CLI scene.list command" );
      
    assert!( output.status.success(), "CLI scene.list command failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Scene contains" ) );
    assert!( stdout.contains( "primitive(s)" ) );
  }
  
  #[ test ]
  fn test_cli_invalid_command_without_dot()
  {
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", "help" ] )
      .output()
      .expect( "Failed to execute CLI invalid command" );
      
    assert!( output.status.success(), "CLI should handle invalid commands gracefully" );
    
    let stderr = String::from_utf8_lossy( &output.stderr );
    assert!( stderr.contains( "All commands must start with '.' (dot prefix)" ) );
    assert!( stderr.contains( "Example: .scene.new or .help" ) );
  }
  
  #[ test ]
  fn test_cli_short_aliases()
  {
    // Test .h alias for .help
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", ".h" ] )
      .output()
      .expect( "Failed to execute CLI .h command" );
      
    assert!( output.status.success(), "CLI .h command failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Agnostic Rendering Engine CLI (ARE) - Commands" ) );
    
    // Test .v alias for .version
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", ".v" ] )
      .output()
      .expect( "Failed to execute CLI .v command" );
      
    assert!( output.status.success(), "CLI .v command failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Agnostic Rendering Engine CLI (ARE) v0.1.0" ) );
  }
  
  #[ test ]
  fn test_cli_dot_command_as_help()
  {
    // Test that single dot "." works as help command
    let output = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are", "--", "." ] )
      .output()
      .expect( "Failed to execute CLI . command" );
      
    assert!( output.status.success(), "CLI . command failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Agnostic Rendering Engine CLI (ARE) - Commands" ) );
    assert!( stdout.contains( ".scene.new" ) );
    assert!( stdout.contains( ".help" ) );
  }
  
  #[ test ]
  fn test_cli_repl_mode_entry()
  {
    // Test that CLI enters REPL mode when no arguments provided
    let mut child = Command::new( "cargo" )
      .args( &[ "run", "--features", "cli", "--bin", "are" ] )
      .stdin( std::process::Stdio::piped() )
      .stdout( std::process::Stdio::piped() )
      .spawn()
      .expect( "Failed to start CLI REPL" );
      
    // Send .quit to exit REPL
    if let Some( stdin ) = child.stdin.take()
    {
      use std::io::Write;
      let mut stdin = stdin;
      writeln!( stdin, ".quit" ).expect( "Failed to write to CLI stdin" );
    }
    
    let output = child.wait_with_output().expect( "Failed to wait for CLI" );
    assert!( output.status.success(), "CLI REPL mode failed" );
    
    let stdout = String::from_utf8_lossy( &output.stdout );
    assert!( stdout.contains( "Agnostic Rendering Engine CLI (ARE) - REPL Mode" ) );
    assert!( stdout.contains( "Type .help for available commands, .quit to exit" ) );
    assert!( stdout.contains( "↑/↓ Arrow keys for command history" ) );
    assert!( stdout.contains( "Goodbye!" ) );
  }
}

#[ cfg( not( feature = "cli" ) ) ]
mod no_cli_tests
{
  #[ test ]
  fn test_cli_feature_not_enabled()
  {
    // This test simply verifies that when CLI feature is not enabled,
    // the test suite still compiles and runs without CLI-specific code
    assert!( true, "CLI feature not enabled - test passes" );
  }
}