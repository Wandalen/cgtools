//! Tests for animation modifier Scaler
#![ cfg( feature = "animation" ) ]

use renderer::webgl::
{
  Node,
  animation::{ Scaler, AnimatableComposition, base::{ TRANSLATION_PREFIX, ROTATION_PREFIX, SCALE_PREFIX } }
};
use animation::{ Sequence, Sequencer, Tween, easing::{ EasingBuilder, Linear } };
use mingl::{ F32x3, F64x3, F64x4, QuatF32, QuatF64 };
use std::{ f64::consts::PI, rc::Rc, cell::RefCell };
use rustc_hash::FxHashMap;

/// Helper to create a simple rotation tween sequence
fn rotation_sequence_create( start : QuatF64, end : QuatF64, duration : f64 ) -> Sequence< Tween< QuatF64 > >
{
  let tweens =
  vec![
    Tween::new( start, end, duration / 2.0, Linear::build() ),
    Tween::new( end, start, duration / 2.0, Linear::build() )
  ];
  Sequence::new( tweens ).unwrap()
}

/// Helper to create a simple translation tween sequence
fn translation_sequence_create( start : F64x3, end : F64x3, duration : f64 ) -> Sequence< Tween< F64x3 > >
{
  let tweens =
  vec![
    Tween::new( start, end, duration / 2.0, Linear::build() ),
    Tween::new( end, start, duration / 2.0, Linear::build() )
  ];
  Sequence::new( tweens ).unwrap()
}

#[ test ]
fn test_scaler_new()
{
  let sequencer = Sequencer::new();
  let scaler = Scaler::new( sequencer );

  // Verify internal state through public API
  assert!( scaler.group_get( "nonexistent" ).is_none(), "New scaler should have no scaled nodes" );
}

#[ test ]
fn test_scaler_add_group()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  let nodes = vec![ "node1".into(), "node2".into() ];
  let scale = F64x4::new( 0.5, 0.5, 1.0, 1.0 );

  scaler.add( "group1", nodes.clone(), scale );

  let group = scaler.group_get( "group1" ).unwrap();
  assert_eq!( group.len(), 2, "Group should have 2 nodes" );
}

#[ test ]
fn test_scaler_remove_group()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  scaler.add( "group1", vec![ "node1".into() ], F64x4::splat( 0.5 ) );
  assert!( scaler.group_get( "group1" ).is_some(), "Group should exist" );

  scaler.remove( "group1" );
  assert!( scaler.group_get( "group1" ).is_none(), "Group should be removed" );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "values read back through a getter are the exact literals just written; no arithmetic in between" ) ]
fn test_scaler_scale_get_mut()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 0.5, 0.5, 1.0, 1.0 ) );

  let scale = scaler.scale_get_mut( "group1" ).unwrap();
  *scale = F64x4::new( 1.0, 1.0, 1.0, 1.0 );

  let new_scale = scaler.scale_get( "group1" ).unwrap();
  assert_eq!( new_scale.x(), 1.0 );
  assert_eq!( new_scale.y(), 1.0 );
  assert_eq!( new_scale.z(), 1.0 );
}

#[ test ]
fn test_scaler_clear()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  scaler.add( "group1", vec![ "node1".into() ], F64x4::splat( 0.5 ) );
  scaler.add( "group2", vec![ "node2".into() ], F64x4::splat( 0.8 ) );
  assert!( scaler.group_get( "group1" ).is_some() );
  assert!( scaler.group_get( "group2" ).is_some() );

  scaler.clear();
  assert!( scaler.group_get( "group1" ).is_none(), "All groups should be cleared" );
  assert!( scaler.group_get( "group2" ).is_none(), "All groups should be cleared" );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "values read back through a getter are the exact literals just written; no arithmetic in between" ) ]
