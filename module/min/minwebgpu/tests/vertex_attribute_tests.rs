//! Tests for `layout::vertex_attribute::format_to_size` -- byte size of a `GpuVertexFormat`.
//!
//! Pure data-in/data-out logic over a `Copy` enum value, no live `GpuDevice` involved -- same
//! wasm32-gated shape as the sibling test files, since minwebgpu's real API only exists under
//! `target_arch = "wasm32"` (see `src/lib.rs`'s native `stub` module).

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgpu as gl;
  use gl::{ GpuVertexFormat, layout::vertex_attribute::format_to_size };

  // test_kind: bug_reproducer(BUG-163)
  /// ## Root Cause
  /// `format_to_size` matched only 31 of `GpuVertexFormat`'s 41 variants and panicked on the
  /// other 10 via a `_ => panic!(...)` catch-all, even though every one of them (`Uint8`,
  /// `Sint8`, `Unorm8`, `Snorm8`, `Uint16`, `Sint16`, `Unorm16`, `Snorm16`, `Float16`,
  /// `Unorm8x4Bgra`) is an ordinary, reachable, spec-defined format a caller can legally
  /// construct and pass via `VertexAttribute::format(..)`.
  /// ## Why Not Caught
  /// The match was built by covering the multi-component (`x2`/`x3`/`x4`) formats only; no
  /// existing test or call site ever passed one of the 10 single-component/`Bgra` formats.
  /// ## Fix Applied
  /// Completed the match with all 10 missing variants at their correct WebGPU-spec byte sizes.
  /// ## Prevention
  /// This test exercises every one of the 10 previously-missing variants directly.
  /// ## Pitfall
  /// `web_sys::GpuVertexFormat` is marked `#[non_exhaustive]` by the `#[wasm_bindgen]` macro
  /// expansion even though its own source lists a closed 41-variant enum -- rustc's E0004 still
  /// demands a `_` arm after all 41 named variants are covered (kept as an internal-invariant
  /// `unreachable!()`, not a silent wrong-size fallback: see `format_to_size`'s own doc comment).
  #[ wasm_bindgen_test ]
  fn previously_missing_variants_no_longer_panic_test()
  {
    assert_eq!( format_to_size( GpuVertexFormat::Uint8 ), 1, "Uint8 must be 1 byte" );
    assert_eq!( format_to_size( GpuVertexFormat::Sint8 ), 1, "Sint8 must be 1 byte" );
    assert_eq!( format_to_size( GpuVertexFormat::Unorm8 ), 1, "Unorm8 must be 1 byte" );
    assert_eq!( format_to_size( GpuVertexFormat::Snorm8 ), 1, "Snorm8 must be 1 byte" );
    assert_eq!( format_to_size( GpuVertexFormat::Uint16 ), 2, "Uint16 must be 2 bytes" );
    assert_eq!( format_to_size( GpuVertexFormat::Sint16 ), 2, "Sint16 must be 2 bytes" );
    assert_eq!( format_to_size( GpuVertexFormat::Unorm16 ), 2, "Unorm16 must be 2 bytes" );
    assert_eq!( format_to_size( GpuVertexFormat::Snorm16 ), 2, "Snorm16 must be 2 bytes" );
    assert_eq!( format_to_size( GpuVertexFormat::Float16 ), 2, "Float16 must be 2 bytes" );
    assert_eq!( format_to_size( GpuVertexFormat::Unorm8x4Bgra ), 4, "Unorm8x4Bgra must be 4 bytes" );
  }

  #[ wasm_bindgen_test ]
  fn previously_covered_variants_still_correct_test()
  {
    assert_eq!( format_to_size( GpuVertexFormat::Uint8x2 ), 2 );
    assert_eq!( format_to_size( GpuVertexFormat::Uint8x4 ), 4 );
    assert_eq!( format_to_size( GpuVertexFormat::Float16x2 ), 4 );
    assert_eq!( format_to_size( GpuVertexFormat::Float16x4 ), 8 );
    assert_eq!( format_to_size( GpuVertexFormat::Unorm1010102 ), 4 );
    assert_eq!( format_to_size( GpuVertexFormat::Float32 ), 4 );
    assert_eq!( format_to_size( GpuVertexFormat::Float32x2 ), 8 );
    assert_eq!( format_to_size( GpuVertexFormat::Float32x3 ), 12 );
    assert_eq!( format_to_size( GpuVertexFormat::Float32x4 ), 16 );
  }
}
