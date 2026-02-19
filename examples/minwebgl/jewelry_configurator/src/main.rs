//! Jewelry configurator — browser entry point.
//!
//! Sets up the canvas, builds the lil-gui control panel from [`JewelryConfig`],
//! exposes `window.jewelryParams` / `window.colorsJson` / `window.jewelryGui` for the
//! HTML color-swatch buttons, loads the default model, and drives the render loop.
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::std_instead_of_alloc ) ]

mod lil_gui;

use minwebgl as gl;
use gl::{ web_sys, JsCast, canvas::HtmlCanvasElement, browser::Config };
use web_sys::js_sys;
use std::rc::Rc;
use core::cell::RefCell;
use wasm_bindgen::prelude::*;
use jewelry_configurator::{ JewelryRenderer, JewelryConfig };

const MODELS : &[ &str ] = &[ "gltf/0.glb", "gltf/1.glb", "gltf/2.glb" ];
const COLORS_JSON : &str = include_str!( "../colors.json" );

fn main()
{
  gl::browser::setup( Config::default() );
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
  let params_obj = serde_wasm_bindgen::to_value( &config ).unwrap();

  js_sys::Reflect::set( window.as_ref(), &"jewelryParams".into(), &params_obj ).ok();
  js_sys::Reflect::set( window.as_ref(), &"colorsJson".into(), &JsValue::from_str( COLORS_JSON ) ).ok();

  let gui = lil_gui::new_gui();
  js_sys::Reflect::set( window.as_ref(), &"jewelryGui".into(), &gui ).ok();
  lil_gui::add_color( &gui, &params_obj, "gem_color" );
  lil_gui::add_color( &gui, &params_obj, "metal_color" );
  lil_gui::add( &gui, &params_obj, "clear_color", Some( 0.0 ), Some( 5.0 ), Some( 0.01 ) );
  lil_gui::add( &gui, &params_obj, "exposure", Some( 0.1 ), Some( 5.0 ), Some( 0.01 ) );
  lil_gui::add( &gui, &params_obj, "roughness", Some( 0.0 ), Some( 1.0 ), Some( 0.01 ) );
  lil_gui::add( &gui, &params_obj, "metalness", Some( 0.0 ), Some( 1.0 ), Some( 0.01 ) );

  // Renderer setup
  let mut renderer = JewelryRenderer::new( &canvas, "environment_maps/studio", "environment_maps/studio3/env-gem-4.hdr", false ).await;

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
  let mut prev_config = config;
  let run = move | time : f64 |
  {
    let delta_time = time - prev_time;
    prev_time = time;

    // Skip frame if renderer is borrowed by async loading task
    let Ok( mut renderer ) = renderer.try_borrow_mut() else { return true; };

    let current : JewelryConfig = serde_wasm_bindgen::from_value( params_obj.clone() ).unwrap();
    if current != prev_config
    {
      renderer.update_config( current );
      prev_config = current;
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
