//! Filter button generation for the UI

use web_sys::{ wasm_bindgen::JsCast };

/// Generates filter buttons in the UI grid
pub fn generate_filter_buttons()
{
  let filters =
  [
    // id,                display name
    ( "box-blur",         "Box Blur" ),
    ( "gaussian-blur",    "Gaussian Blur" ),
    ( "stack-blur",       "Stack Blur" ),
    ( "binarize",         "Binarize" ),
    ( "bcgimp",           "Brightness (GIMP)" ),
    ( "bcph",             "Brightness (PS)" ),
    ( "channels",         "Channels" ),
    ( "color-transform",  "Color Transform" ),
    ( "desaturate",       "Desaturate" ),
    ( "dither",           "Dither" ),
    ( "edge",             "Edge Detect" ),
    ( "emboss",           "Emboss" ),
    ( "enrich",           "Enrich" ),
    ( "flip",             "Flip" ),
    ( "gamma",            "Gamma" ),
    ( "grayscale",        "Grayscale" ),
    ( "hsl-adjust",       "HSL Adjust" ),
    ( "invert",           "Invert" ),
    ( "mosaic",           "Mosaic" ),
    ( "oil",              "Oil Paint" ),
    ( "posterize",        "Posterize" ),
    ( "rescale",          "Rescale" ),
    ( "resize-nn",        "Resize (NN)" ),
    ( "resize-bilinear",  "Resize (Bilinear)" ),
    ( "sepia",            "Sepia" ),
    ( "sharpen",          "Sharpen" ),
    ( "solarize",         "Solarize" ),
    ( "transpose",        "Transpose" ),
    ( "twirl",            "Twirl" ),
  ];

  let document = web_sys::window().unwrap().document().unwrap();
  let grid_container = document.get_element_by_id( "filters-grid" ).unwrap();

  for ( id, name ) in filters
  {
    create_filter_card( &document, &grid_container, id, name );
  }
}

/// Creates a single filter card element
fn create_filter_card
(
  document : &web_sys::Document,
  grid_container : &web_sys::Element,
  id : &str,
  name : &str
)
{
  // Create filter card
  let card = document.create_element( "div" ).unwrap();
  card.set_class_name( "filter-card" );
  card.set_id( id );

  // Create thumbnail container
  let thumbnail = document.create_element( "div" ).unwrap();
  thumbnail.set_class_name( "filter-thumbnail" );

  // Create img element
  let img = document.create_element( "img" ).unwrap();
  let img_element = img.dyn_into::< web_sys::HtmlImageElement >().unwrap();

  // Map filter ID to thumbnail filename
  let thumbnail_name = map_filter_id_to_thumbnail( id );
  let thumbnail_path = format!( "/assets/thumbnails/{}.png", thumbnail_name );
  img_element.set_src( &thumbnail_path );
  img_element.set_alt( name );
  img_element.set_class_name( "filter-thumbnail-img" );

  thumbnail.append_child( &img_element ).unwrap();

  // Create name label
  let label = document.create_element( "div" ).unwrap();
  label.set_class_name( "filter-name" );
  label.set_text_content( Some( name ) );

  // Assemble card
  card.append_child( &thumbnail ).unwrap();
  card.append_child( &label ).unwrap();
  grid_container.append_child( &card ).unwrap();
}

/// Maps filter ID to its corresponding thumbnail filename
fn map_filter_id_to_thumbnail( id : &str ) -> &str
{
  match id
  {
    "box-blur" => "boxblur",
    "gaussian-blur" => "gaussianblur",
    "stack-blur" => "stackblur",
    "binarize" => "binarize",
    "bcgimp" => "brightnessgimp",
    "bcph" => "brightnessps",
    "channels" => "channels",
    "color-transform" => "colortransform",
    "desaturate" => "desaturate",
    "dither" => "dither",
    "edge" => "edgedetection",
    "emboss" => "emboss",
    "enrich" => "enrich",
    "flip" => "flip",
    "gamma" => "gamma",
    "grayscale" => "grayscale",
    "hsl-adjust" => "hsl",
    "invert" => "invert",
    "mosaic" => "mosaic",
    "oil" => "oil",
    "posterize" => "posterize",
    "rescale" => "rescale",
    "resize-nn" => "resizenn",
    "resize-bilinear" => "resizebillinear",
    "sepia" => "sepia",
    "sharpen" => "sharpen",
    "solarize" => "solarize",
    "transpose" => "transpose",
    "twirl" => "twirl",
    _ => id,
  }
}
