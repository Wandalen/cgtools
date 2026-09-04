//! Verifies `uniform`'s `f32_matrix_length_error` -- the error `UniformMatrixUpload::matrix_upload`
//! ( for `[ f32 ]`/`[ f32 ; N ]` in `src/uniform/float32.rs` ) constructs when the flat data length
//! doesn't match a supported square-matrix size. Extracted into its own function purely so its
//! message content is unit-testable without a live `GL` -- `matrix_upload` itself takes `&GL`,
//! which can't be constructed outside a browser. Relocated to `tests/` per the all-tests-in-tests/
//! convention; the helper is exported at the `uniform` module path for exactly this purpose.

use minwebgl::{ uniform::f32_matrix_length_error, uniform::vector_upload_length_error, WebglError };

// test_kind: bug_reproducer(BUG-277)
/// ## Root Cause
/// `UniformMatrixUpload::matrix_upload` for both `[ f32 ]` and `[ f32 ; N ]`
/// ( `src/uniform/float32.rs` ) built their "unsupported length" error by copy-pasting
/// `UniformUpload::upload`'s vector-length error arm verbatim: item kind `"vector"` and
/// known-lengths string `"1, 2, 3, 4"` -- both wrong for a matrix upload, whose only valid flat
/// lengths are 4, 9, 16 ( 2x2, 3x3, 4x4 ). A caller passing, say, a 3- or 6-element slice to
/// `matrix_upload` got an error claiming the upload was a "vector" and that valid lengths were
/// "1, 2, 3, 4" -- actively misleading, since 3 IS one of those claimed-valid lengths yet was
/// still rejected.
///
/// ## Why Not Caught
/// `matrix_upload` takes `&GL` ( `web_sys::WebGl2RenderingContext` ), which can't be constructed
/// outside a browser, so nothing could call it directly from a native `cargo test` run to observe
/// the error text; no live-GL example in this repo exercises the error branch either ( every real
/// caller passes an already-correctly-sized matrix ).
///
/// ## Fix Applied
/// Extracted the error construction into `f32_matrix_length_error( type_name, len )`, a pure
/// function with no `GL` dependency, returning
/// `WebglError::CantUploadUniform( "matrix", type_name, len, "4, 9, 16" )`. Both `matrix_upload`
/// impls' catch-all arms now call it instead of constructing the error inline.
///
/// ## Prevention
/// RED state (empirically confirmed): reverting `f32_matrix_length_error`'s body to
/// `WebglError::CantUploadUniform( "vector", type_name, len, "1, 2, 3, 4" )` and re-running this
/// test genuinely fails both the item-kind and known-lengths assertions below — verified via a
/// temporary probe before this fix was finalized.
///
/// ## Pitfall
/// `WebglError::CantUploadUniform`'s constant string arguments have no compiler-enforced link to
/// the match arm they describe -- copy-pasting a sibling match arm's error branch silently carries
/// over a stale, misleading message that only a reader ( or a test asserting on message content )
/// catches.
#[ test ]
fn f32_matrix_length_error_reports_matrix_not_vector()
{
  for len in [ 0usize, 1, 2, 3, 5, 6, 8, 10, 17, 25 ]
  {
    let error = f32_matrix_length_error( "&[f32]", len );
    let WebglError::CantUploadUniform( kind, type_name, reported_len, known ) = error
    else
    {
      panic!( "expected WebglError::CantUploadUniform, got {error:?}" );
    };
    assert_eq!( kind, "matrix", "item kind must be \"matrix\", not the vector error's \"vector\" (len {len})" );
    assert_eq!( type_name, "&[f32]" );
    assert_eq!( reported_len, len );
    assert_eq!( known, "4, 9, 16", "known-lengths field must list the matrix sizes, not the vector error's \"1, 2, 3, 4\" (len {len})" );
  }
}

