#![ doc( html_root_url = "https://docs.rs/color_space_convertion/latest/color_space_convertion/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Converts colors from RGBA into another color spaces" ) ]

use minwebgl as gl;
use gl::{
  JsCast,
  web_sys::
  {
    HtmlElement,
    wasm_bindgen::prelude::Closure,
    HtmlInputElement,
    Event
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
        color_element : element_get( document, &format!( "{name}-rectangle" ) )?,
        color_coord_label : element_get( document, &format!( "{name}-value" ) )?
      }
    )
  }
}

fn input_element_get( document: &web_sys::Document, id: &str ) -> Result< HtmlInputElement, gl::WebglError >
{
  document.get_element_by_id( id )
  .ok_or
  (
    gl::WebglError::MissingDataError( "Element not found ( get_input_element )" )
  )?
  .dyn_into::< HtmlInputElement >()
  .map_err
  (
    | _ | gl::WebglError::NotSupportedForType( "Element can't be converted to HtmlInputElement" )
  )
}

fn element_get( document: &web_sys::Document, id: &str ) -> Result< HtmlElement, gl::WebglError >
{
  document.get_element_by_id( id )
  .ok_or
  (
    gl::WebglError::MissingDataError( "Element not found ( get_element )" )
  )?
  .dyn_into::< HtmlElement >()
  .map_err
  (
    | _ | gl::WebglError::NotSupportedForType( "Element can't be converted to HtmlElement" )
  )
}

