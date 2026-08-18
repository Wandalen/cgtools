use super::*;

#[ test ]
fn assumptions()
{
  let align1 = std::mem::align_of::< [ [ u8 ; 3 ] ; 3 ] >();
  let align2 = std::mem::align_of::< [ u8 ; 9 ] >();
  println!( "align : {align1}" );
  assert_eq!( align1, align2, "Same alignment" );

  let size1 = std::mem::size_of::< [ [ u8 ; 3 ] ; 3 ] >();
  let size2 = std::mem::size_of::< [ u8 ; 9 ] >();
  println!( "size : {size1}" );
  assert_eq!( size1, size2, "Same size" );
}

// `scalar_ref` only retrieves an element that was stored verbatim via `row_major_set` —
// no arithmetic occurs, so the result is bit-identical to the original literal.
#[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
fn test_scalar_ref_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 2, 2, f32, D > : the_module::ScalarRef< Scalar = f32 >,
  the_module::Mat< 2, 2, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 2, 2, f32, D > : Default,
  the_module::Mat< 2, 2, f32, D > : the_module::ConstLayout,
  the_module::Mat< 2, 2, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat, Ix2 };

  // Use row_major_set for consistent logical layout regardless of internal storage
  let mat = Mat::< 2, 2, f32, D >::default().row_major_set( &[ 1.0, 2.0, 3.0, 4.0 ] );

  // Test scalar_ref for each element - these should work consistently now
  let scalar = mat.scalar_ref( Ix2( 0, 0 ) );
  let exp = &1.0;
  assert_eq!( scalar, exp, "Expected {exp:?}, got {scalar:?}" );

  let scalar = mat.scalar_ref( Ix2( 0, 1 ) );
  let exp = &2.0;
  assert_eq!( scalar, exp, "Expected {exp:?}, got {scalar:?}" );

  let scalar = mat.scalar_ref( Ix2( 1, 0 ) );
  let exp = &3.0;
  assert_eq!( scalar, exp, "Expected {exp:?}, got {scalar:?}" );

  let scalar = mat.scalar_ref( Ix2( 1, 1 ) );
  let exp = &4.0;
  assert_eq!( scalar, exp, "Expected {exp:?}, got {scalar:?}" );
}

#[ test ]
fn test_scalar_ref_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_scalar_ref_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_scalar_ref_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_scalar_ref_generic::< DescriptorOrderColumnMajor >();
}

fn test_scalar_mut_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat< 3, 3, f32, D > : the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat< 3, 3, f32, D > : the_module::Indexable< Index = the_module::Ix2 >,
  the_module::Mat< 3, 3, f32, D > : Default,
  the_module::Mat< 3, 3, f32, D > : the_module::ConstLayout,
  the_module::Mat< 3, 3, f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat, Ix2 };

  let mut mat = Mat::< 3, 3, f32, D >::default().set_raw( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );

  // Modify a specific element
  let index = Ix2( 2, 2 ); // Access the element at row 2, column 2
  let value = mat.scalar_mut( index );
  *value = 10.0;

  // Verify the modification
  let expected = Mat::< 3, 3, f32, D >::default().set_raw( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0 ] );
  assert_eq!( mat.raw_slice(), expected.raw_slice(), "Modification failed" );
}

#[ test ]
fn test_scalar_mut_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_scalar_mut_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_scalar_mut_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_scalar_mut_generic::< DescriptorOrderColumnMajor >();
}

// test_kind: bug_reproducer(BUG-288)
/// ## Root Cause
/// `ScalarRef::scalar_ref` (`md/access.rs`) and its mirrored inherent wrapper `Mat::scalar_ref`
/// (`d2/mat/access_mirror.rs`) both documented their return value as "A mutable reference,"
/// copy-pasted from the adjacent `ScalarMut::scalar_mut`/`Mat::scalar_mut` methods -- but both
/// take `&self` and return `&Self::Scalar`/`&<Self as Collection>::Scalar` (immutable).
/// ## Why Not Caught
/// `test_scalar_ref_generic` above already exercises `scalar_ref`'s correct immutable-read
/// behavior, but no test read the doc comment text itself -- a doc string carries zero compiler
/// enforcement, so a behaviorally-correct, genuinely-immutable method can carry an arbitrarily
/// wrong "mutable" claim indefinitely with every runtime test still green.
/// ## Fix Applied
/// Reworded both doc comments from "A mutable reference" to "A reference" (`md/access.rs`,
/// `d2/mat/access_mirror.rs`); no behavioral change.
/// ## Prevention
/// When two sibling methods (a `_ref`/`_mut` pair) are documented together, diff each one's doc
/// text against its own receiver (`&self` vs `&mut self`) and return type, not just copy the
/// nearby sibling's wording.
/// ## Pitfall
/// A caller trusting either doc could wrongly assume `scalar_ref` grants write access through a
/// shared reference -- undetectable by any runtime test, since the method's actual behavior was
/// always correctly immutable.
#[ test ]
fn scalar_ref_doc_does_not_claim_mutable()
{
  fn assert_doc_says_reference_not_mutable( src : &str, needle : &str, file : &str )
  {
    let fn_pos = src.find( needle ).unwrap_or_else( || panic!( "{needle:?} not found in {file}" ) );
    let preceding = &src[ ..fn_pos ];
    let doc_line = preceding.lines().rev()
      .find( | line | line.trim_start().starts_with( "///" ) )
      .unwrap_or_else( || panic!( "no doc comment found before {needle:?} in {file}" ) );

    assert!( !doc_line.contains( "mutable" ), "{file}'s scalar_ref doc must not claim a mutable reference (BUG-288), got: {doc_line:?}" );
    assert!( doc_line.contains( "reference" ), "{file}'s scalar_ref doc must describe a reference, got: {doc_line:?}" );
  }

  let trait_src = include_str!( "../../../../src/md/access.rs" );
  assert_doc_says_reference_not_mutable( trait_src, "fn scalar_ref( &self", "md/access.rs" );

  let mirror_src = include_str!( "../../../../src/d2/mat/access_mirror.rs" );
  assert_doc_says_reference_not_mutable( mirror_src, "pub fn scalar_ref( &self", "d2/mat/access_mirror.rs" );
}
