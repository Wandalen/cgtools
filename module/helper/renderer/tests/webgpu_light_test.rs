//! Unit-level validation tests for `renderer::webgpu::Lights::spot_push` — the cone-angle
//! invariant its own doc comment states but didn't use to enforce ( see BUG-255 ). Pure CPU-side
//! data packing, no GPU device needed.
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use renderer::webgpu::Lights;

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
