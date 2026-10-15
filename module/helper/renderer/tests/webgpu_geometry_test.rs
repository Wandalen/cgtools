//! Unit-level validation tests for `renderer::webgpu::Geometry::new` — no
//! full render pipeline or pixel readback, just the attribute-length
//! cross-validation contract stated in its own doc comment ( see
//! `native_render_test.rs` for the full end-to-end render path instead ).
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use gpu_hal::{ Device, Error };
use renderer::webgpu::Geometry;

fn native_device() -> Device
{
  let ( device, _queue, _surface ) = Device::new_native( 4, 4 )
  .expect( "no native wgpu adapter available" );
  device
}

/// ## Root Cause
/// `Geometry::new` derived `vertex_count` from `positions.len() / 3` alone and never
/// cross-checked `normals`/`uvs`/`colors` against it, despite the function's own doc comment
/// stating all 4 arrays share "the same vertex count" as an intended invariant.
/// ## Why Not Caught
/// No test ever passed mismatched-length attribute arrays prior to this bug.
/// ## Fix Applied
/// `Geometry::new` now rejects any attribute array whose length doesn't match the vertex count
/// implied by `positions`, returning `Error::InvalidInput` instead of silently building an
/// undersized vertex buffer.
/// ## Prevention
/// This test passes a `uvs` array one vertex short of what `positions` implies and asserts
/// `Geometry::new` returns `Err` instead of `Ok`.
/// ## Pitfall
/// The mismatched buffer would previously upload successfully and only fail — as an out-of-bounds
/// GPU buffer read with no CPU-visible error — at the next draw call, far from where the bad data
/// was actually supplied.
#[ test ]
fn new_rejects_uvs_shorter_than_vertex_count()
{
  let device = native_device();

  // 4 vertices per positions/normals/colors, but uvs only covers 3.
  let result = Geometry::new
  (
    &device,
    &[ 0.0; 12 ],
    &[ 0.0; 12 ],
    &[ 0.0; 6 ], // 3 vertices' worth, not 4
    &[ 1.0; 16 ],
    None
  );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "short uvs must be rejected with InvalidInput, got {:?}", result.as_ref().err() );
}

#[ test ]
fn new_rejects_normals_longer_than_vertex_count()
{
  let device = native_device();

  let result = Geometry::new
  (
    &device,
    &[ 0.0; 12 ], // 4 vertices
    &[ 0.0; 15 ], // 5 vertices' worth
    &[ 0.0; 8 ],
    &[ 1.0; 16 ],
    None
  );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "long normals must be rejected with InvalidInput, got {:?}", result.as_ref().err() );
}

#[ test ]
fn new_rejects_colors_mismatched_with_vertex_count()
{
  let device = native_device();

  let result = Geometry::new
  (
    &device,
    &[ 0.0; 12 ], // 4 vertices
    &[ 0.0; 12 ],
    &[ 0.0; 8 ],
    &[ 1.0; 12 ], // 3 vertices' worth, not 4
    None
  );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "mismatched colors must be rejected with InvalidInput, got {:?}", result.as_ref().err() );
}

#[ test ]
fn new_rejects_positions_not_a_multiple_of_three()
{
  let device = native_device();

  let result = Geometry::new( &device, &[ 0.0; 10 ], &[ 0.0; 9 ], &[ 0.0; 6 ], &[ 1.0; 12 ], None );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "positions.len() not a multiple of 3 must be rejected, got {:?}", result.as_ref().err() );
}

#[ test ]
fn new_accepts_consistent_attribute_lengths()
{
  let device = native_device();

  // 4 vertices, every attribute array exactly the length `positions` implies.
  let result = Geometry::new
  (
    &device,
    &[ 0.0; 12 ],
    &[ 0.0; 12 ],
    &[ 0.0; 8 ],
    &[ 1.0; 16 ],
    Some( vec![ 0, 1, 2, 0, 2, 3 ] )
  );

  assert!( result.is_ok(), "consistent attribute lengths must be accepted, got err {:?}", result.as_ref().err() );
}