fn test_grouped_nodes_independence()
{
  let mut sequencer = Sequencer::new();

  // Add two rotation animations with different angles
  let rot1_start = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let rot1_end = QuatF64::from_axis_angle( F64x3::new( 0.0, 0.0, 1.0 ), PI / 2.0 );
  let seq1 = rotation_sequence_create( rot1_start, rot1_end, 1.0 );
  sequencer.insert( format!( "node1{ROTATION_PREFIX}" ).as_str(), seq1 );

  let rot2_start = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let rot2_end = QuatF64::from_axis_angle( F64x3::new( 1.0, 0.0, 0.0 ), PI );
  let seq2 = rotation_sequence_create( rot2_start, rot2_end, 1.0 );
  sequencer.insert( format!( "node2{ROTATION_PREFIX}" ).as_str(), seq2 );

  let mut scaler = Scaler::new( sequencer );

  // Add two groups with different scaling factors
  scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 1.0, 0.5, 1.0, 1.0 ) ); // 50% rotation scaling
  scaler.add( "group2", vec![ "node2".into() ], F64x4::new( 1.0, 0.25, 1.0, 1.0 ) ); // 25% rotation scaling

  // Verify groups are independent
  let group1_scale = scaler.scale_get( "group1" ).unwrap();
  let group2_scale = scaler.scale_get( "group2" ).unwrap();

  assert_eq!( group1_scale.y(), 0.5, "Group1 should have 0.5 rotation scale" );
  assert_eq!( group2_scale.y(), 0.25, "Group2 should have 0.25 rotation scale" );
}

#[ test ]
fn test_animatable_composition_update()
{
  let mut sequencer = Sequencer::new();

  // Add a simple translation animation
  let start = F64x3::new( 0.0, 0.0, 0.0 );
  let end = F64x3::new( 1.0, 1.0, 1.0 );
  let seq = translation_sequence_create( start, end, 1.0 );
  sequencer.insert( format!( "node1{TRANSLATION_PREFIX}" ).as_str(), seq );

  let mut scaler = Scaler::new( sequencer );

  // Update should not panic
  scaler.update( 0.5 );
}

#[ test ]
#[ expect( clippy::float_cmp, reason = "values read back through a getter are the exact literals just written; no arithmetic in between" ) ]
fn test_scaler_weights_structure()
{
  let sequencer = Sequencer::new();
  let mut scaler = Scaler::new( sequencer );

  // Test that weights have three components: translation (x), rotation (y), scale (z)
  scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 0.5, 0.7, 0.3, 1.0 ) );

  let weights = scaler.scale_get( "group1" ).unwrap();
  assert_eq!( weights.x(), 0.5, "X component should be translation weight" );
  assert_eq!( weights.y(), 0.7, "Y component should be rotation weight" );
  assert_eq!( weights.z(), 0.3, "Z component should be scale weight" );
}

/// ## Root Cause
/// `AnimatableComposition::set` for `Scaler` only ever called `scaled_rotation_apply` for
/// grouped nodes -- `scaled_translation_apply`/`scaled_scale_apply` did not exist, so a grouped
/// node's translation and scale channels were never touched at all, regardless of the group's
/// own `x` ( translation ) / `z` ( scale ) weight components.
///
/// ## Why Not Caught
/// `test_grouped_nodes_independence` only asserts on `Scaler::scale_get`'s own weight
/// bookkeeping getter, never on a node's actual applied transform after `set()` -- no
/// pre-existing test samples a grouped node's translation/scale post-`set()` at all.
///
/// ## Fix Applied
/// Added `scaled_translation_apply`/`scaled_scale_apply`, mirroring `scaled_rotation_apply`'s
/// per-segment delta-scaling pattern for the `F64x3` translation/scale channels, and wired both
/// into `set()` for every grouped node ( BUG-184, `scaling.rs` ).
///
/// ## Prevention
/// This test groups a node with a non-default translation and scale animation and asserts the
/// resulting node transform is no longer frozen at `Node::new()`'s defaults after `set()`.
///
/// ## Pitfall
/// This test only asserts translation/scale were applied AT ALL, not their precise numeric
/// value -- see `test_scaled_translation_first_segment_not_corrupted_by_last_segment_end_value`
/// ( BUG-185, fixed separately ) for a test pinning down the exact sampled value.
// test_kind: bug_reproducer(BUG-184)
#[ test ]
fn test_scaler_applies_translation_and_scale_to_grouped_nodes()
{
  let mut sequencer = Sequencer::new();

  let translation_seq = translation_sequence_create
  (
    F64x3::new( 10.0, 20.0, 30.0 ),
    F64x3::new( 40.0, 50.0, 60.0 ),
    2.0
  );
  sequencer.insert( format!( "node1{TRANSLATION_PREFIX}" ).as_str(), translation_seq );

  let scale_seq = translation_sequence_create
  (
    F64x3::new( 5.0, 5.0, 5.0 ),
    F64x3::new( 8.0, 8.0, 8.0 ),
    2.0
  );
  sequencer.insert( format!( "node1{SCALE_PREFIX}" ).as_str(), scale_seq );

  let mut scaler = Scaler::new( sequencer );
  scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 1.0, 1.0, 1.0, 1.0 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  nodes.insert( "node1".to_string().into_boxed_str(), node.clone() );

  scaler.set( &nodes );

  let default_node = Node::new();
  let got_translation = node.borrow().translation_get();
  let got_scale = node.borrow().scale_get();

  assert_ne!
  (
    got_translation, default_node.translation_get(),
    "grouped node's translation must no longer be frozen at the default after set()"
  );
  assert_ne!
  (
    got_scale, default_node.scale_get(),
    "grouped node's scale must no longer be frozen at the default after set()"
  );
}

