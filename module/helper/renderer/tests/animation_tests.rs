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
  use animation::{ Sequence, Tween, Sequencer };
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

  async fn animation_test_init( gltf_path : &str ) -> GLTF
  {
    gl::browser::setup( gl::browser::Config::default() );
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
    let gltf = animation_test_init( "../../../../assets/gltf/animated/bug_bunny.glb" ).await;

    assert_eq!( gltf.animations.len(), 3 );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn test_morph_target_animation_loading()
  {
    let gltf = animation_test_init( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert_eq!( gltf.animations.len(), 1 );

    let animation = &gltf.animations[ 0 ];

    assert!( !animation.nodes.is_empty() );

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

  /// BUG-188's fixture as a self-contained `data:` URI: one node whose translation channel has
  /// exactly one keyframe, `( 1.5, 2.5, -3.5 )` at time 0. The file this test first read,
  /// `assets/gltf/animated/single_keyframe_translation.gltf`, was never added to the DVC
  /// remote, so no checkout had it.
  fn single_keyframe_translation_uri() -> String
  {
    let floats = [ 0.0_f32, 1.5, 2.5, -3.5 ];
    let bytes : Vec< u8 > = floats.iter().flat_map( | f | f.to_le_bytes() ).collect();
    // `btoa` encodes a "binary string", one char per byte.
    let window = gl::web_sys::window().unwrap();
    let buffer = window.btoa( &bytes.iter().map( | &b | char::from( b ) ).collect::< String >() ).unwrap();
    let json = format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        "scene" : 0,
        "scenes" : [ {{ "nodes" : [ 0 ] }} ],
        "nodes" : [ {{ "name" : "probe" }} ],
        "buffers" : [ {{ "byteLength" : 16, "uri" : "data:application/octet-stream;base64,{buffer}" }} ],
        "bufferViews" : [
          {{ "buffer" : 0, "byteOffset" : 0, "byteLength" : 4 }},
          {{ "buffer" : 0, "byteOffset" : 4, "byteLength" : 12 }}
        ],
        "accessors" : [
          {{ "bufferView" : 0, "componentType" : 5126, "count" : 1, "type" : "SCALAR", "min" : [ 0.0 ], "max" : [ 0.0 ] }},
          {{ "bufferView" : 1, "componentType" : 5126, "count" : 1, "type" : "VEC3" }}
        ],
        "animations" : [ {{
          "samplers" : [ {{ "input" : 0, "output" : 1, "interpolation" : "LINEAR" }} ],
          "channels" : [ {{ "sampler" : 0, "target" : {{ "node" : 0, "path" : "translation" }} }} ]
        }} ]
      }}"#
    );
    format!( "data:model/gltf+json;base64,{}", window.btoa( &json ).unwrap() )
  }

  #[ wasm_bindgen_test( async ) ]
  async fn test_single_keyframe_translation_not_dropped()
  {
    let gltf = animation_test_init( &single_keyframe_translation_uri() ).await;

    assert_eq!( gltf.animations.len(), 1 );

    let animation = &gltf.animations[ 0 ];

    let sequencer = animation.animation.as_any().downcast_ref::< Sequencer >()
    .expect( "Animation is not Sequencer" );
    let keys = sequencer.keys();

    // BUG-188 regression: a channel with exactly one keyframe used to be silently dropped --
    // `Sequence::new` rejects fewer than 2 players and the loader's `.ok()` swallowed the
    // resulting error with no diagnostic anywhere. If the channel got dropped, no key ending
    // in TRANSLATION_PREFIX exists at all.
    let translation_key = keys.iter().find( | v | v.ends_with( TRANSLATION_PREFIX ) )
    .expect( "BUG-188 regression: single-keyframe translation channel was silently dropped" );

    let translation_seq = sequencer.get::< Sequence< Tween< gl::F64x3 > > >( translation_key )
    .expect( "Sequencer key exists but is not the expected Sequence< Tween< F64x3 > > type" );

    // Post-fix shape: the lone tween is duplicated to satisfy Sequence::new's minimum-2
    // requirement, holding the single authored value.
    assert_eq!( translation_seq.players().len(), 2 );

    let current = translation_seq.current_get().expect( "Sequence has no current player" );
    assert_eq!( current.value_get(), gl::F64x3::from_array( [ 1.5, 2.5, -3.5 ] ) );
  }
}
