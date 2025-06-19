use minwebgl as gl;
use gl::{
  JsCast,
  web_sys::
  {
    HtmlElement,
    wasm_bindgen::prelude::Closure,
    HtmlInputElement, 
    Event,
    console
  }
};
use hex_color::HexColor;
use color::
{
  ColorSpace,
  Srgb, LinearSrgb,
  A98Rgb, Aces2065_1, AcesCg, DisplayP3, Hsl, Hwb, Lab, Lch,
  Oklab, Oklch, ProphotoRgb, Rec2020, XyzD50, XyzD65,
};

struct RectInfo 
{
  name : String,
  color_element : HtmlElement,
  color_coord_label : HtmlElement,
}

impl RectInfo
{
  fn new(
    document: &web_sys::Document, 
    name : &str,
  ) -> Result< Self, gl::WebglError >
  {
    Ok(
      Self
      {
        name : name.to_string(),
        color_element : get_element( &document, &format!( "{name}-rectangle" ) )?,
        color_coord_label : get_element( &document, &format!( "{name}-value" ) )?
      }
    )
  }
}

fn get_input_element( document: &web_sys::Document, id: &str ) -> Result< HtmlInputElement, gl::WebglError > 
{
  document.get_element_by_id( id )
  .ok_or_else( 
    || gl::WebglError::MissingDataError( "Element not found ( get_input_element )" ) 
  )?
  .dyn_into::< HtmlInputElement >()
  .or_else( 
    | _ | Err( gl::WebglError::NotSupportedForType( "Element can't be converted to HtmlInputElement" ) ) 
  )
}

fn get_element( document: &web_sys::Document, id: &str ) -> Result< HtmlElement, gl::WebglError > 
{
  document.get_element_by_id( id )
  .ok_or_else( 
    || gl::WebglError::MissingDataError( "Element not found ( get_element )" )
  )?
  .dyn_into::< HtmlElement >()
  .or_else( 
    | _ | Err( gl::WebglError::NotSupportedForType( "Element can't be converted to HtmlElement" ) ) 
  )
}

