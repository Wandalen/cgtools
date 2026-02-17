//! Jewelry configurator

mod lil_gui;

use minwebgl as gl;
use gl::{ web_sys, JsCast, canvas::HtmlCanvasElement };
use jewelry_configurator::{ JewelryRenderer, helpers::JewelryConfig };
use serde::{ Serialize, Deserialize };

#[ derive( Serialize, Deserialize, Clone, PartialEq ) ]
struct GuiParams
{
  gem_color : [ f32; 3 ],
  metal_color : [ f32; 3 ],
  clear_color : [ f32; 3 ],
  exposure : f32,
  roughness : f32,
  metalness : f32,
}

impl From< &JewelryConfig > for GuiParams
{
  fn from( config : &JewelryConfig ) -> Self
  {
    Self
    {
      gem_color : config.gem_color.into(),
      metal_color : config.metal_color.into(),
      clear_color : config.clear_color.into(),
      exposure : config.exposure,
      roughness : config.roughness,
      metalness : config.metalness,
    }
  }
}

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { run().await } );
}

async fn run()
{
  let window = web_sys::window().unwrap();
  let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
  let height = window.inner_height().unwrap().as_f64().unwrap() as u32;

  let document = window.document().unwrap();

  let canvas = document.get_element_by_id( "canvas" ).unwrap().unchecked_into::< HtmlCanvasElement >();
  canvas.set_width( width );
  canvas.set_height( height );

  let config = JewelryConfig::default();
  let params = GuiParams::from( &config );
  let params_obj = serde_wasm_bindgen::to_value( &params ).unwrap();

  let gui = lil_gui::new_gui();
  lil_gui::add_color( &gui, &params_obj, "gem_color" );
  lil_gui::add_color( &gui, &params_obj, "metal_color" );
  lil_gui::add_color( &gui, &params_obj, "clear_color" );
  lil_gui::add( &gui, &params_obj, "exposure", Some( 0.1 ), Some( 5.0 ), Some( 0.01 ) );
  lil_gui::add( &gui, &params_obj, "roughness", Some( 0.0 ), Some( 1.0 ), Some( 0.01 ) );
  lil_gui::add( &gui, &params_obj, "metalness", Some( 0.0 ), Some( 1.0 ), Some( 0.01 ) );

  let path = "gltf/1.glb";
  let mut renderer = JewelryRenderer::new( &canvas );
  renderer.init( "environment_maps/studio", "environment_maps/studio3/env-gem-4.hdr" ).await;
  renderer.load_jewelry_gltf( path ).await;

  let mut prev_time = 0.0;
  let mut prev_params = params;
  let run = move | t : f64 |
  {
    let delta_time = t - prev_time;
    prev_time = t;

    let current : GuiParams = serde_wasm_bindgen::from_value( params_obj.clone() ).unwrap();
    if current != prev_params
    {
      let [ r, g, b ] = current.gem_color;
      renderer.set_gem_color( r, g, b );
      let [ r, g, b ] = current.metal_color;
      renderer.set_metal_color( r, g, b );
      let [ r, g, b ] = current.clear_color;
      renderer.set_clear_color( r, g, b );
      renderer.set_exposure( current.exposure );
      renderer.set_roughness( current.roughness );
      renderer.set_metalness( current.metalness );
      prev_params = current;
    }

    renderer.render_jewelry( path, delta_time );
    true
  };

  gl::exec_loop::run( run );
}
