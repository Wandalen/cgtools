//! Verifies the glTF animation loader's pure decode/sequence-building logic --
//! `renderer::webgl::animation::loaders::gltf::{ channel_decode, vec3_sequence, weights_sequence }`
//! -- with zero `gl`/`GL`/`WebGl` calls anywhere in their bodies. Mirrors
//! `gltf_light_parsing_test.rs`'s inline-JSON-fixture-via-`Gltf::from_slice` pattern ( task 118's
//! own precedent ) for a second pure sub-surface of this crate's glTF loading, this time the
//! animation loader rather than the light-extension loader. `quat_sequence` is the same shape and
//! remains deliberately out of scope here ( same-shape follow-up work ); `weights_sequence` is
//! covered only for its BUG-262 zero-targets regression, not its general keyframe-building logic.

use renderer::webgl::animation::loaders::gltf::{ channel_decode, vec3_sequence, weights_sequence };
use gltf::animation::util::ReadOutputs;
use mingl::F64x3;

fn first_channel( gltf : &gltf::Gltf ) -> gltf::animation::Channel< '_ >
{
  gltf.animations().next().expect( "fixture declares one animation" )
  .channels().next().expect( "animation declares one channel" )
}

const LINEAR_TWO_KEYFRAME_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "scene": 0,
  "scenes": [ { "nodes": [ 0 ] } ],
  "nodes": [ { "name": "linear_two_keyframe_node" } ],
  "animations":
  [
    {
      "name": "linear_two_keyframe_translation",
      "channels": [ { "sampler": 0, "target": { "node": 0, "path": "translation" } } ],
      "samplers": [ { "input": 0, "output": 1, "interpolation": "LINEAR" } ]
    }
  ],
  "accessors":
  [
    { "bufferView": 0, "byteOffset": 0, "componentType": 5126, "count": 2, "type": "SCALAR" },
    { "bufferView": 1, "byteOffset": 0, "componentType": 5126, "count": 2, "type": "VEC3" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 8 },
    { "buffer": 0, "byteOffset": 8, "byteLength": 24 }
  ],
  "buffers": [ { "byteLength": 32, "uri": "placeholder.bin" } ]
}
"#;

/// Raw little-endian bytes matching [`LINEAR_TWO_KEYFRAME_FIXTURE`]'s accessor/bufferView
/// layout exactly: 2 `f32` times, then 2 `[f32; 3]` translation values -- passed directly as
/// `channel_decode`/`vec3_sequence`'s `buffers` parameter, bypassing the document's own
/// ( placeholder, never-read ) `uri` entirely.
fn linear_two_keyframe_buffers() -> Vec< Vec< u8 > >
{
  let mut bytes = Vec::new();
  for t in [ 0.0f32, 1.0f32 ]
  {
    bytes.extend_from_slice( &t.to_le_bytes() );
  }
  for v in [ [ 1.0f32, 2.0, 3.0 ], [ 4.0, 5.0, 6.0 ] ]
  {
    for c in v
    {
      bytes.extend_from_slice( &c.to_le_bytes() );
    }
  }
  vec![ bytes ]
}

const CUBICSPLINE_TWO_KEYFRAME_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "scene": 0,
  "scenes": [ { "nodes": [ 0 ] } ],
  "nodes": [ { "name": "cubicspline_two_keyframe_node" } ],
  "animations":
  [
    {
      "name": "cubicspline_two_keyframe_translation",
      "channels": [ { "sampler": 0, "target": { "node": 0, "path": "translation" } } ],
      "samplers": [ { "input": 0, "output": 1, "interpolation": "CUBICSPLINE" } ]
    }
  ],
  "accessors":
  [
    { "bufferView": 0, "byteOffset": 0, "componentType": 5126, "count": 2, "type": "SCALAR" },
    { "bufferView": 1, "byteOffset": 0, "componentType": 5126, "count": 6, "type": "VEC3" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 8 },
    { "buffer": 0, "byteOffset": 8, "byteLength": 72 }
  ],
  "buffers": [ { "byteLength": 80, "uri": "placeholder.bin" } ]
}
"#;

/// Matches [`CUBICSPLINE_TWO_KEYFRAME_FIXTURE`]: 2 `f32` times, then 2 keyframes worth of
/// CubicSpline ( in-tangent, value, out-tangent ) `VEC3` triples ( 6 total ). T02 only asserts
/// on `channel_decode`'s reported component count, so the triples' actual content is arbitrary.
fn cubicspline_two_keyframe_buffers() -> Vec< Vec< u8 > >
{
  let mut bytes = Vec::new();
  for t in [ 0.0f32, 1.0f32 ]
  {
    bytes.extend_from_slice( &t.to_le_bytes() );
  }
  for _ in 0 .. 6
  {
    for c in [ 0.0f32, 0.0, 0.0 ]
    {
      bytes.extend_from_slice( &c.to_le_bytes() );
    }
  }
  vec![ bytes ]
}

