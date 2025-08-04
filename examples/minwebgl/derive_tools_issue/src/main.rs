fn main() {}

#[ cfg( test ) ]
mod tests
{
  // use derive_tools::EnumCount;
  use strum::EnumCount;

  #[ derive( EnumCount ) ]
  #[allow(dead_code)]
  enum Test
  {
    Variant1,
    Variant2,
    Variant3,
  }

  #[ test ]
  fn test()
  {
    assert_eq!( Test::COUNT, 3 );
  }
}