/// Companion check: the message rendered via `Display` actually contains the corrected fields --
/// pins the fix at the user-visible error text, not just the enum's internal tuple fields.
#[ test ]
fn f32_matrix_length_error_display_mentions_matrix_and_valid_lengths()
{
  let message = f32_matrix_length_error( "&[f32; 5]", 5 ).to_string();
  assert!( message.contains( "matrix" ), "message must mention \"matrix\", got: {message}" );
  assert!( message.contains( "4, 9, 16" ), "message must list valid lengths 4, 9, 16, got: {message}" );
  assert!( !message.contains( "1, 2, 3, 4" ), "message must not carry over the vector error's valid-lengths list, got: {message}" );
}

// test_kind: bug_reproducer(BUG-426)
/// ## Root Cause
/// `UniformUpload::upload` for `[ [ f32 ; N ] ]`/`[ [ i32 ; N ] ]`/`[ [ u32 ; N ] ]`
/// ( `src/uniform/float32.rs`, `int32.rs`, `unsigned32.rs` ) all `match N { .. }` to pick the
/// right `glUniformNfv`-family call, but their catch-all `_` arm reported `self.len()` -- the
/// *outer* slice's element count, i.e. how many `[ T ; N ]` vectors were passed -- as the
/// invalid "length" in the error, instead of `N`, the *inner* array's own arity that the
/// surrounding `match` is actually on. Uploading `&[ [ f32 ; 5 ] ; 3 ]` ( 3 vectors, each of
/// the unsupported arity 5 ) produced a self-contradictory message reading
/// "...of length 3. Known length : [ 1, 2, 3, 4 ]" -- 3 IS in the claimed-valid list, because
/// the field reported was never the field that actually failed validation.
///
/// ## Why Not Caught
/// `UniformUpload::upload` takes `&GL` ( `web_sys::WebGl2RenderingContext` ), which can't be
/// constructed outside a browser, so nothing could call it directly from a native `cargo test`
/// run to observe the error text; no live-GL example in this repo exercises the error branch
/// either ( every real caller passes an already-correctly-sized vector array ).
///
/// ## Fix Applied
/// Extracted the error construction into `vector_upload_length_error( type_name, n )`, a pure
/// function with no `GL` dependency, returning
/// `WebglError::CantUploadUniform( "vector", type_name, n, "1, 2, 3, 4" )`. All three `[ [ T ; N ] ]`
/// `upload` impls' catch-all arms now call it with `N` ( not `self.len()` ) instead of
/// constructing the error inline.
///
/// ## Prevention
/// RED state (empirically confirmed): reverting any of the three call sites back to
/// `Err( vector_upload_length_error( type_name_of_val( self ), self.len() ) )` ( i.e. `self.len()`
/// instead of `N` ) and re-running this test with an outer/inner length mismatch genuinely fails
/// the `reported_n` assertion below -- verified via a temporary probe before this fix was
/// finalized.
///
/// ## Pitfall
/// A slice-of-arrays impl ( matches on the const generic `N` ) and a plain-slice impl ( matches
/// on `self.len()` ) can share an identically-shaped error arm while differing in exactly which
/// value the `match` scrutinee -- and therefore the error's "invalid length" field -- actually
/// is; copy-pasting one into the other silently keeps the wrong field.
#[ test ]
fn vector_upload_length_error_reports_inner_arity_not_outer_len()
{
  // `n` here stands in for the inner array arity `N` a `[ [ T ; N ] ]` impl's `_` arm is
  // reached with ( always `> 4` or `0`, since 1..=4 are handled by their own match arms
  // before the catch-all ); the outer slice length is deliberately different from `n` in
  // every case, so a regression back to reporting `self.len()` cannot coincidentally match.
  for ( type_name, n ) in [ ( "&[[f32; 5]]", 5usize ), ( "&[[f32; 0]]", 0 ), ( "&[[f32; 6]]", 6 ) ]
  {
    let error = vector_upload_length_error( type_name, n );
    let WebglError::CantUploadUniform( kind, reported_type, reported_n, known ) = error
    else
    {
      panic!( "expected WebglError::CantUploadUniform, got {error:?}" );
    };
    assert_eq!( kind, "vector" );
    assert_eq!( reported_type, type_name );
    assert_eq!( reported_n, n, "reported length must be the inner array arity N, not the outer slice's self.len()" );
    assert_eq!( known, "1, 2, 3, 4" );
  }
}