const SINGLE_KEYFRAME_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "scene": 0,
  "scenes": [ { "nodes": [ 0 ] } ],
  "nodes": [ { "name": "single_keyframe_node" } ],
  "animations":
  [
    {
      "name": "single_keyframe_translation",
      "channels": [ { "sampler": 0, "target": { "node": 0, "path": "translation" } } ],
      "samplers": [ { "input": 0, "output": 1, "interpolation": "LINEAR" } ]
    }
  ],
  "accessors":
  [
    { "bufferView": 0, "byteOffset": 0, "componentType": 5126, "count": 1, "type": "SCALAR" },
    { "bufferView": 1, "byteOffset": 0, "componentType": 5126, "count": 1, "type": "VEC3" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 4 },
    { "buffer": 0, "byteOffset": 4, "byteLength": 12 }
  ],
  "buffers": [ { "byteLength": 16, "uri": "placeholder.bin" } ]
}
"#;

/// Same shape as `assets/gltf/animated/single_keyframe_translation.gltf` ( the BUG-188
/// regression fixture, not reused directly per this task's Out of Scope ): 1 `f32` time, 1
/// `[f32; 3]` translation value.
fn single_keyframe_buffers() -> Vec< Vec< u8 > >
{
  let mut bytes = Vec::new();
  bytes.extend_from_slice( &0.0f32.to_le_bytes() );
  for c in [ 7.0f32, 8.0, 9.0 ]
  {
    bytes.extend_from_slice( &c.to_le_bytes() );
  }
  vec![ bytes ]
}

#[ test ]
fn channel_decode_reports_one_component_for_linear_interpolation()
{
  let gltf = gltf::Gltf::from_slice( LINEAR_TWO_KEYFRAME_FIXTURE.as_bytes() ).unwrap();
  let channel = first_channel( &gltf );
  let buffers = linear_two_keyframe_buffers();

  let ( components, times, values ) = channel_decode( &channel, &buffers )
  .expect( "well-formed Linear channel, must decode" );

  assert_eq!( components, 1 );
  assert_eq!( times, vec![ 0.0, 1.0 ] );
  assert!( matches!( values, ReadOutputs::Translations( _ ) ) );
}

#[ test ]
fn channel_decode_reports_three_components_for_cubicspline_interpolation()
{
  let gltf = gltf::Gltf::from_slice( CUBICSPLINE_TWO_KEYFRAME_FIXTURE.as_bytes() ).unwrap();
  let channel = first_channel( &gltf );
  // AF2: confirm the fixture's sampler really is CubicSpline, not a Linear channel mislabeled.
  assert_eq!( channel.sampler().interpolation(), gltf::animation::Interpolation::CubicSpline );
  let buffers = cubicspline_two_keyframe_buffers();

  let ( components, times, values ) = channel_decode( &channel, &buffers )
  .expect( "well-formed CubicSpline channel, must decode" );

  assert_eq!( components, 3 );
  assert_eq!( times.len(), 2 );
  assert!( matches!( values, ReadOutputs::Translations( _ ) ) );
}

#[ test ]
fn vec3_sequence_builds_two_tweens_matching_authored_translation_vectors()
{
  let gltf = gltf::Gltf::from_slice( LINEAR_TWO_KEYFRAME_FIXTURE.as_bytes() ).unwrap();
  let channel = first_channel( &gltf );
  let buffers = linear_two_keyframe_buffers();

  let sequence = vec3_sequence( &channel, &buffers )
  .expect( "well-formed 2-keyframe Linear channel, must build a Sequence" );

  let tweens = sequence.players();
  assert_eq!( tweens.len(), 2 );

  let first_value = F64x3::from_array( [ 1.0, 2.0, 3.0 ] );
  let second_value = F64x3::from_array( [ 4.0, 5.0, 6.0 ] );

  // Keyframe 1 has no predecessor, so its tween holds its own value ( zero duration ).
  assert_eq!( tweens[ 0 ].start_value, first_value );
  assert_eq!( tweens[ 0 ].end_value, first_value );
  // Keyframe 2's tween interpolates from keyframe 1's value to its own.
  assert_eq!( tweens[ 1 ].start_value, first_value );
  assert_eq!( tweens[ 1 ].end_value, second_value );
}

