//! Integration tests related to animations

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  use minwebgl as gl;
  use renderer::webgl::loaders::gltf::load;

  #[ wasm_bindgen_test ]
  async fn test_animation_loading()
  {
    gl::browser::setup( Default::default() );
    let options = gl::context::ContexOptions::default().antialias( false );

    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();
    let window = gl::web_sys::window().unwrap();
    let document = window.document().unwrap();

    let gltf_path = "gltf/bug_bunny.glb";
    let gltf = load( &document, gltf_path, &gl ).await.unwrap();

    assert_eq!( gltf.animations.len(), 3 );
  }
}
