//! Tests for animation modifier Mirror
#![ cfg( feature = "animation" ) ]

use animation::{ Sequence, Sequencer, Tween, easing::{ EasingBuilder, Linear } };
use mingl::{ F64x3, QuatF64 };
use renderer::webgl::animation::
{
  Mirror,
  MirrorPlane,
  base::
  {
    MORPH_TARGET_PREFIX,
    ROTATION_PREFIX,
    SCALE_PREFIX,
    TRANSLATION_PREFIX
  }
};

fn create_animation() -> Sequencer
{
  let mut animation = Sequencer::new();

  let linear = Linear::new();
  animation.insert
  (
    TRANSLATION_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( F64x3::splat( -1.0 ), F64x3::splat( 0.0 ), 0.5, linear.clone() ),
        Tween::new( F64x3::splat( 0.0 ), F64x3::splat( 1.0 ), 0.5, linear ).with_delay( 0.5 )
      ]
    ).unwrap()
  );

  let linear = Linear::new();
  animation.insert
  (
    ROTATION_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( QuatF64::from( [ -1.0, -1.0, -1.0, 1.0 ] ), QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ), 0.5, linear.clone() ),
        Tween::new( QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ), QuatF64::from( [ 1.0, 1.0, 1.0, 1.0 ] ), 0.5, linear ).with_delay( 0.5 )
      ]
    ).unwrap()
  );

  let linear = Linear::new();
  animation.insert
  (
    SCALE_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( F64x3::splat( 1.0 ), F64x3::splat( 2.0 ), 0.5, linear.clone() ),
        Tween::new( F64x3::splat( 2.0 ), F64x3::splat( 3.0 ), 0.5, linear ).with_delay( 0.5 )
      ]
    ).unwrap()
  );

  let linear = Linear::new();
  animation.insert
  (
    MORPH_TARGET_PREFIX,
    Sequence::new
    (
      vec!
      [
        Tween::new( vec![ 0.5, 0.5, 0.5 ], vec![ 0.75, 0.75, 0.75 ], 0.5, linear.clone() ),
        Tween::new( vec![ 0.75, 0.75, 0.75 ], vec![ 1.0, 1.0, 1.0 ], 0.5, linear ).with_delay( 0.5 )
      ]
    ).unwrap()
  );

  animation
}