/// ## Root Cause
/// `scaled_rotation_apply`'s continuity-rebase line only ran when `scale < 1.0 && i > 0`,
/// leaving every segment after the first sampling a stale, un-rebased `start_value` whenever
/// `scale >= 1.0` -- which includes the GUI's own default amplitude ( 1.0 ) and its entire
/// "amplify" range ( 1.0 to 3.0 ), so continuity was broken for the most common case, not an
/// edge case.
///
/// ## Why Not Caught
/// No pre-existing test drove a `Sequence` past its first segment boundary while asserting on
/// the resulting node rotation value -- `test_grouped_nodes_independence` only checks
/// `Scaler::scale_get`'s own bookkeeping, never a post-`set()` node transform.
///
/// ## Fix Applied
/// Changed the guard to `if i > 0`, unconditional on `scale` ( BUG-186, `scaling.rs` ).
///
/// ## Prevention
/// This test drives a two-segment rotation sequence past its first boundary with `scale = 1.5`
/// ( >= 1.0, the exact case the old guard skipped ) and asserts the second segment's sampled
/// rotation is close to the first segment's own scaled end value -- independently recomputed
/// here using the same axis-angle scaling formula `scaled_rotation_apply` itself uses -- rather
/// than the stale, un-rebased original value.
///
/// ## Pitfall
/// A guard combining an unrelated numeric condition ( `scale < 1.0` ) with the actual
/// correctness condition ( `i > 0` ) via `&&` silently narrows when a piece of otherwise
/// unconditional bookkeeping logic runs -- always double-check every clause in a multi-condition
/// guard is actually load-bearing for what the guard claims to protect.
// test_kind: bug_reproducer(BUG-186)
#[ test ]
fn test_scaled_rotation_continuity_rebase_applies_when_scale_at_or_above_one()
{
  let mut sequencer = Sequencer::new();

  let q_a = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] );
  let q_b = QuatF64::from_axis_angle( F64x3::new( 0.0, 0.0, 1.0 ), PI / 2.0 );
  let seq = rotation_sequence_create( q_a, q_b, 2.0 );
  sequencer.insert( format!( "node1{ROTATION_PREFIX}" ).as_str(), seq );

  let mut scaler = Scaler::new( sequencer );
  // scale ( y component ) = 1.5, i.e. >= 1.0 -- the exact case the pre-fix
  // `scale < 1.0 && i > 0` guard incorrectly skipped the continuity rebase for.
  scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 1.0, 1.5, 1.0, 1.0 ) );

  // Both tweens built by `rotation_sequence_create` share the same default delay ( 0.0 ), so
  // any nonzero update pushes `current_id_get()` straight to the second ( and only other )
  // tween -- this makes the scaling loop process both segments, exercising the `i > 0` rebase
  // branch.
  scaler.update( 0.002 );

  // Independently reproduce segment 0's own scaled end value -- the continuity target segment
  // 1's start_value must rebase to once BUG-186 is fixed -- using the same axis-angle scaling
  // formula `scaled_rotation_apply` itself uses.
  let delta0 = q_a.conjugate() * q_b;
  let angle0 = 2.0 * delta0.w().clamp( -1.0, 1.0 ).acos();
  let axis0 = F64x3::new( 0.0, 0.0, 1.0 );
  let expected_continuity_target = ( q_a * QuatF64::from_axis_angle( axis0, angle0 * 1.5 ) ).normalize();
  let expected_f32 = QuatF32::from( expected_continuity_target.0.map( | v | v as f32 ) );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  nodes.insert( "node1".to_string().into_boxed_str(), node.clone() );

  scaler.set( &nodes );

  let got = node.borrow().rotation_get();
  let dot = got.dot( &expected_f32 ).abs();

  assert!
  (
    dot > 0.99,
    "segment 1 must start interpolating from segment 0's own scaled end value ( continuity ), \
    even when scale ( 1.5 ) is >= 1.0 -- got {got:?}, expected close to {expected_f32:?}, \
    |dot|={dot}"
  );
}

