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
    load_texture_data_4f,
    loaders::gltf::load,
    skeleton::Skeleton
  };

  async fn init_test() -> GL
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContextOptions::default().antialias( false );

    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, options ).unwrap()
  }

  async fn init_skeleton_test( gltf_path : &str ) -> Skeleton
  {
    let gl = init_test().await;
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
    gltf.scenes[ 0 ].borrow().traverse( &mut get_skeleton );

    skeleton.unwrap().borrow().clone()
  }

  #[ wasm_bindgen_test( async ) ]
  async fn set_displacement_another_new_displacement_size_test()
  {
    let mut skeleton = init_skeleton_test( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert!
    (
      !skeleton.displacements_as_mut().as_mut().unwrap()
      .set_displacement
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
    let skeleton = init_skeleton_test( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    let skeleton_clone = skeleton.clone();

    assert_eq!( skeleton.has_skin(), skeleton_clone.has_skin() );
    assert_eq!( skeleton.has_morph_targets(), skeleton_clone.has_morph_targets() );
    assert_eq!( skeleton.displacements_as_ref().as_ref().unwrap().default_weights, skeleton_clone.displacements_as_ref().as_ref().unwrap().default_weights );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn skeleton_load_displacement_test()
  {
    let skeleton = init_skeleton_test( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert!( skeleton.displacements_as_ref().is_some() );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn skeleton_load_transform_test()
  {
    let skeleton = init_skeleton_test( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert!( skeleton.transforms_as_ref().is_some() );
  }

  #[ wasm_bindgen_test( async ) ]
  async fn load_texture_data_4f_test()
  {
    let gl = init_test().await;

    let texture = gl.create_texture().unwrap();

    for a in ( 0..1024_u32 ).step_by( 256 )
    {
      let data = vec![ 0.0_f32; ( a * a ) as usize * 4 ];

      assert!( load_texture_data_4f( &gl, &texture, &data, [ a, a ] ).is_ok() );
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
  use renderer::webgl::{ calculate_data_texture_size, skeleton::DisplacementsData };

  #[ test ]
  fn pack_displacements_data_test()
  {
    let mut displacements = DisplacementsData::new();

    let data = displacements.pack_displacements_data();

    assert_eq!( data.len(), 0 );

    displacements.set_displacement
    (
      Some( [ [ 1.0, 1.0, 1.0 ]; 16 ].to_vec() ),
      &gltf::Semantic::Positions,
      16
    );

    let data = displacements.pack_displacements_data();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 );
    assert_eq!( data.get( 0..4 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0 ] );

    displacements.set_displacement
    (
      Some( [ [ 2.0, 2.0, 2.0 ]; 16 ].to_vec() ),
      &gltf::Semantic::Normals,
      16
    );

    let data = displacements.pack_displacements_data();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 2 );
    assert_eq!( data.get( 0..8 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0 ] );

    displacements.set_displacement
    (
      Some( [ [ 3.0, 3.0, 3.0 ]; 16 ].to_vec() ),
      &gltf::Semantic::Tangents,
      16
    );

    let data = displacements.pack_displacements_data();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 3 );
    assert_eq!( data.get( 0..12 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 3.0, 3.0, 3.0, 1.0 ] );

    displacements.set_displacement( None, &gltf::Semantic::Normals, 16 );

    let data = displacements.pack_displacements_data();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 2 );
    assert_eq!( data.get( 0..8 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 3.0, 3.0, 3.0, 1.0 ] );
  }

  mod calculate_data_texture_size_tests
  {
    use super::calculate_data_texture_size;

    fn is_power_of_4( v : u32 ) -> bool
    {
      v.is_power_of_two() && v.trailing_zeros().is_multiple_of( 2 )
    }

    #[ test ]
    fn returns_power_of_4()
    {
      for data_size in [ 1, 2, 3, 4, 7, 16, 31, 64, 100, 257, 1024 ]
      {
        let size = calculate_data_texture_size( data_size );
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
        let size = calculate_data_texture_size( data_size );
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
        let size = calculate_data_texture_size( data_size );

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
      // texture rows — see the doc comment on `calculate_data_texture_size` ).
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
        let size = calculate_data_texture_size( data_size );
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
      let size = calculate_data_texture_size( 0 );

      // Current behavior: log(0) → -inf → pow → 0
      // This test documents the behavior explicitly.
      assert_eq!( size, 0 );
    }
  }
}