#[ test ]
fn transition_mirroring_test()
{
  let animation = create_animation();

  let animation_xy = Mirror::along_plane( &animation, MirrorPlane::XY );
  let animation_xz = Mirror::along_plane( &animation, MirrorPlane::XZ );
  let animation_yz = Mirror::along_plane( &animation, MirrorPlane::YZ );

  let sequence = animation_xy.get::< Sequence< Tween< F64x3 > > >( TRANSLATION_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, F64x3::from_array( [ -1.0, -1.0, 1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, F64x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
  assert_eq!( players[ 1 ].start_value, F64x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
  assert_eq!( players[ 1 ].end_value, F64x3::from_array( [ 1.0, 1.0, -1.0 ] ) );

  let sequence = animation_xz.get::< Sequence< Tween< F64x3 > > >( TRANSLATION_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, F64x3::from_array( [ -1.0, 1.0, -1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, F64x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
  assert_eq!( players[ 1 ].start_value, F64x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
  assert_eq!( players[ 1 ].end_value, F64x3::from_array( [ 1.0, -1.0, 1.0 ] ) );

  let sequence = animation_yz.get::< Sequence< Tween< F64x3 > > >( TRANSLATION_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, F64x3::from_array( [ 1.0, -1.0, -1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, F64x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
  assert_eq!( players[ 1 ].start_value, F64x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
  assert_eq!( players[ 1 ].end_value, F64x3::from_array( [ -1.0, 1.0, 1.0 ] ) );
}

#[ test ]
fn rotation_mirroring_test()
{
  let animation = create_animation();

  let animation_xy = Mirror::along_plane( &animation, MirrorPlane::XY );
  let animation_xz = Mirror::along_plane( &animation, MirrorPlane::XZ );
  let animation_yz = Mirror::along_plane( &animation, MirrorPlane::YZ );

  let sequence = animation_xy.get::< Sequence< Tween< QuatF64 > > >( ROTATION_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, QuatF64::from( [ 1.0, 1.0, -1.0, 1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ) );
  assert_eq!( players[ 1 ].start_value, QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ) );
  assert_eq!( players[ 1 ].end_value, QuatF64::from( [ -1.0, -1.0, 1.0, 1.0 ] ) );

  let sequence = animation_xz.get::< Sequence< Tween< QuatF64 > > >( ROTATION_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, QuatF64::from( [ 1.0, -1.0, 1.0, 1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ) );
  assert_eq!( players[ 1 ].start_value, QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ) );
  assert_eq!( players[ 1 ].end_value, QuatF64::from( [ -1.0, 1.0, -1.0, 1.0 ] ) );

  let sequence = animation_yz.get::< Sequence< Tween< QuatF64 > > >( ROTATION_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, QuatF64::from( [ -1.0, 1.0, 1.0, 1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ) );
  assert_eq!( players[ 1 ].start_value, QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ) );
  assert_eq!( players[ 1 ].end_value, QuatF64::from( [ 1.0, -1.0, -1.0, 1.0 ] ) );
}

#[ test ]
fn scale_mirroring_test()
{
  let animation = create_animation();

  let animation_xy = Mirror::along_plane( &animation, MirrorPlane::XY );
  let animation_xz = Mirror::along_plane( &animation, MirrorPlane::XZ );
  let animation_yz = Mirror::along_plane( &animation, MirrorPlane::YZ );

  let sequence = animation_xy.get::< Sequence< Tween< F64x3 > > >( SCALE_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, F64x3::from_array( [ 1.0, 1.0, 1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, F64x3::from_array( [ 2.0, 2.0, 2.0 ] ) );
  assert_eq!( players[ 1 ].start_value, F64x3::from_array( [ 2.0, 2.0, 2.0 ] ) );
  assert_eq!( players[ 1 ].end_value, F64x3::from_array( [ 3.0, 3.0, 3.0 ] ) );

  let sequence = animation_xz.get::< Sequence< Tween< F64x3 > > >( SCALE_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, F64x3::from_array( [ 1.0, 1.0, 1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, F64x3::from_array( [ 2.0, 2.0, 2.0 ] ) );
  assert_eq!( players[ 1 ].start_value, F64x3::from_array( [ 2.0, 2.0, 2.0 ] ) );
  assert_eq!( players[ 1 ].end_value, F64x3::from_array( [ 3.0, 3.0, 3.0 ] ) );

  let sequence = animation_yz.get::< Sequence< Tween< F64x3 > > >( SCALE_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, F64x3::from_array( [ 1.0, 1.0, 1.0 ] ) );
  assert_eq!( players[ 0 ].end_value, F64x3::from_array( [ 2.0, 2.0, 2.0 ] ) );
  assert_eq!( players[ 1 ].start_value, F64x3::from_array( [ 2.0, 2.0, 2.0 ] ) );
  assert_eq!( players[ 1 ].end_value, F64x3::from_array( [ 3.0, 3.0, 3.0 ] ) );
}

#[ test ]
fn morph_targets_mirroring_test()
{
  let animation = create_animation();

  let animation_xy = Mirror::along_plane( &animation, MirrorPlane::XY );
  let animation_xz = Mirror::along_plane( &animation, MirrorPlane::XZ );
  let animation_yz = Mirror::along_plane( &animation, MirrorPlane::YZ );

  let sequence = animation_xy.get::< Sequence< Tween< Vec< f64 > > > >( MORPH_TARGET_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, vec![ 0.5, 0.5, 0.5 ] );
  assert_eq!( players[ 0 ].end_value, vec![ 0.75, 0.75, 0.75 ] );
  assert_eq!( players[ 1 ].start_value, vec![ 0.75, 0.75, 0.75 ] );
  assert_eq!( players[ 1 ].end_value, vec![ 1.0, 1.0, 1.0 ] );

  let sequence = animation_xz.get::< Sequence< Tween< Vec< f64 > > > >( MORPH_TARGET_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, vec![ 0.5, 0.5, 0.5 ] );
  assert_eq!( players[ 0 ].end_value, vec![ 0.75, 0.75, 0.75 ] );
  assert_eq!( players[ 1 ].start_value, vec![ 0.75, 0.75, 0.75 ] );
  assert_eq!( players[ 1 ].end_value, vec![ 1.0, 1.0, 1.0 ] );

  let sequence = animation_yz.get::< Sequence< Tween< Vec< f64 > > > >( MORPH_TARGET_PREFIX ).unwrap();
  let players = sequence.players();
  assert_eq!( players[ 0 ].start_value, vec![ 0.5, 0.5, 0.5 ] );
  assert_eq!( players[ 0 ].end_value, vec![ 0.75, 0.75, 0.75 ] );
  assert_eq!( players[ 1 ].start_value, vec![ 0.75, 0.75, 0.75 ] );
  assert_eq!( players[ 1 ].end_value, vec![ 1.0, 1.0, 1.0 ] );
}
