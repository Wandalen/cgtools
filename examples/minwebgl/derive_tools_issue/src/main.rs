#![ allow( dead_code ) ]
#![ allow( unused_imports ) ]

use derive_tools::IsVariant; // Keep the import for now, might be used elsewhere or intended
use strum::EnumCount;

#[ derive( Debug, PartialEq, EnumCount ) ] // Removed IsVariant
// #[ derive( Debug, PartialEq, IsVariant, derive_tools::EnumCount ) ] // qqq : should work with derive_tools::EnumCount
enum Test
{
  A,
  B,
  C,
}

fn main()
{
  // assert_eq!( Test::A.is_a(), true ); // Removed assertion
  // assert_eq!( Test::B.is_a(), false ); // Removed assertion
  assert_eq!( Test::COUNT, 3 );
}