#[ test ]
fn vec3_sequence_duplicates_lone_tween_for_single_keyframe_channel()
{
  // BUG-188 regression precedent -- exercises the guard in `vec3_sequence` that duplicates a
  // lone tween so `Sequence::new`'s minimum-2 requirement doesn't silently drop the channel.
  let gltf = gltf::Gltf::from_slice( SINGLE_KEYFRAME_FIXTURE.as_bytes() ).unwrap();
  let channel = first_channel( &gltf );
  let buffers = single_keyframe_buffers();

  let sequence = vec3_sequence( &channel, &buffers )
  .expect( "single-keyframe channel must still build a Sequence, not be silently dropped" );

  // AF1: assert the tween count itself ( 2, not 1 ), not just that the call returned `Some` --
  // that alone would still pass even if the BUG-188 guard were silently removed.
  let tweens = sequence.players();
  assert_eq!( tweens.len(), 2 );

  let authored_value = F64x3::from_array( [ 7.0, 8.0, 9.0 ] );
  for tween in tweens
  {
    assert_eq!( tween.start_value, authored_value );
    assert_eq!( tween.end_value, authored_value );
  }
}

const MORPH_WEIGHTS_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "scene": 0,
  "scenes": [ { "nodes": [ 0 ] } ],
  "nodes": [ { "name": "morph_weights_node" } ],
  "animations":
  [
    {
      "name": "morph_weights_animation",
      "channels": [ { "sampler": 0, "target": { "node": 0, "path": "weights" } } ],
      "samplers": [ { "input": 0, "output": 1, "interpolation": "LINEAR" } ]
    }
  ],
  "accessors":
  [
    { "bufferView": 0, "byteOffset": 0, "componentType": 5126, "count": 2, "type": "SCALAR" },
    { "bufferView": 1, "byteOffset": 0, "componentType": 5126, "count": 2, "type": "SCALAR" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 8 },
    { "buffer": 0, "byteOffset": 8, "byteLength": 8 }
  ],
  "buffers": [ { "byteLength": 16, "uri": "placeholder.bin" } ]
}
"#;

/// Matches [`MORPH_WEIGHTS_FIXTURE`]: 2 `f32` times, then 2 `f32` weight values ( as if the glTF
/// encoded 1 morph target's worth of weight keyframes -- irrelevant to the BUG-262 regression
/// test below, which passes `targets == 0` explicitly regardless of what this buffer encodes ).
fn morph_weights_buffers() -> Vec< Vec< u8 > >
{
  let mut bytes = Vec::new();
  for t in [ 0.0f32, 1.0f32 ]
  {
    bytes.extend_from_slice( &t.to_le_bytes() );
  }
  for w in [ 0.5f32, 0.8f32 ]
  {
    bytes.extend_from_slice( &w.to_le_bytes() );
  }
  vec![ bytes ]
}

/// ## Root Cause
/// `weights_sequence` computed `weights.chunks( components * targets )` with no guard against
/// `targets == 0`. `[T]::chunks` panics unconditionally when given a chunk size of `0`. A glTF
/// mesh can legitimately carry morph-target animation channels while omitting the optional
/// `mesh.weights` default array; in that case `DisplacementsData::morph_weights_get()` stays
/// empty and `load()`'s `Property::MorphTargetWeights` arm passes that length straight through
/// as `targets`, reaching `weights_sequence` with `targets == 0`.
///
/// ## Why Not Caught
/// No test exercised `weights_sequence` at all prior to this bug -- `gltf_animation_loader_test.rs`
/// only covered `channel_decode`/`vec3_sequence`, and no existing glTF test fixture combined
/// morph-target animation channels with an absent `mesh.weights` array.
///
/// ## Fix Applied
/// `weights_sequence` now returns `None` immediately when `targets == 0`, before reaching
/// `.chunks(..)` -- every call site already handles `None` gracefully via
/// `let Some( sequence ) = weights_sequence( .. ) else { continue; }` ( BUG-262,
/// `animation/loaders/gltf.rs` ).
///
/// ## Prevention
/// This test calls `weights_sequence` directly with `targets == 0` against an otherwise
/// well-formed morph-weight channel and asserts it returns `None` instead of panicking.
///
/// ## Pitfall
/// A chunk-size parameter derived from optional/external input must be checked for zero before
/// use, even when the call site "usually" supplies a nonzero value -- `[T]::chunks( 0 )` panics
/// unconditionally rather than returning an empty iterator.
// test_kind: bug_reproducer(BUG-262)
#[ test ]
fn weights_sequence_returns_none_instead_of_panicking_when_targets_is_zero()
{
  let gltf = gltf::Gltf::from_slice( MORPH_WEIGHTS_FIXTURE.as_bytes() ).unwrap();
  let channel = first_channel( &gltf );
  let buffers = morph_weights_buffers();

  let result = weights_sequence( &channel, &buffers, 0 );
  assert!( result.is_none(), "targets == 0 must produce None instead of panicking inside `[T]::chunks`" );
}
