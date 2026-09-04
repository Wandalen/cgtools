//! Unit-level validation tests for `renderer::webgpu::Lights::point_push` and `spot_push`. Pure
//! CPU-side data packing, no GPU device needed. `spot_push`'s tests additionally cover the
//! cone-angle invariant its own doc comment states but didn't use to enforce ( see BUG-255 ).
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
#![ expect( clippy::float_cmp, reason = "assertions check exact pass-through of position/color into the raw buffer; point_push does no arithmetic on them, so epsilon comparison would weaken the test" ) ]

use renderer::webgpu::{ Lights, MAX_POINT_LIGHTS };

/// A well-formed point light argument set.
fn valid_point_args() -> ( [ f32; 3 ], [ f32; 3 ], f32, f32 )
{
  (
    [ 1.0, 2.0, 3.0 ],
    [ 1.0, 0.5, 0.25 ],
    2.5,
    10.0
  )
}

#[ test ]
fn point_push_accepts_and_packs_a_light()
{
  let ( position, color, strength, range ) = valid_point_args();
  let mut lights = Lights::new();

  assert!( lights.point_push( position, color, strength, range ), "a well-formed point light must be accepted" );

  let raw = lights.as_raw();
  assert_eq!( raw.counts[ 0 ], 1 );
  assert_eq!( raw.point[ 0 ].position_range, [ position[ 0 ], position[ 1 ], position[ 2 ], range ] );
  assert_eq!( raw.point[ 0 ].color_strength, [ color[ 0 ], color[ 1 ], color[ 2 ], strength ] );
}

#[ test ]
fn point_push_sequential_pushes_land_in_incrementing_slots()
{
  let mut lights = Lights::new();

  assert!( lights.point_push( [ 1.0, 0.0, 0.0 ], [ 1.0, 0.0, 0.0 ], 1.0, 1.0 ) );
  assert!( lights.point_push( [ 0.0, 1.0, 0.0 ], [ 0.0, 1.0, 0.0 ], 2.0, 2.0 ) );
  assert!( lights.point_push( [ 0.0, 0.0, 1.0 ], [ 0.0, 0.0, 1.0 ], 3.0, 3.0 ) );

  let raw = lights.as_raw();
  assert_eq!( raw.counts[ 0 ], 3 );
  assert_eq!( raw.point[ 0 ].position_range, [ 1.0, 0.0, 0.0, 1.0 ] );
  assert_eq!( raw.point[ 1 ].position_range, [ 0.0, 1.0, 0.0, 2.0 ] );
  assert_eq!( raw.point[ 2 ].position_range, [ 0.0, 0.0, 1.0, 3.0 ] );
}

#[ test ]
fn point_push_rejects_past_capacity_without_corrupting_existing_entries()
{
  let ( position, color, strength, range ) = valid_point_args();
  let mut lights = Lights::new();

  for _ in 0..MAX_POINT_LIGHTS
  {
    assert!( lights.point_push( position, color, strength, range ) );
  }
  assert_eq!( lights.as_raw().counts[ 0 ], MAX_POINT_LIGHTS as u32 );

  assert!( !lights.point_push( [ 9.0, 9.0, 9.0 ], [ 9.0, 9.0, 9.0 ], 9.0, 9.0 ), "pushing past MAX_POINT_LIGHTS must be rejected" );

  let raw = lights.as_raw();
  assert_eq!( raw.counts[ 0 ], MAX_POINT_LIGHTS as u32, "rejected push must not increment the count" );
  assert_eq!( raw.point[ 0 ].position_range, [ position[ 0 ], position[ 1 ], position[ 2 ], range ], "rejected push must not corrupt existing entries" );
}

/// A well-formed spot light argument set — mirrors `examples/minwebgpu/renderer_pbr_scene`'s own
/// real `spot_push` call.
fn valid_args() -> ( [ f32; 3 ], [ f32; 3 ], [ f32; 3 ], f32, f32, f32, f32 )
{
  (
    [ 0.0, 6.0, 5.0 ],
    [ 0.0, -1.0, -0.8 ],
    [ 1.0, 1.0, 1.0 ],
    60.0,
    30.0,
    0.35,
    0.55
  )
}

#[ test ]
fn spot_push_accepts_valid_cone_angles()
{
  let ( position, direction, color, strength, range, inner, outer ) = valid_args();
  let mut lights = Lights::new();

  assert!( lights.spot_push( position, direction, color, strength, range, inner, outer ), "a well-formed spot light must be accepted" );
}

/// ## Root Cause
/// `Lights::spot_push`'s own doc comment documented the caller obligation
/// `inner_cone_angle <= outer_cone_angle`, but the function never validated it — and
/// `shaders/main.wgsl`'s `smoothstep( light.outer.x, light.color_inner.w, angle )` divides by
/// `( inner_cone_angle - outer_cone_angle )` internally, which is exactly `0.0` the moment a
/// caller follows the documented ( non-strict ) contract to the letter with equal angles —
/// producing NaN that propagates into every fragment lit by that spot light.
/// ## Why Not Caught
/// `spot_push` had zero test coverage of any kind prior to this bug, and no test ever exercised
/// the shader-side `smoothstep` call with equal cone angles — this crate has no native WebGPU
/// pixel-readback path for the lit fragment shader, so the shader-side symptom itself isn't
/// natively testable, only the CPU-side contract that feeds it is.
/// ## Fix Applied
/// `spot_push` now rejects non-finite cone angles and tightens the documented invariant from
/// `inner_cone_angle <= outer_cone_angle` to a strict `inner_cone_angle < outer_cone_angle`,
/// returning `false` ( the same "dropped" signal already used for a full light array ) instead of
/// packing a degenerate light into the uniform buffer.
/// ## Prevention
/// Any CPU-side setter that packs data consumed by a GPU formula with a division must validate
/// against that formula's actual domain, not just the caller-facing doc comment's stated
/// obligation — a documented `<=` is not automatically safe merely because it's documented.
/// ## Pitfall
/// The previous doc comment's own `<=` wording was itself part of the bug: it explicitly permitted
/// the exact input that breaks the consuming shader formula, so a caller reading only the doc
/// comment ( not the shader ) would reasonably conclude equal angles are fine.
#[ test ]
fn spot_push_rejects_equal_cone_angles()
{
  let ( position, direction, color, strength, range, inner, _ ) = valid_args();
  let mut lights = Lights::new();

  assert!( !lights.spot_push( position, direction, color, strength, range, inner, inner ), "inner_cone_angle == outer_cone_angle must be rejected -- it divides by zero in the shader's smoothstep call" );
}

#[ test ]
fn spot_push_rejects_inner_greater_than_outer()
{
  let ( position, direction, color, strength, range, inner, outer ) = valid_args();
  let mut lights = Lights::new();

  assert!( !lights.spot_push( position, direction, color, strength, range, outer, inner ), "inner_cone_angle > outer_cone_angle must be rejected" );
}

#[ test ]
fn spot_push_rejects_non_finite_cone_angles()
{
  let ( position, direction, color, strength, range, inner, outer ) = valid_args();
  let mut lights = Lights::new();

  assert!( !lights.spot_push( position, direction, color, strength, range, f32::NAN, outer ), "NaN inner_cone_angle must be rejected" );
  assert!( !lights.spot_push( position, direction, color, strength, range, inner, f32::INFINITY ), "infinite outer_cone_angle must be rejected" );
}
