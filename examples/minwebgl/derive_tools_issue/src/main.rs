use derive_tools::prelude::*;
use derive_tools::dependency::strum;
use strum::EnumCount;
// use derive_tools::EnumCount;

#[ derive( EnumCount ) ]
enum Test
{
  Variant1,
  Variant2,
  Variant3,
}

fn main() {}

#[ cfg( test ) ]
mod tests
{
  use crate::Test;

  #[ test ]
  fn test()
  {
    assert_eq!( Test::COUNT, 3 );
  }
}
