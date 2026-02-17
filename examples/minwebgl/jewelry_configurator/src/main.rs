//! Jewelry configurator

mod lil_gui;

use minwebgl as gl;
use gl::{ web_sys, JsCast, canvas::HtmlCanvasElement };
use std::{ cell::RefCell, rc::Rc };
use wasm_bindgen::prelude::*;
use jewelry_configurator::{ JewelryRenderer, helpers::JewelryConfig };
use serde::{ Serialize, Deserialize };

const MODELS : &[ &str ] = &[ "gltf/0.glb", "gltf/1.glb", "gltf/2.glb" ];

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

  // GUI setup
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

  // Renderer setup
  let mut renderer = JewelryRenderer::new( &canvas );
  renderer.init( "environment_maps/studio", "environment_maps/studio3/env-gem-4.hdr" ).await;

  // Load the default model
  let default_index = 1_usize;
  renderer.load_jewelry_gltf( MODELS[ default_index ] ).await;

  let renderer = Rc::new( RefCell::new( renderer ) );

  // Track which model is active and which are already loaded
  let active_model : Rc< RefCell< usize > > = Rc::new( RefCell::new( default_index ) );
  let loaded : Rc< RefCell< Vec< bool > > > = Rc::new( RefCell::new( vec![ false; MODELS.len() ] ) );
  loaded.borrow_mut()[ default_index ] = true;

  setup_buttons( &document, &renderer, &active_model, &loaded );

  // Render loop
  let mut prev_time = 0.0;
  let mut prev_params = params;
  let run = move | t : f64 |
  {
    let delta_time = t - prev_time;
    prev_time = t;

    // Skip frame if renderer is borrowed by async loading task
    let Ok( mut renderer ) = renderer.try_borrow_mut() else { return true; };

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

    let idx = *active_model.borrow();
    renderer.render_jewelry( MODELS[ idx ], delta_time );
    true
  };

  gl::exec_loop::run( run );
}

/// Sets up click handlers on model selection buttons.
/// Closures are leaked via `forget` so they live for the lifetime of the page.
fn setup_buttons
(
  document : &web_sys::Document,
  renderer : &Rc< RefCell< JewelryRenderer > >,
  active_model : &Rc< RefCell< usize > >,
  loaded : &Rc< RefCell< Vec< bool > > >,
)
{
  let buttons : Vec< web_sys::HtmlElement > = ( 0..MODELS.len() )
  .map( | i | document.get_element_by_id( &format!( "btn-{i}" ) ).unwrap().unchecked_into() )
  .collect();

  for i in 0..MODELS.len()
  {
    let renderer = renderer.clone();
    let active_model = active_model.clone();
    let loaded = loaded.clone();
    let buttons_clone = buttons.clone();

    let closure = Closure::< dyn FnMut() >::new( move ||
    {
      // Update active button styling
      for btn in &buttons_clone
      {
        btn.class_list().remove_1( "active" ).ok();
      }
      buttons_clone[ i ].class_list().add_1( "active" ).ok();

      // If already loaded, just switch
      if loaded.borrow()[ i ]
      {
        *active_model.borrow_mut() = i;
        return;
      }

      // Otherwise load asynchronously
      let renderer = renderer.clone();
      let active_model = active_model.clone();
      let loaded = loaded.clone();

      gl::spawn_local( async move
      {
        let path = MODELS[ i ];
        renderer.borrow_mut().load_jewelry_gltf( path ).await;
        loaded.borrow_mut()[ i ] = true;
        *active_model.borrow_mut() = i;
      });
    });

    buttons[ i ].set_onclick( Some( closure.as_ref().unchecked_ref() ) );
    closure.forget();
  }
}