async fn run() -> Result< (), gl::WebglError >
{
  let window = gl::web_sys::window().expect( "no global `window` exists" );
  let document = window.document().expect( "should have a document on window" );

  let srgb_color_picker = get_input_element( &document, "srgb-color-picker" )?;

  let mut rectangle_elements = vec![];

  for name in 
  [
    "a98rgb",
    "aces2065-1",
    "aces-cg",
    "display-p3",
    "hsl",
    "hwb",
    "lab",
    "lch",
    "linear-srgb",
    "oklab",
    "oklch",
    "prophoto-rgb",
    "rec2020",
    "xyz-d50",
    "xyz-d65"
  ]
  {
    rectangle_elements.push( RectInfo::new( &document, name )? );
  }

  let srgb_element = get_element( &document, "srgb-value" )?;

  let set_color = | rect_elem : &HtmlElement, css_color : &str |
  {
    rect_elem.style()
    .set_property( "background-color", css_color )
    .expect( "Failed to set style" );
  };

  let ftou = | c : f32 | ( u8::MAX as f32 * c.clamp( 0.0, 1.0 ) ).round() as u8;

  let update_rectangles = Closure::< dyn FnMut( Event ) >::new( 
    move | event : Event | 
    {
      let target = event.target().expect( "Event should have a target" );
      let input_element = target.dyn_into::< HtmlInputElement >().expect( "Target should be an input element" );
      let hex_color = input_element.value();

      console::log_1( &format!( "sRGB picker changed to: {}", hex_color ).into() );

      let src_hex_color = match HexColor::parse( &hex_color ) 
      {
        Ok( c ) => c,
        Err( e ) => {
          console::error_1( &format!( "Failed to parse hex color: {:?}", e ).into() );
          return;
        }
      };

      let base_srgb_components = 
      [
        src_hex_color.r as f32 / 255.0,
        src_hex_color.g as f32 / 255.0,
        src_hex_color.b as f32 / 255.0,
      ];

      let color_css = format!( 
        "rgb( {} {} {} )", 
        src_hex_color.r, 
        src_hex_color.g, 
        src_hex_color.b 
      );
      srgb_element.set_text_content( Some( &color_css ) );

      for rect_elem in rectangle_elements.iter() 
      {
        match rect_elem.name.as_str() 
        {
          "a98rgb" => 
          {
            let [ r, g, b ] = Srgb::convert::< A98Rgb >( base_srgb_components );
            let color_css = format!( "rgb( {} {} {} )", ftou( r ), ftou( g ), ftou( b ) );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "aces2065-1" => 
          {
            let [ r, g, b ] = Srgb::convert::< Aces2065_1 >( base_srgb_components );
            let color_css = format!( "rgb( {} {} {} )", ftou( r ), ftou( g ), ftou( b ) );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "aces-cg" => 
          {
            let [ r, g, b ] = Srgb::convert::< AcesCg >( base_srgb_components );
            let color_css = format!( "rgb( {} {} {} )", ftou( r ), ftou( g ), ftou( b ) );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "display-p3" => 
          {
            let [ r, g, b ] = Srgb::convert::< DisplayP3 >( base_srgb_components );
            let color_css = format!( "rgb( {} {} {} )", ftou( r ), ftou( g ), ftou( b ) );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "hsl" => 
          {
            let [ h, s, l ] = Srgb::convert::< Hsl >( base_srgb_components );
            let color_css = format!( "hsl( {:.2} {:.2} {:.2} )", h, s, l );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "hwb" => 
          {
            let [ h, w, b ] = Srgb::convert::< Hwb >( base_srgb_components );
            let color_css = format!( "hwb( {:.2} {:.2} {:.2} )", h, w, b );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "lab" => 
          {
            let [ l, a, b ] = Srgb::convert::< Lab >( base_srgb_components );
            let color_css = format!( "lab( {:.2} {:.2} {:.2} )", l, a, b );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "lch" => 
          {
            let [ l, c, h ] = Srgb::convert::< Lch >( base_srgb_components );
            let color_css = format!( "lch( {:.2} {:.2} {:.2} )", l, c, h );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "linear-srgb" => 
          {
            let [ r, g, b ] = Srgb::convert::< LinearSrgb >( base_srgb_components );
            let color_css = format!( "rgb( {} {} {} )", ftou( r ), ftou( g ), ftou( b ) );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "oklab" => 
          {
            let [ l, a, b ] = Srgb::convert::< Oklab >( base_srgb_components );
            let color_css = format!( "oklab( {:.2} {:.2} {:.2} )", l, a, b );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "oklch" => 
          {
            let [ l, c, h ] = Srgb::convert::< Oklch >( base_srgb_components );
            let color_css = format!( "oklch( {:.2} {:.2} {:.2} )", l, c, h );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "prophoto-rgb" => 
          {
            let [ r, g, b ] = Srgb::convert::< ProphotoRgb >( base_srgb_components );
            let color_css = format!( "rgb( {} {} {} )", ftou( r ), ftou( g ), ftou( b ) );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "rec2020" => 
          {
            let [ r, g, b ] = Srgb::convert::< Rec2020 >( base_srgb_components );
            let color_css = format!( "rgb( {} {} {} )", ftou( r ), ftou( g ), ftou( b ) );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "xyz-d50" => 
          {
            let [ x, y, z ] = Srgb::convert::< XyzD50 >( base_srgb_components );
            let color_css = format!( "color(xyz-d50 {:.2} {:.2} {:.2})", x, y, z  );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          "xyz-d65" => 
          {
            let [ x, y, z ] = Srgb::convert::< XyzD65 >( base_srgb_components );
            let color_css = format!( "color(xyz-d65 {:.2} {:.2} {:.2})", x, y, z );
            set_color( &rect_elem.color_element, &color_css );
            rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
          },
          _ => {
            console::warn_1( &format!( "Unknown rectangle ID: {}", rect_elem.name ).into() );
            continue;
          }
        };
      }
    }
  );

  srgb_color_picker.add_event_listener_with_callback( "input", update_rectangles.as_ref().unchecked_ref() ).unwrap();
  update_rectangles.forget();

  let initial_event = Event::new( "input" ).expect( "Failed to create initial event" );
  srgb_color_picker.dispatch_event( &initial_event ).unwrap();

  Ok( () )
}

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async move { run().await.unwrap() } );
}