/// ## Root Cause
/// `scaled_translation_apply`/`scaled_rotation_apply`/`scaled_scale_apply` each ended with an
/// unconditional `tweens[ 0 ].start_value = tweens.last().unwrap().end_value;`, regardless of
/// whether the per-segment loop above it ( `for i in 0..( ( current + 1 ).min( tweens.len() ) )`
/// ) had actually reached the last segment this call. Since `tweens` is rebuilt fresh from the
/// unscaled Sequencer data on every call and never persists across frames, this write was either
/// inert ( `current` at the last index -- `tweens[ 0 ]` isn't sampled and is discarded before the
/// next call anyway ) or actively harmful ( `current == 0`, the common case of playing a
/// sequence's first segment -- it overwrote the CURRENTLY SAMPLED tween's `start_value` with the
/// raw, un-rebased, unscaled `end_value` of an unrelated, untouched last segment ).
///
/// ## Why Not Caught
/// `test_scaled_rotation_continuity_rebase_applies_when_scale_at_or_above_one` ( BUG-186 ) only
/// drives past the first segment boundary, landing on the LAST segment of a two-segment sequence
/// -- exactly the case where this clobber is inert. No pre-existing test sampled a node's
/// transform while `current == 0`.
///
/// ## Fix Applied
/// Deleted the unconditional `tweens[ 0 ].start_value = tweens.last().unwrap().end_value;` line
/// from all three `scaled_*_apply` functions ( BUG-185, `scaling.rs` ).
///
/// ## Prevention
/// This test drives a two-segment translation sequence to `elapsed = 0.5`, well within the first
/// segment's `[ 0, 2.0 )` window ( `current == 0` ), where the second segment's authored end
/// value ( `( 1000, 1000, 1000 )` ) is wildly different from the first segment's own start/end (
/// `( 0, 0, 0 )` -> `( 10, 0, 0 )` ) -- any leftover clobber would be immediately, grossly visible
/// rather than masked by a coincidentally-similar value.
///
/// ## Pitfall
/// All values here are exact sums of multiples of 0.5 ( duration 2.0, elapsed 0.5, endpoints on
/// whole numbers ), so the expected sampled value ( `2.5, 0.0, 0.0` ) is bit-exact under `f64`/
/// `f32` arithmetic -- no epsilon-tolerance ambiguity between the pre-fix ( `752.5, 750.0, 750.0`
/// ) and post-fix values.
// test_kind: bug_reproducer(BUG-185)
#[ test ]
fn test_scaled_translation_first_segment_not_corrupted_by_last_segment_end_value()
{
  let mut sequencer = Sequencer::new();

  let seq = Sequence::new
  (
    vec!
    [
      Tween::new( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 10.0, 0.0, 0.0 ), 2.0, Linear::build() ),
      Tween::new( F64x3::new( 10.0, 0.0, 0.0 ), F64x3::new( 1000.0, 1000.0, 1000.0 ), 2.0, Linear::build() )
      .with_delay( 2.0 )
    ]
  ).unwrap();
  sequencer.insert( format!( "node1{TRANSLATION_PREFIX}" ).as_str(), seq );

  let mut scaler = Scaler::new( sequencer );
  scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 1.0, 1.0, 1.0, 1.0 ) );

  // Advances the real Sequencer to elapsed = 0.5, well inside the first segment's [ 0, 2.0 )
  // window -- current_id_get() == 0, the case the unconditional clobber corrupted.
  scaler.update( 0.5 );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  nodes.insert( "node1".to_string().into_boxed_str(), node.clone() );

  scaler.set( &nodes );

  let got = node.borrow().translation_get();
  let expected = F32x3::new( 2.5, 0.0, 0.0 );

  assert_eq!
  (
    got, expected,
    "sampling 25% through the first segment ( 0,0,0 ) -> ( 10,0,0 ) must interpolate from the \
    segment's OWN start value, not the unrelated last segment's raw end value ( 1000,1000,1000 \
    ) -- got {got:?}, expected {expected:?}"
  );
}

