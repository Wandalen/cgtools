//! Regression tests verifying `main.rs`'s GUI-weight override loop can reset a morph weight back
//! to exactly 0.0 once a slider has been touched, instead of leaving it stuck at its last
//! positive value.
//!
//! `morph_targets` is a binary-only example crate (no `[lib]` target) whose GUI-override loop
//! operates on plain `Vec<f32>` with no wasm dependency, so this test reads `main.rs`'s own real
//! source text via `include_str!` (to anchor against regression of the actual fix) plus a
//! hand-ported pure-Rust mirror of the fixed override loop to verify its behavior directly.

const MAIN_RS : &str = include_str!( "../src/main.rs" );

/// Mirrors `main.rs`'s fixed GUI-weight override loop: an untouched slot (`NAN` sentinel) leaves
/// `weights` alone; a touched slot (any real f32, including 0.0) overwrites it.
fn apply_gui_override( weights : &mut [ f32 ], gui_weights : &[ f32 ] )
{
  for i in 0..weights.len().min( gui_weights.len() )
  {
    if !gui_weights[ i ].is_nan()
    {
      weights[ i ] = gui_weights[ i ];
    }
  }
}

/// ## Root Cause
/// The GUI-weight override loop copied `gui_weights[i]` into the mesh's actual morph weight only
/// `if gui_weights[i] > 0.0`, using the buffer's `0.0` fill value as an implicit "slider not yet
/// touched" sentinel. Since `0.0` is also a real, reachable slider position (sliders range
/// `0.0..=1.0`), the check could never distinguish "never touched" from "dragged back down to its
/// minimum" — once a slider was raised above `0.0`, dragging it back to exactly `0.0` failed the
/// `> 0.0` check and left the mesh's actual weight stuck at its last positive value.
///
/// ## Why Not Caught
/// The forward direction (raising a slider above 0) always worked correctly and is the obvious
/// first thing to try when testing the demo — only reducing a previously-raised slider back to
/// its exact minimum exposes the stuck state, a less obvious interaction to think to test.
///
/// ## Fix Applied
/// Changed the buffer's fill value from `0.0` to `f32::NAN` and the guard from
/// `gui_weights[i] > 0.0` to `!gui_weights[i].is_nan()` — `NAN` is never a value a slider's
/// `onChange` callback can produce, so it unambiguously means "untouched", and `0.0` becomes a
/// normal, always-applied value like any other.
///
/// ## Prevention
/// This test anchors the real fix's sentinel/guard text in `main.rs` via `include_str!`, and
/// exercises a hand-ported mirror of the loop directly: an untouched (`NAN`) slot must leave the
/// mesh weight alone, and a touched-then-zeroed (`0.0`) slot must actually reset it.
///
/// ## Pitfall
/// A range's own minimum value makes a poor "not yet set" sentinel whenever that minimum is also
/// a legitimate, user-reachable state — prefer a value the real domain can never produce (here,
/// `NAN`, since morph weight sliders only ever emit finite values) over overloading an in-domain
/// value as a dual-purpose flag.
#[ test ]
#[ allow( clippy::float_cmp, reason = "values under test pass through a straight assignment with no arithmetic, so exact equality is the correct check, not an approximation" ) ]
fn bug_reproducer_bug_xxx_gui_slider_can_reset_weight_to_zero()
{
  assert!
  (
    MAIN_RS.contains( "vec![ f32::NAN; 60 ]" ),
    "gui_weights should be sentinel-initialized with NAN, not 0.0 (BUG-XXX)"
  );
  assert!
  (
    MAIN_RS.contains( "!gui_weights[ i ].is_nan()" ),
    "the override guard should check is_nan(), not `> 0.0` (BUG-XXX)"
  );

  // Untouched slot: NAN sentinel must never overwrite the mesh's actual weight.
  let mut weights = vec![ 0.42_f32 ];
  apply_gui_override( &mut weights, &[ f32::NAN ] );
  assert_eq!( weights[ 0 ], 0.42, "an untouched slider must not override the mesh weight" );

  // Touched, raised: a positive GUI value must override.
  let mut weights = vec![ 0.0_f32 ];
  apply_gui_override( &mut weights, &[ 0.8 ] );
  assert_eq!( weights[ 0 ], 0.8, "a raised slider must override the mesh weight" );

  // Touched, then reset to exactly 0.0: must actually reset, not stay stuck at the prior value.
  let mut weights = vec![ 0.8_f32 ];
  apply_gui_override( &mut weights, &[ 0.0 ] );
  assert_eq!
  (
    weights[ 0 ], 0.0,
    "a slider dragged back to exactly 0.0 must reset the mesh weight, not leave it stuck (BUG-XXX)"
  );
}
