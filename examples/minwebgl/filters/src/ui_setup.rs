//! UI setup for filter controls
#![ allow( clippy::if_not_else ) ]


use crate::*;
use utils::*;
use filters::*;
use web_sys::
{
  wasm_bindgen,
  HtmlElement,
};
use wasm_bindgen::{ prelude::*, JsCast };
use std::{ cell::RefCell, rc::Rc } ;

fn show_controls_bar()
{
  controls::show();
}

pub fn hide_controls_bar()
{
  controls::hide();
  controls::clear_controls();
}

pub fn setup_ui( filter_renderer : &Rc< RefCell< Renderer > > ) -> Rc< RefCell< String > >
{
  let current_filter = Rc::new( RefCell::new( String::from( "none" ) ) );
  generate_filter_buttons();
  setup_filters_without_controls( filter_renderer, &current_filter );
  setup_filters_with_controls( filter_renderer, &current_filter );

  current_filter
}

fn generate_filter_buttons()
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
    let thumbnail_name = match id
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
    };
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
}

fn setup_filters_without_controls( filter_renderer : &Rc< RefCell< Renderer > >, current_filter : &Rc< RefCell< String > > )
{
  // Filters that don't have parameters
  let filters =
  [
    ( "desaturate",  make_closure_with_filter_tracking( filter_renderer, desaturate::Desaturate, "desaturate", current_filter ) ),
    ( "edge",        make_closure_with_filter_tracking( filter_renderer, edge::Edge, "edge", current_filter ) ),
    ( "emboss",      make_closure_with_filter_tracking( filter_renderer, emboss::Emboss, "emboss", current_filter ) ),
    ( "enrich",      make_closure_with_filter_tracking( filter_renderer, enrich::Enrich, "enrich", current_filter ) ),
    ( "grayscale",   make_closure_with_filter_tracking( filter_renderer, gray_scale::GrayScale, "grayscale", current_filter ) ),
    ( "invert",      make_closure_with_filter_tracking( filter_renderer, invert::Invert, "invert", current_filter ) ),
    ( "sepia",       make_closure_with_filter_tracking( filter_renderer, sepia::Sepia, "sepia", current_filter ) ),
    ( "solarize",    make_closure_with_filter_tracking( filter_renderer, solarize::Solarize, "solarize", current_filter ) ),
    ( "transpose",   make_closure_with_filter_tracking( filter_renderer, transpose::Transpose, "transpose", current_filter ) ),
  ];

  for ( card_id, closure ) in filters
  {
    let card = get_element_by_id_unchecked::< web_sys::HtmlElement >( card_id );
    card.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
}

// Macro to reduce repetition in filter setup
macro_rules! setup_filter_with_sliders {
  (
    $filter_renderer:expr,
    $current_filter:expr,
    $card_id:expr,
    $filter_type:ty,
    $initial_value:expr,
    [ $( ($label:expr, $prop:expr, $min:expr, $max:expr, $step:expr) ),+ ]
  ) => {{
    let filter_renderer_clone = $filter_renderer.clone();
    let current_filter_clone = $current_filter.clone();
    let onclick : Closure< dyn Fn() > = Closure::new( move ||
    {
      *current_filter_clone.borrow_mut() = String::from( $card_id );
      filter_renderer_clone.borrow_mut().save_previous_texture();

      // Clear and setup controls
      controls::clear_controls();

      // Add sliders
      $(
        let initial_val = serde_wasm_bindgen::to_value( &$initial_value ).unwrap();
        let obj_map = initial_val.dyn_into::< web_sys::js_sys::Object >().unwrap();
        let val = web_sys::js_sys::Reflect::get( &obj_map, &JsValue::from_str( $prop ) ).unwrap();
        let num_val = val.as_f64().unwrap_or( $min );
        controls::add_slider( $label, $prop, num_val, $min, $max, $step );
      )+

      // Apply initial filter
      filter_renderer_clone.borrow_mut().apply_filter( &$initial_value );

      // Setup onChange callback
      let fr = filter_renderer_clone.clone();
      let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
      {
        let filter : $filter_type = serde_wasm_bindgen::from_value( values ).unwrap();
        fr.borrow_mut().apply_filter( &filter );
      });
      controls::on_change( callback.as_ref().unchecked_ref() );
      callback.forget();

      show_controls_bar();
    });

    let card = get_element_by_id_unchecked::< HtmlElement >( $card_id );
    card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
    onclick.forget();
  }};
}

// Helper for blur filters (they have generic type parameters)
fn setup_blur_filter< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  _label : &str,
  blur_type : T,
  max : f64
)
where
blur::Blur< T > : Filter
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let card_id_str = card_id.to_string();
  let blur_type_init = blur_type.clone();

  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    *current_filter_clone.borrow_mut() = card_id_str.clone();
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();
    controls::add_slider( "Size", "size", 5.0, 1.0, max, 1.0 );

    let initial = blur::Blur::new( 5, blur_type_init.clone() );
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let blur_type_change = blur_type_init.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "size" ) ).unwrap();
      let size = val.as_f64().unwrap() as i32;

      let filter = blur::Blur::new( size, blur_type_change.clone() );
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    show_controls_bar();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

