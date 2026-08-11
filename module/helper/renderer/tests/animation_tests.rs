//! Integration tests related to animations
#![ cfg( feature = "animation" ) ]

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use animation::{ Sequence, Tween, Sequencer, AnimatablePlayer };
  use renderer::webgl::
  {
    animation::base::
    {
      TRANSLATION_PREFIX,
      ROTATION_PREFIX,
      SCALE_PREFIX,
      MORPH_TARGET_PREFIX
    },
    loaders::gltf::{ GLTF, load }
  };

  async fn init_animation_test( gltf_path : &str ) -> GLTF
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );

    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();
    let window = gl::web_sys::window().unwrap();
    let document = window.document().unwrap();

    load( &document, gltf_path, &gl ).await.unwrap()
  }

  #[ wasm_bindgen_test( async ) ]
  async fn test_animation_loading()
  {
    let gltf = init_animation_test( "../../../../assets/gltf/animated/bug_bunny.glb" ).await;

    assert_eq!( gltf.animations.len(), 3 );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn test_morph_target_animation_loading()
  {
    let gltf = init_animation_test( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert_eq!( gltf.animations.len(), 1 );

    let animation = &gltf.animations[ 0 ];

    assert!( animation.nodes.len() > 0 );

    let sequencer = animation.animation.as_any().downcast_ref::< Sequencer >()
    .expect( "Animation is not Sequencer" );
    let keys = sequencer.keys();
    assert!( keys.iter().filter( | v | v.ends_with( MORPH_TARGET_PREFIX ) ).count() > 0 );
    assert!
    (
      keys
      .iter()
      .filter
      (
        | v |
        {
          v.ends_with( TRANSLATION_PREFIX ) ||
          v.ends_with( ROTATION_PREFIX ) ||
          v.ends_with( SCALE_PREFIX )
        }
      )
      .count() > 0
    );

    let morph_target_key = keys.iter().find( | v | v.ends_with( MORPH_TARGET_PREFIX ) ).unwrap();
    // The loader stores morph weights as `Sequence< Tween< Vec< f64 > > >`
    // (`weights_sequence` in `webgl/animation/loaders/gltf.rs`) — the tween
    // layer is part of the stored type, not an implementation detail.
    let morph_target_seq = sequencer.get::< Sequence< Tween< Vec< f64 > > > >( morph_target_key ).unwrap();

    assert!( !morph_target_seq.players().is_empty() );

    let morph_target_player = morph_target_seq.current_get().unwrap();

    // zophrac.glb ground truth: mesh 0 declares 53 morph targets and its
    // weights channel outputs 53 values per keyframe (5194 outputs across 98
    // keyframes) — checkable by parsing the glb's JSON chunk.
    assert_eq!( morph_target_player.value_get().len(), 53 );
  }
}
