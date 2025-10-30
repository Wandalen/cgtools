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
use serde::de::DeserializeOwned;

fn show_apply_cancel_buttons()
{
  if let Some( apply_btn ) = web_sys::window()
    .and_then( | w | w.document() )
    .and_then( | d | d.get_element_by_id( "apply-btn" ) )
  {
    let _ = apply_btn.class_list().add_1( "visible" );
  }
  if let Some( cancel_btn ) = web_sys::window()
    .and_then( | w | w.document() )
    .and_then( | d | d.get_element_by_id( "cancel-btn" ) )
  {
    let _ = cancel_btn.class_list().add_1( "visible" );
  }
}

pub fn hide_apply_cancel_buttons()
{
  if let Some( apply_btn ) = web_sys::window()
    .and_then( | w | w.document() )
    .and_then( | d | d.get_element_by_id( "apply-btn" ) )
  {
    let _ = apply_btn.class_list().remove_1( "visible" );
  }
  if let Some( cancel_btn ) = web_sys::window()
    .and_then( | w | w.document() )
    .and_then( | d | d.get_element_by_id( "cancel-btn" ) )
  {
    let _ = cancel_btn.class_list().remove_1( "visible" );
  }
}

pub fn setup_ui( filter_renderer : &Rc< RefCell< Renderer > > ) -> Rc< RefCell< String > >
{
  let current_filter = Rc::new( RefCell::new( String::from( "none" ) ) );
  let gui = lil_gui::new_gui();
  generate_filter_buttons( &gui );
  setup_filters_without_gui( filter_renderer, &current_filter );
  setup_filters_with_gui( &gui, filter_renderer, &current_filter );

  // Hide all GUI folders initially
  show_only( "", &gui );

  current_filter
}

fn generate_filter_buttons( gui : &JsValue )
{
  let filters =
  [
    // id,                title of control to show,       display name,                     icon
    ( "box-blur",         "Box Blur",                     "Box Blur",                        "🔲" ),
    ( "gaussian-blur",    "Gaussian Blur",                "Gaussian Blur",                   "🌫️" ),
    ( "stack-blur",       "Stack Blur",                   "Stack Blur",                      "📚" ),
    ( "binarize",         "Binarize",                     "Binarize",                        "⚫" ),
    ( "bcgimp",           "BrightnessContrast GIMP",      "Brightness (GIMP)",               "☀️" ),
    ( "bcph",             "BrightnessContrast Photoshop", "Brightness (PS)",                 "💡" ),
    ( "channels",         "Channels",                     "Channels",                        "🎨" ),
    ( "color-transform",  "Color Transform",              "Color Transform",                 "🌈" ),
    ( "desaturate",       "",                             "Desaturate",                      "⚪" ),
    ( "dither",           "Dithering",                    "Dither",                          "🎲" ),
    ( "edge",             "",                             "Edge Detect",                     "📐" ),
    ( "emboss",           "",                             "Emboss",                          "🔨" ),
    ( "enrich",           "",                             "Enrich",                          "✨" ),
    ( "flip",             "Flip",                         "Flip",                            "🔄" ),
    ( "gamma",            "Gamma",                        "Gamma",                           "⚡" ),
    ( "grayscale",        "",                             "Grayscale",                       "⬜" ),
    ( "hsl-adjust",       "HSL Adjust",                   "HSL Adjust",                      "🎚️" ),
    ( "invert",           "",                             "Invert",                          "🔁" ),
    ( "mosaic",           "Mosaic",                       "Mosaic",                          "🔷" ),
    ( "oil",              "Oil",                          "Oil Paint",                       "🖌️" ),
    ( "posterize",        "Posterize",                    "Posterize",                       "🎭" ),
    ( "rescale",          "Rescale",                      "Rescale",                         "📏" ),
    ( "resize-nn",        "Resize (Nearest)",             "Resize (NN)",                     "🔍" ),
    ( "resize-bilinear",  "Resize (Bilinear)",            "Resize (Bilinear)",               "🔎" ),
    ( "sepia",            "",                             "Sepia",                           "📷" ),
    ( "sharpen",          "Sharpen",                      "Sharpen",                         "🔪" ),
    ( "solarize",         "",                             "Solarize",                        "☀️" ),
    ( "transpose",        "",                             "Transpose",                       "🔀" ),
    ( "twirl",            "Twirl",                        "Twirl",                           "🌀" ),
  ];

  let document = web_sys::window().unwrap().document().unwrap();
  let grid_container = document.get_element_by_id( "filters-grid" ).unwrap();

  for ( id, control_title, name, icon ) in filters
  {
    // Create filter card
    let card = document.create_element( "div" ).unwrap();
    card.set_class_name( "filter-card" );
    card.set_id( id );

    // Create thumbnail
    let thumbnail = document.create_element( "div" ).unwrap();
    thumbnail.set_class_name( "filter-thumbnail" );
    thumbnail.set_text_content( Some( icon ) );

    // Create name label
    let label = document.create_element( "div" ).unwrap();
    label.set_class_name( "filter-name" );
    label.set_text_content( Some( name ) );

    // Assemble card
    card.append_child( &thumbnail ).unwrap();
    card.append_child( &label ).unwrap();
    grid_container.append_child( &card ).unwrap();

    // Add click handler to show/hide GUI controls
    let gui = gui.clone();
    let onclick : Closure< dyn Fn() > = Closure::new( move || show_only( control_title, &gui ) );
    card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
    onclick.forget();
  }
}

