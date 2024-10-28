use crate::*;
use utils::*;
use filters::*;
use web_sys::
{
  wasm_bindgen,
  HtmlButtonElement,
};
use wasm_bindgen::{ prelude::*, JsCast };
use std::{ cell::RefCell, rc::Rc } ;
use serde::de::DeserializeOwned;

pub fn setup_ui( filter_renderer : &Rc< RefCell< Renderer > > )
{
  let gui = lil_gui::new_gui();
  generate_filter_buttons( &gui );
  setup_filters_without_gui( filter_renderer );
  setup_filters_with_gui( &gui, filter_renderer );

  // Here's called click on original filter button
  // It is needed to trigger function that
  // setups proper control panel visibilty for current filter
  get_element_by_id_unchecked::< HtmlButtonElement >( "original" ).click();
}

fn generate_filter_buttons( gui : &JsValue )
{
  let buttons =
  [
    // id,                title of control to show,       text
    ( "original",         "",                             "Original"),
    ( "box-blur",         "Box Blur",                     "Box Blur" ),
    ( "gaussian-blur",    "Gaussian Blur",                "Gaussian Blur" ),
    ( "stack-blur",       "Stack Blur",                   "Stack Blur" ),
    ( "binarize",         "Binarize",                     "Binarize" ),
    ( "bcgimp",           "BrightnessContrast GIMP",      "Brightness Contrast GIMP" ),
    ( "bcph",             "BrightnessContrast Photoshop", "Brightness Contrast Photoshop" ),
    ( "channels",         "Channels",                     "Channels" ),
    ( "color-transform",  "Color Transform",              "Color Transform" ),
    ( "desaturate",       "",                             "Desaturate" ),
    ( "dither",           "Dithering",                    "Dither" ),
    ( "edge",             "",                             "Edge" ),
    ( "emboss",           "",                             "Emboss" ),
    ( "enrich",           "",                             "Enrich" ),
    ( "flip",             "Flip",                         "Flip" ),
    ( "gamma",            "Gamma",                        "Gamma" ),
    ( "grayscale",        "",                             "Grayscale" ),
    ( "hsl-adjust",       "HSL Adjust",                   "HSL Adjust" ),
    ( "invert",           "",                             "Invert" ),
    ( "mosaic",           "Mosaic",                       "Mosaic" ),
    ( "oil",              "Oil",                          "Oil" ),
    ( "posterize",        "Posterize",                    "Posterize" ),
    ( "rescale",          "Rescale",                      "Rescale" ),
    ( "resize-nn",        "Resize (Nearest)",             "Resize (Nearest Neighbor)" ),
    ( "resize-bilinear",  "Resize (Bilinear)",            "Resize (Bilinear)" ),
    ( "sepia",            "",                             "Sepia" ),
    ( "sharpen",          "Sharpen",                      "Sharpen" ),
    ( "solarize",         "",                             "Solarize" ),
    ( "transpose",        "",                             "Transpose" ),
    ( "twirl",            "Twirl",                        "Twirl" ),
  ];

  let document = web_sys::window().unwrap().document().unwrap();
  let buttons_container = document.get_element_by_id( "buttons" ).unwrap();
  for ( id, control_title, text ) in buttons
  {
    let btn = document.create_element( "button" ).unwrap().dyn_into::< HtmlButtonElement >().unwrap();
    btn.set_id( id );
    btn.set_text_content( Some( text ) );
    let gui = gui.clone();
    let onclik : Closure< dyn Fn() > = Closure::new( move || show_only( control_title, &gui ) );
    btn.add_event_listener_with_callback( "click", onclik.as_ref().unchecked_ref() ).unwrap();
    buttons_container.append_child( &btn ).unwrap();
    onclik.forget();
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

fn setup_filters_without_gui( filter_renderer : &Rc< RefCell< Renderer > > )
{
  // These are buttons ids and their respective filters
  // It's done for filters that don't contain any parameters
  let buttons =
  [
    ( "original",    make_closure_with_filter( filter_renderer, original::Original ) ),
    ( "desaturate",  make_closure_with_filter( filter_renderer, desaturate::Desaturate ) ),
    ( "edge",        make_closure_with_filter( filter_renderer, edge::Edge ) ),
    ( "emboss",      make_closure_with_filter( filter_renderer, emboss::Emboss ) ),
    ( "enrich",      make_closure_with_filter( filter_renderer, enrich::Enrich ) ),
    ( "grayscale",   make_closure_with_filter( filter_renderer, gray_scale::GrayScale ) ),
    ( "invert",      make_closure_with_filter( filter_renderer, invert::Invert ) ),
    ( "sepia",       make_closure_with_filter( filter_renderer, sepia::Sepia ) ),
    ( "solarize",    make_closure_with_filter( filter_renderer, solarize::Solarize ) ),
    ( "transpose",   make_closure_with_filter( filter_renderer, transpose::Transpose ) ),
  ];

  // Basically sets draw call on button click with respective filter
  for ( button_id, closure ) in buttons
  {
    let button = get_element_by_id_unchecked::< HtmlButtonElement >( button_id );
    button.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
}

fn setup_filters_with_gui( gui : &JsValue, filter_renderer : &Rc< RefCell< Renderer > > )
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
    let filter_button = get_element_by_id_unchecked::< HtmlButtonElement >( button_id );
    let onclick : Closure< dyn Fn() > = Closure::new
    (
      {
        let obj = obj.clone();
        move ||
        {
          ( onclick )( &obj );
        }
      }
    );
    filter_button.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
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
  let filter_button = get_element_by_id_unchecked::< HtmlButtonElement >( "channels" );
  let onclick : Closure< dyn Fn() > = Closure::new
  (
    {
      let obj = obj.clone();
      let filter_renderer = filter_renderer.clone();
      move ||
      {
        let onclick = onclick_closure::< channels::Channels >( &filter_renderer );
        ( onclick )( &obj );
      }
    }
  );
  filter_button.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
  closure.forget();

  let flip_gui = lil_gui::add_folder( &gui, "Flip" );
  let closure = onchange_closure::< flip::Flip >( filter_renderer );
  let obj = serde_wasm_bindgen::to_value( &flip::Flip { flip: flip::FlipDirection::FlipX } ).unwrap();
  let options = web_sys::js_sys::Array::of3( &"FlipX".into(), &"FlipY".into(), &"FlipXY".into() );
  lil_gui::add_dropdown( &flip_gui, &obj, "flip", &options.into() );
  lil_gui::on_finish_change( &flip_gui, closure.as_ref().unchecked_ref() );
  let filter_button = get_element_by_id_unchecked::< HtmlButtonElement >( "flip" );
  let onclick : Closure< dyn Fn() > = Closure::new
  (
    {
      let obj = obj.clone();
      let filter_renderer = filter_renderer.clone();
      move ||
      {
        let onclick = onclick_closure::< flip::Flip >( &filter_renderer );
        ( onclick )( &obj );
      }
    }
  );
  filter_button.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
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

fn make_closure_with_filter( filter_renderer : &Rc< RefCell< Renderer > >, filter : impl Filter + 'static )
-> Closure< dyn Fn() >
{
  let filter_renderer = filter_renderer.clone();
  Closure::new( Box::new( move || filter_renderer.borrow_mut().apply_filter( &filter ) ) )
}
