//! Log tests

#[ test ]
fn manual_setup_test()
{
  use std::panic;
  let config = browser_log::panic::Config::default();
  panic::set_hook( Box::new( move | info | browser_log::panic::hook( info, &config ) ) );
  // panic::set_hook( Box::new( browser_log::panic::hook ) );
}

#[ test ]
fn setpu_test()
{
  browser_log::panic::setup( Default::default() );
  browser_log::panic::setup( Default::default() );
  browser_log::panic::setup( Default::default() );
}