/// ## Root Cause
/// `scaled_translation_apply`/`scaled_rotation_apply`/`scaled_scale_apply` each clone their
/// tweens directly from the persistent Sequencer's own already-playing state ( carrying forward
/// real, non-zero `elapsed` ), then wrap those clones in a brand-new local `Sequence` and drive
/// it via `.update( <channel>.time() )` -- passing the FULL ABSOLUTE elapsed time as though
/// replaying a still-fresh sequence from scratch. `Sequence::new` resets none of its players'
/// own state, so the already-non-zero Tween-level `elapsed` and the freshly-applied absolute
/// time compound, roughly doubling the effective elapsed time driving the sampled value.
///
/// ## Why Not Caught
/// No pre-existing test asserted a scaled channel's sampled value against its real elapsed
/// FRACTION of a segment's duration -- `test_scaled_rotation_continuity_rebase_applies_when_
/// scale_at_or_above_one` ( BUG-186 ) only checks segment-boundary CONTINUITY ( that one
/// segment's start rebases to the previous segment's end ), a value equality that stays correct
/// regardless of the underlying elapsed being wrong by a constant multiplicative factor.
///
/// ## Fix Applied
/// Added a `tween.reset()` pass over every cloned tween immediately after cloning, in all three
/// `scaled_*_apply` functions, before the per-segment rebase/scale loop ( BUG-198, `scaling.rs`
/// ) -- makes the local replay behave like a genuinely fresh sequence driven from t=0, matching
/// what `Sequence::new` + absolute-time `.update()` already assumes for its OWN bookkeeping.
///
/// ## Prevention
/// This test drives a translation sequence to EXACTLY half its first segment's duration ( elapsed
/// 2.0 of 4.0 ) and asserts the sampled value is the 50%-interpolated value ( 10.0 ), not the
/// segment's END value ( 20.0 ) -- the doubled effective elapsed BUG-198 produced would clamp
/// `normalized_time` to 1.0 ( elapsed 2.0+2.0 = 4.0 = duration ), freezing the output at the end
/// pose a full segment-duration-half early.
///
/// ## Pitfall
/// Distinct from BUG-185 ( which corrupts WHICH value `start_value` holds ) -- this bug corrupts
/// WHEN, along a still-correct start/end pair, the current sample falls. Driving elapsed to
/// exactly half the FIRST segment's own duration ( never reaching a second segment ) isolates it
/// fully from BUG-185, which requires an untouched last segment to manifest at all.
// test_kind: bug_reproducer(BUG-198)
#[ test ]
fn test_scaled_translation_speed_matches_real_elapsed_not_doubled()
{
  let mut sequencer = Sequencer::new();

  let seq = Sequence::new
  (
    vec!
    [
      Tween::new( F64x3::new( 0.0, 0.0, 0.0 ), F64x3::new( 20.0, 0.0, 0.0 ), 4.0, Linear::build() ),
      Tween::new( F64x3::new( 20.0, 0.0, 0.0 ), F64x3::new( 30.0, 0.0, 0.0 ), 1.0, Linear::build() )
      .with_delay( 4.0 )
    ]
  ).unwrap();
  sequencer.insert( format!( "node1{TRANSLATION_PREFIX}" ).as_str(), seq );

  let mut scaler = Scaler::new( sequencer );
  scaler.add( "group1", vec![ "node1".into() ], F64x4::new( 1.0, 1.0, 1.0, 1.0 ) );

  // Exactly half of the first segment's 4.0s duration -- current_id_get() == 0 throughout.
  scaler.update( 2.0 );

  let mut nodes = FxHashMap::default();
  let node = Rc::new( RefCell::new( Node::new() ) );
  nodes.insert( "node1".to_string().into_boxed_str(), node.clone() );

  scaler.set( &nodes );

  let got = node.borrow().translation_get();
  let expected = F32x3::new( 10.0, 0.0, 0.0 );

  assert_eq!
  (
    got, expected,
    "sampling exactly 50% through a 4.0s first segment must interpolate to the 50% value ( 10.0 \
    ), not freeze at the segment's END value ( 20.0, what a doubled effective elapsed would \
    produce ) -- got {got:?}, expected {expected:?}"
  );
}
