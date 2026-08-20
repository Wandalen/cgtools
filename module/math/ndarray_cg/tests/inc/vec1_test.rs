use super::*;

use the_module::{ F32x1, I32x1 };

#[ test ]
fn accessor_test()
{
  let v = I32x1::new( 5 );
  assert_eq!( v.x(), 5 );

  let v = F32x1::new( 5.0 );
  // `v` is constructed from the same literal compared against — no arithmetic occurs, so
  // the stored component is bit-identical to the literal.
  #[ expect( clippy::float_cmp, reason = "assertion checks exact expected value; no arithmetic drift is possible and epsilon comparison would weaken it" ) ]
  {
    assert_eq!( v.x(), 5.0 );
  }
}
