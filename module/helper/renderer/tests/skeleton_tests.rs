//! Tests for Skeleton structure and related stuff
#![ cfg( feature = "animation" ) ]

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgl as gl;
  use gl::GL;
  use std::{ rc::Rc, cell::RefCell };
  use renderer::webgl::
  {
    Object3D,
    calculate_data_texture_size,
    load_texture_data_4f,
    loaders::gltf::{ GLTF, load },
    skeleton::{ DisplacementsData, Skeleton, TransformsData }
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

    gltf.scene[ 0 ].borrow().traverse( &mut get_skeleton );

    skeleton.unwrap().borrow().clone()
  }

  #[ wasm_bindgen_test( async ) ]
  async fn set_displacement_another_new_displacement_size_test()
  {
    let skeleton = init_skeleton_test( "../../../../assets/gltf/animated/morph_targets/zophrac.glb" ).await;

    assert!
    (
      !skeleton.displacements_as_mut().unwrap()
      .set_displacement
      (
        Some( [ [ 0.0; 3 ]; 2 ].to_vec() ),
        gltf::Semantic::Tangents,
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
    assert_eq!( skeleton.displacements_as_ref().unwrap().default_weights, skeleton_clone.displacements_as_ref().unwrap().default_weights );
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

  #[ test ]
  fn pack_displacements_data_test()
  {
    let mut displacements = DisplacementsData::new();

    let data = displacements.pack_displacements_data();

    assert_eq!( data.len(), 0 );

    displacements.set_displacement
    (
      Some( [ [ 1.0, 1.0, 1.0 ]; 16 ].to_vec() ),
      gltf::Semantic::Positions,
      16
    );

    let data = displacements.pack_displacements_data();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 );
    assert_eq!( data.get( 0..4 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0 ] );

    displacements.set_displacement
    (
      Some( [ [ 2.0, 2.0, 2.0 ]; 16 ].to_vec() ),
      gltf::Semantic::Normals,
      16
    );

    let data = displacements.pack_displacements_data();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 2 );
    assert_eq!( data.get( 0..8 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0 ] );

    displacements.set_displacement
    (
      Some( [ [ 3.0, 3.0, 3.0 ]; 16 ].to_vec() ),
      gltf::Semantic::Tangents,
      16
    );

    let data = displacements.pack_displacements_data();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 3 );
    assert_eq!( data.get( 0..12 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 3.0, 3.0, 3.0, 1.0 ] );

    displacements.set_displacement( None, gltf::Semantic::Normals, 16 );

    let data = displacements.pack_displacements_data();

    assert_ne!( data.len(), 0 );
    assert_eq!( data.len(), 16 * 4 * 2 );
    assert_eq!( data.get( 0..8 ).unwrap(), &[ 1.0, 1.0, 1.0, 1.0, 3.0, 3.0, 3.0, 1.0 ] );
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

  mod calculate_data_texture_size_tests
  {
    use super::calculate_data_texture_size;

    fn is_power_of_4( v : u32 ) -> bool
    {
      v > 0 && ( v & ( v - 1 ) ) == 0 && ( v.trailing_zeros() % 2 == 0 )
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
          "size={} is not a power of 4 for data_size={}",
          size,
          data_size
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
          "texture {}x{} cannot fit {} elements",
          size,
          size,
          data_size
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
            "size={} is not minimal for data_size={}",
            size,
            data_size
          );
        }
      }
    }

    #[ test ]
    fn exact_square_boundaries()
    {
      let cases =
      [
        ( 1, 1 ),
        ( 4, 2 ),
        ( 16, 4 ),
        ( 64, 8 ),
        ( 256, 16 ),
        ( 1024, 32 ),
      ];

      for ( data_size, expected_side ) in cases
      {
        let size = calculate_data_texture_size( data_size );
        assert_eq!
        (
          size,
          expected_side,
          "wrong size for perfect square data_size={}",
          data_size
        );
      }
    }

    #[ test ]
    fn just_over_square_boundary()
    {
      let cases =
      [
        ( 2, 2 ),
        ( 5, 4 ),
        ( 17, 8 ),
        ( 65, 16 ),
        ( 257, 32 ),
      ];

      for ( data_size, expected_side ) in cases
       {
        let size = calculate_data_texture_size( data_size );
        assert_eq!
        (
          size,
          expected_side,
          "wrong size just over square boundary data_size={}",
          data_size
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