// 185 lines : one linear event-handler setup ending in a flat match over 14 color-space
// conversion arms; splitting the match would only relocate the repetition, not reduce it.
#[ allow( clippy::too_many_lines, reason = "one linear event-handler setup ending in a flat match over 14 color-space conversion arms; splitting the match would only relocate the repetition, not reduce it" ) ]
fn app_run() -> Result< (), gl::WebglError >
{
  let window = gl::web_sys::window().expect( "no global `window` exists" );
  let document = window.document().expect( "should have a document on window" );

  let srgb_color_picker = input_element_get( &document, "srgb-color-picker" )?;

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

  let srgb_element = element_get( &document, "srgb-value" )?;

  let set_color = | rect_elem : &HtmlElement, css_color : &str |
  {
    rect_elem.style()
    .set_property( "background-color", css_color )
    .expect( "Failed to set style" );
  };

  // `component` is clamped to [0.0, 1.0] before scaling by `u8::MAX`, so the rounded
  // result is always exactly representable in `u8` — no truncation or sign loss is possible.
  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "component is clamped to [0.0, 1.0] before scaling by u8::MAX, so the rounded result is always exactly representable in u8 — no truncation or sign loss is possible" ) ]
  let ftou = | component : f32 | ( f32::from(u8::MAX) * component.clamp( 0.0, 1.0 ) ).round() as u8;

  let update_rectangles = Closure::< dyn FnMut( Event ) >::new
  (
    move | event : Event |
    {
      let target = event.target().expect( "Event should have a target" );
      let input_element = target.dyn_into::< HtmlInputElement >()
      .expect( "Target should be an input element" );
      let hex_color = input_element.value();

      gl::info!( "sRGB picker changed to: {hex_color}" );

      let src_hex_color = match HexColor::parse( &hex_color )
      {
        Ok( color ) => color,
        Err( error ) =>
        {
          panic!( "Failed to parse hex color: {error:?}" );
        }
      };

      let base_srgb_components =
      [
        f32::from(src_hex_color.r) / 255.0,
        f32::from(src_hex_color.g) / 255.0,
        f32::from(src_hex_color.b) / 255.0,
      ];

      let color_css = format!
      (
        "rgb( {red} {green} {blue} )",
        red = src_hex_color.r,
        green = src_hex_color.g,
        blue = src_hex_color.b
      );
      srgb_element.set_text_content( Some( &color_css ) );

      for rect_elem in &rectangle_elements
      {
        let color_css = match rect_elem.name.as_str()
        {
          "a98rgb" =>
          {
            let [ red, green, blue ] = Srgb::convert::< A98Rgb >( base_srgb_components );
            format!( "rgb( {} {} {} )", ftou( red ), ftou( green ), ftou( blue ) )
          },
          "aces2065-1" =>
          {
            let [ red, green, blue ] = Srgb::convert::< Aces2065_1 >( base_srgb_components );
            format!( "rgb( {} {} {} )", ftou( red ), ftou( green ), ftou( blue ) )
          },
          "aces-cg" =>
          {
            let [ red, green, blue ] = Srgb::convert::< AcesCg >( base_srgb_components );
            format!( "rgb( {} {} {} )", ftou( red ), ftou( green ), ftou( blue ) )
          },
          "display-p3" =>
          {
            let [ red, green, blue ] = Srgb::convert::< DisplayP3 >( base_srgb_components );
            format!( "rgb( {} {} {} )", ftou( red ), ftou( green ), ftou( blue ) )
          },
          "hsl" =>
          {
            // Fix(BUG-XXX): saturation/lightness are numeric [0,100] per the `color` crate's
            // own Hsl docs, but CSS's `hsl()` grammar types those two components as
            // `<percentage>`, not `<number>` (unlike `lab()`/`lch()`, which accept either) —
            // a bare number is invalid syntax and the browser silently drops the whole
            // declaration, leaving the swatch unstyled.
            // Root cause: the format string omitted the `%` suffix CSS requires for S/L.
            // Pitfall: `lab`/`lch`/`oklab`/`oklch` correctly stay bare (`<number>` is valid
            // there) — don't blanket-append `%` to every color-space arm above.
            let [ hue, saturation, lightness ] = Srgb::convert::< Hsl >( base_srgb_components );
            format!( "hsl( {hue:.2} {saturation:.2}% {lightness:.2}% )" )
          },
          "hwb" =>
          {
            // Fix(BUG-XXX): same defect class as the "hsl" arm above — W/B are numeric
            // [0,100] but CSS's `hwb()` grammar requires `<percentage>` for both.
            let [ hue, whiteness, blackness ] = Srgb::convert::< Hwb >( base_srgb_components );
            format!( "hwb( {hue:.2} {whiteness:.2}% {blackness:.2}% )" )
          },
          "lab" =>
          {
            let [ lightness, a_axis, b_axis ] = Srgb::convert::< Lab >( base_srgb_components );
            format!( "lab( {lightness:.2} {a_axis:.2} {b_axis:.2} )" )
          },
          "lch" =>
          {
            let [ lightness, chroma, hue ] = Srgb::convert::< Lch >( base_srgb_components );
            format!( "lch( {lightness:.2} {chroma:.2} {hue:.2} )" )
          },
          "linear-srgb" =>
          {
            let [ red, green, blue ] = Srgb::convert::< LinearSrgb >( base_srgb_components );
            format!( "rgb( {} {} {} )", ftou( red ), ftou( green ), ftou( blue ) )
          },
          "oklab" =>
          {
            let [ lightness, a_axis, b_axis ] = Srgb::convert::< Oklab >( base_srgb_components );
            format!( "oklab( {lightness:.2} {a_axis:.2} {b_axis:.2} )" )
          },
          "oklch" =>
          {
            let [ lightness, chroma, hue ] = Srgb::convert::< Oklch >( base_srgb_components );
            format!( "oklch( {lightness:.2} {chroma:.2} {hue:.2} )" )
          },
          "prophoto-rgb" =>
          {
            let [ red, green, blue ] = Srgb::convert::< ProphotoRgb >( base_srgb_components );
            format!( "rgb( {} {} {} )", ftou( red ), ftou( green ), ftou( blue ) )
          },
          "rec2020" =>
          {
            let [ red, green, blue ] = Srgb::convert::< Rec2020 >( base_srgb_components );
            format!( "rgb( {} {} {} )", ftou( red ), ftou( green ), ftou( blue ) )
          },
          "xyz-d50" =>
          {
            let [ x_coord, y_coord, z_coord ] = Srgb::convert::< XyzD50 >( base_srgb_components );
            format!( "color(xyz-d50 {x_coord:.2} {y_coord:.2} {z_coord:.2})"  )
          },
          "xyz-d65" =>
          {
            let [ x_coord, y_coord, z_coord ] = Srgb::convert::< XyzD65 >( base_srgb_components );
            format!( "color(xyz-d65 {x_coord:.2} {y_coord:.2} {z_coord:.2})" )
          },
          _ =>
          {
            gl::warn!( "Unknown rectangle ID: {}", rect_elem.name );
            continue;
          }
        };

        set_color( &rect_elem.color_element, &color_css );
        rect_elem.color_coord_label.set_text_content( Some( &color_css ) );
      }
    }
  );

  srgb_color_picker.add_event_listener_with_callback
  (
    "input",
    update_rectangles.as_ref().unchecked_ref()
  )
  .unwrap();
  update_rectangles.forget();

  let initial_event = Event::new( "input" )
  .expect( "Failed to create initial event" );
  srgb_color_picker.dispatch_event( &initial_event ).unwrap();

  Ok( () )
}

fn main()
{
  gl::browser::setup( gl::browser::Config::default() );
  gl::spawn_local( async move { app_run().unwrap() } );
}
