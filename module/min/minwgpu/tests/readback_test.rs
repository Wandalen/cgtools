//! Native tests for the pure row-padding logic behind `readback::rgba8`.
//!
//! The GPU half of readback ( buffer copy, mapping, polling ) needs a real
//! adapter and device and is exercised by the native `examples/minwgpu`
//! binaries; the padding math is pure and is pinned here.

use minwgpu::readback::{ padded_bytes_per_row, rows_unpad };

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
