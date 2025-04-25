mod input;
mod model;
mod render;

use input::*;
use model::*;
use render::*;

use js_sys::wasm_bindgen::{
  JsCast,
  JsValue,
};
use nalgebra_glm as glm;
use wasm_bindgen::{
  prelude::*,
  JsCast,
};
use web_sys::{
  HtmlCanvasElement,
  WebGl2RenderingContext as GL,
  WebGlUniformLocation,
};

#[ wasm_bindgen ]
pub struct App
{
  model_reader : model::Reader,
  renderer : Renderer,
}

#[ wasm_bindgen ]
impl App
{
  #[ wasm_bindgen( constructor ) ]
  pub fn new(
    canvas_id : &str,
    width : i32,
    height : i32,
  ) -> Result< Self, JsValue >
  {
    let webgl_context = get_webgl_context( canvas_id )?;

    let mut app = Self {
      model_reader : model::Reader::new(),
      renderer : Renderer::new( webgl_context, Viewport::new( width, height ) ),
    };

    Ok( app )
  }

  // --- Main Loop Tick ---
  #[ wasm_bindgen ]
  pub fn tick(
    &mut self,
    input : JsValue,
  )
  {
    let Ok( input_state ) = input.dyn_into::< InputState >()
    else
    {
      return;
    };

    self.renderer.update( input_state );
    self.renderer.render();
  }
}

fn get_webgl_context( canvas_id : &str ) -> Result< GL, String >
{
  let document = web_sys::window()
    .ok_or( AppError::WebGlContext )?
    .document()
    .ok_or( AppError::WebGlContext )?;
  let canvas = document
    .get_element_by_id( canvas_id )
    .ok_or( AppError::WebGlContext )?
    .dyn_into::< HtmlCanvasElement >()
    .map_err(| _ | AppError::WebGlContext )?;

  let options = serde_wasm_bindgen::to_value( &serde_json::json!( { "antialias" : false } ) )
    .map_err( | _ | AppError::WebGlContext )?;

  let webgl_context = canvas
    .get_context_with_context_options( "webgl2", &options ) // No AA needed for object buffer
    .map_err( | _ | AppError::WebGlContext )?
    .ok_or( AppError::WebGlContext )?
    .dyn_into::<GL>()
    .map_err( | _ | AppError::WebGlContext )?;

  Ok( webgl_context )
}

// --- Wasm Entry Point ---
#[ wasm_bindgen( start ) ]
pub fn start() -> Result< (), JsValue >
{
  Ok( () )
}
