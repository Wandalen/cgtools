//! Test CLI compilation with unilang.

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::std_instead_of_core ) ]

#[ cfg( feature = "cli" ) ]
fn main() -> Result< (), Box< dyn std::error::Error > >
{
  use tilemap_renderer::cli::*;
  
  println!( "CLI test - unilang compilation works!" );
  
  // Create a CLI app to test basic functionality
  let _app = CliApp::new()?;
  
  println!( "CLI app created successfully" );
  
  Ok( () )
}

#[ cfg( not( feature = "cli" ) ) ]
fn main()
{
  println!( "CLI feature not enabled" );
}