// Helper for resize filters (they have generic type parameters)
fn setup_resize_filter< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  _label : &str,
  resize_type : T
)
where
resize::Resize< T > : Filter
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let card_id_str = card_id.to_string();
  let resize_type_init = resize_type.clone();

  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    *current_filter_clone.borrow_mut() = card_id_str.clone();
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();
    controls::add_slider( "Scale", "scale", 1.0, 0.1, 10.0, 0.01 );

    let initial = resize::Resize::new( 1.0_f32, resize_type_init.clone() );
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let resize_type_change = resize_type_init.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "scale" ) ).unwrap();
      let scale = val.as_f64().unwrap() as f32;

      let filter = resize::Resize::new( scale, resize_type_change.clone() );
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    show_controls_bar();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

// Helper for brightness/contrast filters (they have generic type parameters)
fn setup_brightness_contrast_filter< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  _label : &str,
  bc_type : T,
  min : f64,
  max : f64,
  step : f64
)
where
brightness_contrast::BrightnessContrast< T > : Filter
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let card_id_str = card_id.to_string();
  let bc_type_init = bc_type.clone();

  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    *current_filter_clone.borrow_mut() = card_id_str.clone();
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();
    controls::add_slider( "Brightness", "brightness", 0.0, min, max, step );
    controls::add_slider( "Contrast", "contrast", 0.0, min, max, step );

    let initial = brightness_contrast::BrightnessContrast::new( 0.0, 0.0, bc_type_init.clone() );
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let bc_type_change = bc_type_init.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let brightness_val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "brightness" ) ).unwrap();
      let contrast_val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "contrast" ) ).unwrap();
      let brightness = brightness_val.as_f64().unwrap();
      let contrast = contrast_val.as_f64().unwrap();

      let filter = brightness_contrast::BrightnessContrast::new( brightness as f32, contrast as f32, bc_type_change.clone() );
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    show_controls_bar();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

fn setup_filters_with_controls( filter_renderer : &Rc< RefCell< Renderer > >, current_filter : &Rc< RefCell< String > > )
{
  // Blur filters need manual setup due to generic type parameters
  setup_blur_filter( filter_renderer, current_filter, "box-blur", "Box Blur", blur::Box, 80.0 );
  setup_blur_filter( filter_renderer, current_filter, "gaussian-blur", "Gaussian Blur", blur::Gaussian, 50.0 );
  setup_blur_filter( filter_renderer, current_filter, "stack-blur", "Stack Blur", blur::Stack, 80.0 );

  // Binarize
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "binarize",
    binarize::Binarize,
    binarize::Binarize { threshold: 0.5 },
    [ ("Threshold", "threshold", 0.0, 1.0, 0.001) ]
  );

  // Rescale
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "rescale",
    rescale::Rescale,
    rescale::Rescale { scale: 1.0 },
    [ ("Scale", "scale", 0.1, 10.0, 0.01) ]
  );

  // Resize filters need manual setup due to generic type parameters
  setup_resize_filter( filter_renderer, current_filter, "resize-nn", "Resize (NN)", resize::Nearest );
  setup_resize_filter( filter_renderer, current_filter, "resize-bilinear", "Resize (Bilinear)", resize::Bilinear );

  // Sharpen
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "sharpen",
    sharpen::Sharpen,
    sharpen::Sharpen { factor: 1.0 },
    [ ("Factor", "factor", 1.0, 10.0, 0.1) ]
  );

  // Dithering
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "dither",
    dithering::Dithering,
    dithering::Dithering { levels: 4 },
    [ ("Levels", "levels", 2.0, 20.0, 1.0) ]
  );

  // Posterize
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "posterize",
    posterize::Posterize,
    posterize::Posterize { levels: 4 },
    [ ("Levels", "levels", 2.0, 20.0, 1.0) ]
  );

  // Gamma
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "gamma",
    gamma::Gamma,
    gamma::Gamma { gamma: 1.0 },
    [ ("Gamma", "gamma", 0.1, 5.0, 0.01) ]
  );

  // Mosaic
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "mosaic",
    mosaic::Mosaic,
    mosaic::Mosaic { scale: 10 },
    [ ("Scale", "scale", 1.0, 100.0, 1.0) ]
  );

  // BrightnessContrast filters need manual setup due to generic type parameters
  setup_brightness_contrast_filter( filter_renderer, current_filter, "bcgimp", "BC (GIMP)", brightness_contrast::GIMP, -100.0, 100.0, 1.0 );
  setup_brightness_contrast_filter( filter_renderer, current_filter, "bcph", "BC (PS)", brightness_contrast::Photoshop, -1.0, 1.0, 0.01 );

  // Color Transform
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "color-transform",
    color_transform::ColorTransform,
    color_transform::ColorTransform
    {
      red_multiplier: 1.0,
      green_multiplier: 1.0,
      blue_multiplier: 1.0,
      red_offset: 0.0,
      green_offset: 0.0,
      blue_offset: 0.0,
    },
    [
      ("Red Mult", "redMultiplier", 0.0, 2.0, 0.01),
      ("Green Mult", "greenMultiplier", 0.0, 2.0, 0.01),
      ("Blue Mult", "blueMultiplier", 0.0, 2.0, 0.01),
      ("Red Offset", "redOffset", -1.0, 1.0, 0.01),
      ("Green Offset", "greenOffset", -1.0, 1.0, 0.01),
      ("Blue Offset", "blueOffset", -1.0, 1.0, 0.01)
    ]
  );

  // HSL Adjust
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "hsl-adjust",
    hsl_adjustment::HSLAdjustment,
    hsl_adjustment::HSLAdjustment
    {
      hue: 0.0,
      saturation: 0.0,
      lightness: 0.0,
    },
    [
      ("Hue", "hue", -1.0, 1.0, 0.01),
      ("Saturation", "saturation", -1.0, 1.0, 0.01),
      ("Lightness", "lightness", -1.0, 1.0, 0.01)
    ]
  );

  // Oil
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "oil",
    oil::Oil,
    oil::Oil { levels: 10, range: 3 },
    [
      ("Levels", "levels", 2.0, 50.0, 1.0),
      ("Range", "range", 1.0, 15.0, 1.0)
    ]
  );

  // Twirl
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "twirl",
    twirl::Twirl,
    twirl::Twirl
    {
      center_x: 0.5,
      center_y: 0.5,
      radius: 0.3,
      strength: 10.0,
    },
    [
      ("Center X", "centerX", 0.0, 1.0, 0.01),
      ("Center Y", "centerY", 0.0, 1.0, 0.01),
      ("Radius", "radius", 0.0, 1.0, 0.01),
      ("Strength", "strength", -200.0, 200.0, 1.0)
    ]
  );

  // Channels (dropdown)
  setup_channels_filter( filter_renderer, current_filter );

  // Flip (dropdown)
  setup_flip_filter( filter_renderer, current_filter );
}