fn show_only( target : &str, gui : &JsValue )
{
  let folders = lil_gui::get_folders( gui );
  for folder in folders
  {
    if lil_gui::get_title( &folder ) != target
    {
      lil_gui::hide( &folder );
    }
    else
    {
      lil_gui::show( &folder );
    }
  }
}

fn setup_filters_without_gui( filter_renderer : &Rc< RefCell< Renderer > >, current_filter : &Rc< RefCell< String > > )
{
  // These are filter card ids and their respective filters
  // It's done for filters that don't contain any parameters
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

  // Basically sets draw call on card click with respective filter
  for ( card_id, closure ) in filters
  {
    let card = get_element_by_id_unchecked::< web_sys::HtmlElement >( card_id );
    card.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
}

fn setup_filters_with_gui( gui : &JsValue, filter_renderer : &Rc< RefCell< Renderer > >, current_filter : &Rc< RefCell< String > > )
{
  // Setup filters that uses sliders for value adjustment
  let filters =
  [
    (
      "Box Blur",                                                                   // title of the gui
      onchange_closure::< blur::Blur< blur::Box > >( filter_renderer ),             // onchange callback
      serde_wasm_bindgen::to_value( &blur::Blur::new( 1, blur::Box ) ).unwrap(),    // object that holds values
      vec![ ( "size", 1.0, 80.0, 1.0 ) ],                                           // slider data: property, min, max, step
      "box-blur",                                                                   // id of button that triggers filter
      onclick_closure::< blur::Blur< blur::Box > >( filter_renderer )               // button's onclick callback
    ),
    (
      "Gaussian Blur",
      onchange_closure::< blur::Blur< blur::Gaussian > >( filter_renderer ),
      serde_wasm_bindgen::to_value( &blur::Blur::new( 1, blur::Gaussian ) ).unwrap(),
      vec![ ( "size", 1.0, 50.0, 1.0 ) ],
      "gaussian-blur",
      onclick_closure::< blur::Blur< blur::Gaussian > >( filter_renderer )
    ),
    (
      "Stack Blur",
      onchange_closure::< blur::Blur< blur::Stack > >( filter_renderer ),
      serde_wasm_bindgen::to_value( &blur::Blur::new( 1, blur::Stack ) ).unwrap(),
      vec![ ( "size", 1.0, 80.0, 1.0 ) ],
      "stack-blur",
      onclick_closure::< blur::Blur< blur::Stack > >( filter_renderer )
    ),
    (
      "Binarize",
      onchange_closure::< binarize::Binarize >( filter_renderer ),
      serde_wasm_bindgen::to_value( &binarize::Binarize { threshold: 0.5 } ).unwrap(),
      vec![ ( "threshold", 0.0, 1.0, 0.001 ) ],
      "binarize",
      onclick_closure::< binarize::Binarize >( filter_renderer )
    ),
    (
      "Rescale",
      onchange_closure::< rescale::Rescale >( filter_renderer ),
      serde_wasm_bindgen::to_value( &rescale::Rescale { scale: 1.0 } ).unwrap(),
      vec![ ( "scale", 0.0, 10.0, 0.001 ) ],
      "rescale",
      onclick_closure::< rescale::Rescale >( filter_renderer )
    ),
    (
      "Resize (Nearest)",
      onchange_closure::< resize::Resize< resize::Nearest > >( filter_renderer ),
      serde_wasm_bindgen::to_value( &resize::Resize::new( 1.0, resize::Nearest ) ).unwrap(),
      vec![ ( "scale", 0.0, 10.0, 0.001 ) ],
      "resize-nn",
      onclick_closure::< resize::Resize< resize::Nearest > >( filter_renderer )
    ),
    (
      "Resize (Bilinear)",
      onchange_closure::< resize::Resize< resize::Bilinear > >( filter_renderer ),
      serde_wasm_bindgen::to_value( &resize::Resize::new( 1.0, resize::Bilinear ) ).unwrap(),
      vec![ ( "scale", 0.0, 10.0, 0.001 ) ],
      "resize-bilinear",
      onclick_closure::< resize::Resize< resize::Bilinear > >( filter_renderer )
    ),
    (
      "Sharpen",
      onchange_closure::< sharpen::Sharpen >( filter_renderer ),
      serde_wasm_bindgen::to_value( &sharpen::Sharpen { factor: 1.0 } ).unwrap(),
      vec![ ( "factor", 1.0, 10.0, 1.0 ) ],
      "sharpen",
      onclick_closure::< sharpen::Sharpen >( filter_renderer )
    ),
    (
      "Dithering",
      onchange_closure::< dithering::Dithering >( filter_renderer ),
      serde_wasm_bindgen::to_value( &dithering::Dithering { levels: 2 } ).unwrap(),
      vec![ ( "levels", 2.0, 20.0, 1.0 ) ],
      "dither",
      onclick_closure::< dithering::Dithering >( filter_renderer )
    ),
    (
      "Posterize",
      onchange_closure::< posterize::Posterize >( filter_renderer ),
      serde_wasm_bindgen::to_value( &posterize::Posterize { levels: 2 } ).unwrap(),
      vec![ ( "levels", 2.0, 20.0, 1.0 ) ],
      "posterize",
      onclick_closure::< posterize::Posterize >( filter_renderer )
    ),
    (
      "Gamma",
      onchange_closure::< gamma::Gamma >( filter_renderer ),
      serde_wasm_bindgen::to_value( &gamma::Gamma { gamma: 1.0 } ).unwrap(),
      vec![ ( "gamma", 0.0, 5.0, 0.001 ) ],
      "gamma",
      onclick_closure::< gamma::Gamma >( filter_renderer )
    ),
    (
      "Mosaic",
      onchange_closure::< mosaic::Mosaic >( filter_renderer ),
      serde_wasm_bindgen::to_value( &mosaic::Mosaic { scale: 1 } ).unwrap(),
      vec![ ( "scale", 1.0, 100.0, 1.0 ) ],
      "mosaic",
      onclick_closure::< mosaic::Mosaic >( filter_renderer )
    ),
    (
      "BrightnessContrast GIMP",
      onchange_closure::
      < brightness_contrast::BrightnessContrast< brightness_contrast::GIMP > >( filter_renderer ),
      serde_wasm_bindgen::to_value
      (
        &brightness_contrast::BrightnessContrast::new
        (
          0.0,
          0.0,
          brightness_contrast::GIMP
        )
      ).unwrap(),
      vec!
      [
        ( "brightness", -100.0, 100.0, 1.0 ),
        ( "contrast", -100.0, 100.0, 1.0 ),
      ],
      "bcgimp",
      onclick_closure::
      < brightness_contrast::BrightnessContrast< brightness_contrast::GIMP > >( filter_renderer )
    ),
    (
      "BrightnessContrast Photoshop",
      onchange_closure::
      < brightness_contrast::BrightnessContrast< brightness_contrast::Photoshop > >( filter_renderer ),
      serde_wasm_bindgen::to_value
      (
        &brightness_contrast::BrightnessContrast::new
        (
          0.0,
          0.0,
          brightness_contrast::Photoshop
        )
      ).unwrap(),
      vec!
      [
        ( "brightness", -1.0, 1.0, 0.001 ),
        ( "contrast", -1.0, 1.0, 0.001 ),
      ],
      "bcph",
      onclick_closure::
      < brightness_contrast::BrightnessContrast< brightness_contrast::Photoshop > >( filter_renderer )
    ),
    (
      "Color Transform",
      onchange_closure::< color_transform::ColorTransform >( filter_renderer ),
      serde_wasm_bindgen::to_value
      (
        &color_transform::ColorTransform
        {
          red_multiplier: 1.0,
          green_multiplier: 1.0,
          blue_multiplier: 1.0,
          red_offset: 0.0,
          green_offset: 0.0,
          blue_offset: 0.0,
        }
      ).unwrap(),
      vec!
      [
        ( "redMultiplier", 0.0, 10.0, 0.001 ),
        ( "greenMultiplier", 0.0, 10.0, 0.001 ),
        ( "blueMultiplier", 0.0, 10.0, 0.001 ),
        ( "redOffset", -1.0, 1.0, 0.001 ),
        ( "greenOffset", -1.0, 1.0, 0.001 ),
        ( "blueOffset", -1.0, 1.0, 0.001 ),
      ],
      "color-transform",
      onclick_closure::< color_transform::ColorTransform >( filter_renderer )
    ),
    (
      "HSL Adjust",
      onchange_closure::< hsl_adjustment::HSLAdjustment >( filter_renderer ),
      serde_wasm_bindgen::to_value
      (
        &hsl_adjustment::HSLAdjustment
        {
          hue: 0.0,
          saturation: 0.0,
          lightness: 0.0,
        }
      ).unwrap(),
      vec!
      [
        ( "hue", -1.0, 1.0, 0.001 ),
        ( "saturation", -1.0, 1.0, 0.001 ),
        ( "lightness", -1.0, 1.0, 0.001 ),
      ],
      "hsl-adjust",
      onclick_closure::< hsl_adjustment::HSLAdjustment >( filter_renderer )
    ),
    (
      "Oil",
      onchange_closure::< oil::Oil >( filter_renderer ),
      serde_wasm_bindgen::to_value
      (
        &oil::Oil
        {
          levels: 2,
          range: 1,
        }
      ).unwrap(),
      vec!
      [
        ( "levels", 2.0, 50.0, 1.0 ),
        ( "range", 1.0, 15.0, 1.0 ),
      ],
      "oil",
      onclick_closure::< oil::Oil >( filter_renderer )
    ),
    (
      "Twirl",
      onchange_closure::< twirl::Twirl >( filter_renderer ),
      serde_wasm_bindgen::to_value
      (
        &twirl::Twirl
        {
          center_x: 0.5,
          center_y: 0.5,
          radius: 0.1,
          strength: 1.0,
        }
      ).unwrap(),
      vec!
      [
        ( "centerX", 0.0, 1.0, 0.001 ),
        ( "centerY", 0.0, 1.0, 0.001 ),
        ( "radius", 0.0, 2.0, 0.001 ),
        ( "strength", -200.0, 200.0, 0.1 ),
      ],
      "twirl",
      onclick_closure::< twirl::Twirl >( filter_renderer )
    ),
  ];

  for ( title, closure, obj, sliders, button_id, onclick ) in filters
  {
    let gui = lil_gui::add_folder( &gui, title );
    for ( prop, min, max, step ) in sliders
    {
      lil_gui::add_slider( &gui, &obj, prop, min, max, step );
    }
    lil_gui::on_finish_change( &gui, closure.as_ref().unchecked_ref() );
    let filter_card = get_element_by_id_unchecked::< HtmlElement >( button_id );
    let current_filter_clone = current_filter.clone();
    let button_id_str = button_id.to_string();
    let filter_renderer_clone = filter_renderer.clone();
    let onclick : Closure< dyn Fn() > = Closure::new
    (
      {
        let obj = obj.clone();
        move ||
        {
          *current_filter_clone.borrow_mut() = button_id_str.clone();
          filter_renderer_clone.borrow_mut().save_previous_texture();
          ( onclick )( &obj );
          show_apply_cancel_buttons();
        }
      }
    );
    filter_card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
    onclick.forget();
    closure.forget();
  }

  // These two filters use dropdowns instead of sliders so they are setup manually

  let channels_gui = lil_gui::add_folder( &gui, "Channels" );
  let closure = onchange_closure::< channels::Channels >( filter_renderer );
  let obj = serde_wasm_bindgen::to_value( &channels::Channels { channel: channels::Channel::Red } ).unwrap();
  let options = web_sys::js_sys::Array::of3( &"Red".into(), &"Green".into(), &"Blue".into() );
  lil_gui::add_dropdown( &channels_gui, &obj, "channel", &options.into() );
  lil_gui::on_finish_change( &channels_gui, closure.as_ref().unchecked_ref() );
  let filter_card = get_element_by_id_unchecked::< HtmlElement >( "channels" );
  let current_filter_channels = current_filter.clone();
  let filter_renderer_channels = filter_renderer.clone();
  let onclick : Closure< dyn Fn() > = Closure::new
  (
    {
      let obj = obj.clone();
      let filter_renderer = filter_renderer.clone();
      move ||
      {
        *current_filter_channels.borrow_mut() = String::from( "channels" );
        filter_renderer_channels.borrow_mut().save_previous_texture();
        let onclick = onclick_closure::< channels::Channels >( &filter_renderer );
        ( onclick )( &obj );
        show_apply_cancel_buttons();
      }
    }
  );
  filter_card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
  closure.forget();

  let flip_gui = lil_gui::add_folder( &gui, "Flip" );
  let closure = onchange_closure::< flip::Flip >( filter_renderer );
  let obj = serde_wasm_bindgen::to_value( &flip::Flip { flip: flip::FlipDirection::FlipX } ).unwrap();
  let options = web_sys::js_sys::Array::of3( &"FlipX".into(), &"FlipY".into(), &"FlipXY".into() );
  lil_gui::add_dropdown( &flip_gui, &obj, "flip", &options.into() );
  lil_gui::on_finish_change( &flip_gui, closure.as_ref().unchecked_ref() );
  let filter_card = get_element_by_id_unchecked::< HtmlElement >( "flip" );
  let current_filter_flip = current_filter.clone();
  let filter_renderer_flip = filter_renderer.clone();
  let onclick : Closure< dyn Fn() > = Closure::new
  (
    {
      let obj = obj.clone();
      let filter_renderer = filter_renderer.clone();
      move ||
      {
        *current_filter_flip.borrow_mut() = String::from( "flip" );
        filter_renderer_flip.borrow_mut().save_previous_texture();
        let onclick = onclick_closure::< flip::Flip >( &filter_renderer );
        ( onclick )( &obj );
        show_apply_cancel_buttons();
      }
    }
  );
  filter_card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
  closure.forget();
}

fn onchange_closure< T >( filter_renderer : &Rc< RefCell< Renderer > > )
-> Closure< dyn Fn( JsValue ) >
where
T : Filter + DeserializeOwned
{
  let filter_renderer = filter_renderer.clone();
  Closure::new
  (
    move | obj : JsValue |
    {
      let filter : T = serde_wasm_bindgen::from_value( obj ).unwrap();
      filter_renderer.borrow_mut().apply_filter( &filter );
    }
  )
}

fn onclick_closure< T >( filter_renderer : &Rc< RefCell< Renderer > > )
-> Box < dyn Fn( &JsValue ) >
where
T : Filter + DeserializeOwned
{
  let filter_renderer = filter_renderer.clone();
  Box::new
  (
    move | obj : &JsValue |
    {
      let filter : T = serde_wasm_bindgen::from_value( obj.clone() ).unwrap();
      filter_renderer.borrow_mut().apply_filter( &filter );
    }
  )
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
    filter_renderer.borrow_mut().apply_filter( &filter );
    show_apply_cancel_buttons();
  }))
}
