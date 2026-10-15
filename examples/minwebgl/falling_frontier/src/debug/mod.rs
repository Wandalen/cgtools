mod grid_tuning;
mod grid_tuning_panel;
mod layers_panel;
mod render_layers;

pub use grid_tuning::GridTuning;
pub use render_layers::RenderLayers;
pub use grid_tuning_panel::{ setup_grid_tuning_panel, refresh_selection_status };
pub use layers_panel::{ setup_layers_panel, layers_panel_sync };

use minwebgl::web_sys::{ wasm_bindgen::JsCast, Document, HtmlInputElement };

/// The `input` element the panels themselves created under `id` - a missing
/// or differently typed element is a bug in the panel's own markup.
fn input_by_id( document : &Document, id : &str ) -> HtmlInputElement
{
  document.get_element_by_id( id ).unwrap().dyn_into::< HtmlInputElement >().unwrap()
}

/// Which bottom corner a dev panel sits in.
#[ derive( Clone, Copy ) ]
enum PanelSide
{
  Left,
  Right,
}

/// The fixed-position shell both dev panels are built in: `title` as a
/// clickable summary that collapses the panel to one line, `body` below it.
///
/// What the layout guarantees: a panel sits 12px from its bottom corner and
/// never grows above `100vh - 110px`, so it stays below the HUD's top bar
/// (Pause/Play/Fast, Reset Camera), scrolling instead. What it doesn't: on
/// a viewport narrower than about 480 CSS px the two panels, or a panel and
/// the HUD's unit card, can overlap side by side - collapse one.
fn panel_shell_html( id : &str, side : PanelSide, width_px : u32, title : &str, body : &str ) -> String
{
  let side = match side { PanelSide::Left => "left", PanelSide::Right => "right" };
  format!
  (
    r#"<div id="{id}" style="position:fixed;bottom:12px;{side}:12px;z-index:30;width:{width_px}px;max-height:calc(100vh - 110px);overflow-y:auto;
        background:rgba(8,17,26,0.9);border:1px solid #164e63;border-radius:8px;padding:10px;
        font-family:monospace;font-size:11px;color:#e0f2fe">
      <details open>
        <summary title="Click to collapse or expand" style="font-weight:bold;text-transform:uppercase;cursor:pointer;border-bottom:1px solid #164e63;padding-bottom:4px;margin-bottom:6px">{title}</summary>
        {body}
      </details>
    </div>"#
  )
}