fn setup_channels_filter( filter_renderer : &Rc< RefCell< Renderer > >, current_filter : &Rc< RefCell< String > > )
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    *current_filter_clone.borrow_mut() = String::from( "channels" );
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();

    let options = web_sys::js_sys::Array::of3( &"Red".into(), &"Green".into(), &"Blue".into() );
    controls::add_dropdown( "Channel", "channel", "Red", &options.into() );

    let initial = channels::Channels { channel: channels::Channel::Red };
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      // Get the channel value from the values object
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "channel" ) ).unwrap();
      let channel_str = val.as_string().unwrap();

      // Parse string to enum
      let channel = match channel_str.as_str()
      {
        "Red" => channels::Channel::Red,
        "Green" => channels::Channel::Green,
        "Blue" => channels::Channel::Blue,
        _ => channels::Channel::Red,
      };

      let filter = channels::Channels { channel };
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    show_controls_bar();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( "channels" );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

fn setup_flip_filter( filter_renderer : &Rc< RefCell< Renderer > >, current_filter : &Rc< RefCell< String > > )
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    *current_filter_clone.borrow_mut() = String::from( "flip" );
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();

    let options = web_sys::js_sys::Array::of3( &"FlipX".into(), &"FlipY".into(), &"FlipXY".into() );
    controls::add_dropdown( "Direction", "flip", "FlipX", &options.into() );

    let initial = flip::Flip { flip: flip::FlipDirection::FlipX };
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      // Get the flip value from the values object
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "flip" ) ).unwrap();
      let flip_str = val.as_string().unwrap();

      // Parse string to enum
      let flip = match flip_str.as_str()
      {
        "FlipX" => flip::FlipDirection::FlipX,
        "FlipY" => flip::FlipDirection::FlipY,
        "FlipXY" => flip::FlipDirection::FlipXY,
        _ => flip::FlipDirection::FlipX,
      };

      let filter = flip::Flip { flip };
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    show_controls_bar();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( "flip" );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

fn make_closure_with_filter_tracking
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  filter : impl Filter + 'static,
  filter_name : &str,
  current_filter : &Rc< RefCell< String > >
)
-> Closure< dyn Fn() >
{
  let filter_renderer = filter_renderer.clone();
  let current_filter = current_filter.clone();
  let filter_name = filter_name.to_string();
  Closure::new( Box::new( move ||
  {
    *current_filter.borrow_mut() = filter_name.clone();
    filter_renderer.borrow_mut().save_previous_texture();
    controls::clear_controls();
    filter_renderer.borrow_mut().apply_filter( &filter );
    show_controls_bar();
  }))
}
