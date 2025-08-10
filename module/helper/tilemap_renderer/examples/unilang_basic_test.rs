//! Basic unilang compilation test

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::unnecessary_wraps ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::std_instead_of_core ) ]

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  #[ cfg( feature = "cli" ) ]
  {
    use unilang::*;
    
    println!( "Testing basic unilang compilation..." );
    
    let registry = CommandRegistry::new();
    println!( "✓ CommandRegistry created" );
    
    let _pipeline = Pipeline::new( registry );
    println!( "✓ Pipeline created" );
    
    println!( "Basic unilang test passed!" );
  }
  
  #[ cfg( not( feature = "cli" ) ) ]
  {
    println!( "CLI feature not enabled" );
  }
  
  Ok( () )
}