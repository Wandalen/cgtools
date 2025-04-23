mod render;
mod model;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext as GL, WebGlUniformLocation,
};
use glow::{
    Context as GlowContext, HasContext, Buffer, Program, Texture, VertexArray, Framebuffer
};
use nalgebra_glm as glm;

#[wasm_bindgen]
pub struct App {
    model_reader: model::Reader,
    renderer: Renderer,
}

#[wasm_bindgen]
impl App{
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, width: i32, height: i32) -> Result<Self, JsValue> {
        let mut app = Self{
            model_reader: model::Reader::new(),
            renderer: Renderer::new()
        };


        
        Ok(app)
    }

    // --- Main Loop Tick ---
    #[wasm_bindgen]
    pub fn tick(&mut self, timestamp: f64){

    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: i32, height: i32){

    }
}

// --- Wasm Entry Point ---
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    Ok(())
}

