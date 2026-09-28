mod grid_tuning;
mod grid_tuning_panel;
mod layers_panel;
mod render_layers;

pub use grid_tuning::GridTuning;
pub use render_layers::RenderLayers;
pub use grid_tuning_panel::{ setup_grid_tuning_panel, refresh_selection_status };
pub use layers_panel::setup_layers_panel;

use minwebgl::web_sys::{ wasm_bindgen::JsCast, Document, HtmlInputElement };

/// The `input` element the panels themselves created under `id` - a missing
/// or differently typed element is a bug in the panel's own markup.
fn input_by_id( document : &Document, id : &str ) -> HtmlInputElement
{
  document.get_element_by_id( id ).unwrap().dyn_into::< HtmlInputElement >().unwrap()
}
