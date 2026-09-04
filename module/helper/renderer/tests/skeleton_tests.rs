//! Tests for Skeleton structure and related stuff
#![ cfg( feature = "animation" ) ]

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;

  // Browser, not Node: every test here needs a real WebGL2 context.
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use std::{ rc::Rc, cell::RefCell };
  // Fix(BUG-046): `Node` was used below (in `get_skeleton`) but missing from this import list.
  // Root cause: import list never updated when `get_skeleton`'s closure parameter was typed as
  // `Rc<RefCell<Node>>` — a compile error, so the whole test module never ran.
  // Pitfall: a missing-import compile error silently disables every test in the module; nextest
  // reports 0 tests collected rather than a loud per-test failure.
  use renderer::webgl::
  {
    Node,
    Object3D,
    texture_data_4f_load,
    loaders::gltf::load,
    skeleton::Skeleton
  };

  fn test_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let options = gl::context::ContextOptions::default().antialias( false );

    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  async fn skeleton_test_init( gltf_path : &str ) -> Skeleton
  {
    let gl = test_init();
    let window = gl::web_sys::window().unwrap();
    let document = window.document().unwrap();

    let gltf = load( &document, gltf_path, &gl ).await.unwrap();

    let mut skeleton = None;

    let mut get_skeleton =
    |
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      if let Object3D::Mesh( ref mesh ) = node.borrow().object
      {
        skeleton = mesh.borrow().skeleton.clone();
      }

      Ok( () )
    };

    // Fix(BUG-046): was `gltf.scene[ 0 ]` — `GLTF` has no `.scene` field, only `.scenes` (plural).
    // Root cause: written against an assumed singular field name never checked against `GLTF`'s
    // actual definition.
    // Pitfall: `.scene` vs `.scenes` is a one-character typo that the compiler catches loudly, but
    // only if the module compiles far enough to reach this line — the missing `Node` import above
    // masked this second error until the first was fixed.
    gltf.scenes[ 0 ].borrow().traverse( &mut get_skeleton ).unwrap();

    skeleton.unwrap().borrow().clone()
  }

  #[ wasm_bindgen_test( async ) ]
  async fn set_displacement_another_new_displacement_size_test()
  {
    let mut skeleton = skeleton_test_init( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert!
    (
      !skeleton.displacements_as_mut().as_mut().unwrap()
      .displacement_set
      (
        Some( [ [ 0.0; 3 ]; 2 ].to_vec() ),
        &gltf::Semantic::Tangents,
        2
      )
    );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn skeleton_clone_test()
  {
    let skeleton = skeleton_test_init( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    let skeleton_clone = skeleton.clone();

    assert_eq!( skeleton.has_skin(), skeleton_clone.has_skin() );
    assert_eq!( skeleton.has_morph_targets(), skeleton_clone.has_morph_targets() );
    assert_eq!( skeleton.displacements_as_ref().as_ref().unwrap().default_weights, skeleton_clone.displacements_as_ref().as_ref().unwrap().default_weights );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn skeleton_load_displacement_test()
  {
    let skeleton = skeleton_test_init( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert!( skeleton.displacements_as_ref().is_some() );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn skeleton_load_transform_test()
  {
    let skeleton = skeleton_test_init( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert!( skeleton.transforms_as_ref().is_some() );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn load_texture_data_4f_test()
  {
    let gl = test_init();

    let texture = gl.create_texture().unwrap();

    for a in ( 0..1024_u32 ).step_by( 256 )
    {
      let data = vec![ 0.0_f32; ( a * a ) as usize * 4 ];

      assert!( texture_data_4f_load( &gl, &texture, &data, [ a, a ] ).is_ok() );
    }
  }
}

// Pure-logic tests: no browser, no GL context. They live outside the
// wasm-only module above because the browser harness collects only
// `#[ wasm_bindgen_test ]` functions — a plain `#[ test ]` inside that
// module is silently dead on both targets (cfg'd out natively, never
// collected on wasm).
#[ cfg( not( target_arch = "wasm32" ) ) ]
#[ cfg( test ) ]
mod pure_tests
{
  use renderer::webgl::{ data_texture_size_calculate, skeleton::DisplacementsData, Node, loaders::gltf::skeleton_transforms_data_load };
  use std::{ rc::Rc, cell::RefCell };

  #[ test ]
  fn pack_displacements_data_test()
  {
    let mut displacements = DisplacementsData::new();

    let data = displacements.displacements_data_pack();

    assert_eq!( data.len(), 0 );

    displacements.displacement_set
    (
      Some( [ [ 1.0, 1.0, 1.0 ]; 16 ].to_vec() ),
      &gltf::Semantic::Positions,
      16
    );

    let data = displacements.displacements_data_pack();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 );
    assert_eq!( data.get( 0..4 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0 ] );

    displacements.displacement_set
    (
      Some( [ [ 2.0, 2.0, 2.0 ]; 16 ].to_vec() ),
      &gltf::Semantic::Normals,
      16
    );

    let data = displacements.displacements_data_pack();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 2 );
    assert_eq!( data.get( 0..8 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0 ] );

    displacements.displacement_set
    (
      Some( [ [ 3.0, 3.0, 3.0 ]; 16 ].to_vec() ),
      &gltf::Semantic::Tangents,
      16
    );

    let data = displacements.displacements_data_pack();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 3 );
    assert_eq!( data.get( 0..12 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 3.0, 3.0, 3.0, 1.0 ] );

    displacements.displacement_set( None, &gltf::Semantic::Normals, 16 );

    let data = displacements.displacements_data_pack();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 2 );
    assert_eq!( data.get( 0..8 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 3.0, 3.0, 3.0, 1.0 ] );
  }

  #[ test ]
  fn skinning_joints_resolve_by_index_not_name()
  {
    // BUG-173: joints used to be resolved by matching `joint.name()` against a name-keyed
    // map -- an unnamed joint node (name is optional per glTF spec) was silently dropped,
    // and two joint nodes sharing the same name collapsed to a single map entry. Both
    // failure modes corrupt the positional `JOINTS_0`/`JOINTS_1` vertex-attribute binding,
    // which depends on `skin.joints()`'s iteration position matching 1:1 with the resolved
    // node list. This fixture declares 3 joint nodes: one unnamed, two sharing the name
    // "Bone" -- the old code resolved at most 1 of the 3 (whichever "Bone" entry the
    // HashMap happened to retain last, and never the unnamed one); the fix resolves all 3,
    // each to the correct node by index.
    let zero_matrices_base64 = "AAAA".repeat( 64 ); // 192 zero bytes == 3 MAT4-shaped f32 slots

    let fixture = format!
    (
      r#"
      {{
        "asset": {{ "version": "2.0" }},
        "nodes": [ {{}}, {{ "name": "Bone" }}, {{ "name": "Bone" }} ],
        "skins": [ {{ "joints": [ 0, 1, 2 ], "inverseBindMatrices": 0 }} ],
        "accessors": [ {{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "MAT4" }} ],
        "bufferViews": [ {{ "buffer": 0, "byteOffset": 0, "byteLength": 192 }} ],
        "buffers": [ {{ "byteLength": 192, "uri": "data:application/octet-stream;base64,{zero_matrices_base64}" }} ]
      }}
      "#
    );

    let gltf = gltf::Gltf::from_slice( fixture.as_bytes() ).expect( "fixture must parse and validate" );
    let skin = gltf.skins().next().expect( "fixture declares one skin" );

    let nodes : Vec< Rc< RefCell< Node > > > = ( 0..3 ).map( | _ | Rc::new( RefCell::new( Node::new() ) ) ).collect();
    let buffers : Vec< Vec< u8 > > = vec![ vec![ 0u8; 192 ] ];

    let transforms = skeleton_transforms_data_load( &skin, &nodes, &buffers )
    .expect( "skin has an inverseBindMatrices accessor, must produce Some" );

    let joints = transforms.joints_get();
    assert_eq!( joints.len(), 3, "all 3 joints must resolve, including the unnamed node and both same-named nodes" );
    assert!( Rc::ptr_eq( &joints[ 0 ], &nodes[ 0 ] ), "joint 0 must resolve to the unnamed node at index 0" );
    assert!( Rc::ptr_eq( &joints[ 1 ], &nodes[ 1 ] ), "joint 1 must resolve to node index 1, not node index 2" );
    assert!( Rc::ptr_eq( &joints[ 2 ], &nodes[ 2 ] ), "joint 2 must resolve to node index 2, not node index 1" );
  }

  mod calculate_data_texture_size_tests
  {
    use super::data_texture_size_calculate;

    fn is_power_of_4( v : u32 ) -> bool
    {
      v.is_power_of_two() && v.trailing_zeros().is_multiple_of( 2 )
    }

    #[ test ]
    fn returns_power_of_4()
    {
      for data_size in [ 1, 2, 3, 4, 7, 16, 31, 64, 100, 257, 1024 ]
      {
        let size = data_texture_size_calculate( data_size );
        assert!
        (
          is_power_of_4( size ),
          "size={size} is not a power of 4 for data_size={data_size}"
        );
      }
    }

    #[ test ]
    fn square_fits_data()
    {
      for data_size in 1..10_000
      {
        let size = data_texture_size_calculate( data_size );
        let capacity = ( size as usize ) * ( size as usize );

        assert!
        (
          capacity >= data_size,
          "texture {size}x{size} cannot fit {data_size} elements"
        );
      }
    }

    #[ test ]
    fn is_minimal_power_of_4()
    {
      for data_size in [ 1, 5, 17, 63, 65, 255, 256, 257, 1023 ]
      {
        let size = data_texture_size_calculate( data_size );

        if size > 1
        {
          let smaller = size / 4;
          let smaller_capacity = ( smaller as usize ) * ( smaller as usize );

          assert!
          (
            smaller_capacity < data_size,
            "size={size} is not minimal for data_size={data_size}"
          );
        }
      }
    }

    // Replaces two never-executed tests that expected power-of-2 sides
    // ( f( 4 ) = 2, f( 17 ) = 8, ... ) — impossible alongside
    // `returns_power_of_4`, which passes against the real implementation.
    // The suite was dead ( plain `#[ test ]`s inside a wasm-only module ),
    // so the contradiction was never caught.
    #[ test ]
    fn power_of_four_boundaries()
    {
      // Spec: the side is the smallest power of 4 whose square fits
      // `data_size` ( sides stay powers of 4 so a matrix never straddles two
      // texture rows — see the doc comment on `data_texture_size_calculate` ).
      // An exact fit stays put; one element over jumps to the next power.
      let cases =
      [
        ( 1, 1 ),
        ( 2, 4 ),
        ( 16, 4 ),
        ( 17, 16 ),
        ( 256, 16 ),
        ( 257, 64 ),
        ( 4096, 64 ),
        ( 4097, 256 ),
      ];

      for ( data_size, expected_side ) in cases
      {
        let size = data_texture_size_calculate( data_size );
        assert_eq!
        (
          size,
          expected_side,
          "wrong side for data_size={data_size}"
        );
      }
    }

    #[ test ]
    fn zero_input_behavior_is_documented()
    {
      let size = data_texture_size_calculate( 0 );

      // Current behavior: log(0) → -inf → pow → 0
      // This test documents the behavior explicitly.
      assert_eq!( size, 0 );
    }
  }
}

