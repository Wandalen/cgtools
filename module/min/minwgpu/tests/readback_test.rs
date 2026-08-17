//! Native tests for the pure row-padding logic behind `readback::rgba8`.
//!
//! The GPU half of readback ( buffer copy, mapping, polling ) needs a real
//! adapter and device and is exercised by the native `examples/minwgpu`
//! binaries; the padding math is pure and is pinned here.

use minwgpu::readback::{ padded_bytes_per_row, rows_unpad, bgra_to_rgba_swizzle };

#[ test ]
fn padded_bytes_per_row_keeps_aligned_widths()
{
  // 512 px * 4 B = 2048 B — already a multiple of 256, stays untouched.
  assert_eq!( padded_bytes_per_row( 2048 ), 2048 );
  assert_eq!( padded_bytes_per_row( 256 ), 256 );
  assert_eq!( padded_bytes_per_row( 0 ), 0 );
}

#[ test ]
fn padded_bytes_per_row_rounds_up_unaligned_widths()
{
  // 300 px * 4 B = 1200 B — the width class that breaks a hardcoded
  // `width * 4` readback copy.
  assert_eq!( padded_bytes_per_row( 1200 ), 1280 );
  assert_eq!( padded_bytes_per_row( 1 ), 256 );
  assert_eq!( padded_bytes_per_row( 257 ), 512 );
}

#[ test ]
fn unpad_rows_is_identity_for_aligned_rows()
{
  // 64 px * 4 B = 256 B per row — no padding, data passes through untouched.
  let data : Vec< u8 > = ( 0..=255 ).chain( 0..=255 ).collect();
  assert_eq!( rows_unpad( &data, ( 64, 2 ) ), data );
}

#[ test ]
fn unpad_rows_strips_per_row_padding()
{
  // 1 px rows : 4 pixel bytes, then 252 bytes of sentinel padding per row.
  let height = 3_u32;
  let mut data = Vec::new();
  for row in 0..height
  {
    let base = u8::try_from( row ).unwrap() * 10;
    data.extend_from_slice( &[ base, base + 1, base + 2, base + 3 ] );
    data.extend( core::iter::repeat_n( 0xAA_u8, 252 ) );
  }
  let pixels = rows_unpad( &data, ( 1, height ) );
  assert_eq!( pixels, vec![ 0, 1, 2, 3, 10, 11, 12, 13, 20, 21, 22, 23 ] );
}

// test_kind: bug_reproducer(BUG-166)
/// ## Root Cause
/// `rgba8`'s format check was `format.block_copy_size( None ) != Some( 4 )`, a byte-size-only
/// test that silently accepted `Bgra8Unorm`/`Bgra8UnormSrgb` -- explicitly listed in `rgba8`'s
/// own doc as accepted input -- without ever swapping their native blue/red byte order, so
/// the "RGBA8 pixels" this function promised were actually BGRA-ordered for those two formats.
/// ## Why Not Caught
/// No test exercised `rgba8` ( or any pure logic backing it ) with a `Bgra8*` input; every
/// prior padding/unpad test used a format-agnostic byte buffer with no channel semantics.
/// ## Fix Applied
/// `rgba8` now validates against an explicit four-format allowlist ( `Rgba8Unorm`,
/// `Rgba8UnormSrgb`, `Bgra8Unorm`, `Bgra8UnormSrgb` ) instead of a byte-size heuristic, and
/// swizzles red/blue via the new `bgra_to_rgba_swizzle` for the two BGRA formats after
/// unpadding. `bgra_to_rgba_swizzle` is exposed specifically so this pure swizzle logic is
/// unit-testable without a real GPU device/texture, matching this file's existing scope.
/// ## Prevention
/// This test builds a tightly packed 2-pixel BGRA buffer and asserts `bgra_to_rgba_swizzle`
/// produces the corresponding RGBA buffer, with alpha and green left untouched.
/// ## Pitfall
/// A byte-size check is not a format check -- "4 bytes per pixel" says nothing about channel
/// count, order, or bit layout; several real `wgpu` formats ( `Rg16*`, `R32*`,
/// `Rgb9e5Ufloat`, `Rgb10a2*`, `Rg11b10Ufloat` ) are also 4 bytes per pixel but are not
/// RGBA8-shaped at all, so only an explicit format allowlist can assert this function's
/// caller-visible "RGBA8 pixels" contract.
#[ test ]
fn bgra_to_rgba_swizzle_swaps_red_and_blue_per_pixel()
{
  // Pixel 0: B=10, G=20, R=30, A=40 (native BGRA byte order) -> RGBA: R=30, G=20, B=10, A=40.
  // Pixel 1: B=1, G=2, R=3, A=4 -> RGBA: R=3, G=2, B=1, A=4.
  let mut pixels = vec![ 10, 20, 30, 40, 1, 2, 3, 4 ];
  bgra_to_rgba_swizzle( &mut pixels );
  assert_eq!( pixels, vec![ 30, 20, 10, 40, 3, 2, 1, 4 ] );
}

/// An empty buffer is a valid ( zero-pixel ) input and is left untouched.
#[ test ]
fn bgra_to_rgba_swizzle_empty_is_noop()
{
  let mut pixels : Vec< u8 > = vec![];
  bgra_to_rgba_swizzle( &mut pixels );
  assert!( pixels.is_empty() );
}

/// A buffer whose length is not a multiple of 4 is a caller bug ( a malformed pixel buffer,
/// not a recoverable runtime condition ), matching this crate's existing `# Panics` house
/// style for buffer-shape preconditions.
#[ test ]
#[ should_panic( expected = "pixels must be a whole number of 4-byte RGBA/BGRA pixels" ) ]
fn bgra_to_rgba_swizzle_panics_on_non_multiple_of_4()
{
  let mut pixels = vec![ 1, 2, 3 ];
  bgra_to_rgba_swizzle( &mut pixels );
}
