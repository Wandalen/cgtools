#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ]

use std::collections::HashMap;
use mingl::F32x4;
use minwebgl::
{
  self as gl,
  JsCast,
  web_sys::
  {
    HtmlElement,
    HtmlSelectElement,
    HtmlSpanElement,
    HtmlInputElement,
    window,
    wasm_bindgen::closure::Closure
  }
};
use rand::Rng;
use renderer::webgl::
{
  loaders::gltf::GLTF,
  geometry::AttributeInfo,
  Camera,
  Renderer,
  post_processing::
  {
    self,
    outline::narrow_outline::NarrowOutlinePass,
    outline::normal_depth_outline::NormalDepthOutlinePass,
    outline::wide_outline::WideOutlinePass,
    GBuffer,
    GBufferAttachment,
    Pass,
    SwapFramebuffer
  }
};
use std::rc::Rc;
use std::cell::RefCell;

mod camera_controls;

fn generate_object_colors( object_count : u32 ) -> Vec< F32x4 >
{
  let mut rng = rand::rng();

  let range = 0.2..1.0;
  let object_colors = ( 0..object_count )
  .map
  (
    | _ |
    {
      F32x4::from_array
      (
        [
          rng.random_range( range.clone() ),
          rng.random_range( range.clone() ),
          rng.random_range( range.clone() ),
          1.0
        ]
      )
    }
  )
  .collect::< Vec< _ > >();

  object_colors
}

fn get_attributes( gltf : &GLTF ) -> Result< HashMap< Box< str >, AttributeInfo >, gl::WebglError >
{
  for mesh in &gltf.meshes
  {
    let mesh_ref = mesh.as_ref().borrow();
    for primitive in &mesh_ref.primitives
    {
      let primitive_ref = primitive.as_ref().borrow();
      return Ok( primitive_ref.geometry.as_ref().borrow().get_attributes().clone() );
    }
  }

  Err( gl::WebglError::MissingDataError( "Primitive" ) )
}

fn get_html_element_by_id( id : &str ) -> HtmlElement
{
  let document = window()
  .unwrap()
  .document()
  .unwrap();
  document.get_element_by_id(id)
  .unwrap()
  .dyn_into::< HtmlElement >()
  .unwrap()
}

fn get_html_span_element_by_id( id : &str ) -> HtmlSpanElement
{
  get_html_element_by_id( id )
  .dyn_into::< HtmlSpanElement >()
  .unwrap()
}

fn get_html_input_element_by_id( id : &str ) -> HtmlInputElement
{
  get_html_element_by_id( id )
  .dyn_into::< HtmlInputElement >()
  .unwrap()
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to enable EXT_color_buffer_float extension" );

  let _ = gl.get_extension( "EXT_shader_image_load_store" )
  .expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf_path = "old_rusty_car.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes.clone();
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  gl::info!( "Scene boudnig box: {:?}", scene_bounding_box );
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let dist = scene_bounding_box.max.mag();
  let exponent =
  {
    let bits = diagonal.to_bits();
    let exponent_field = ( ( bits >> 23 ) & 0xFF ) as i32;
    exponent_field - 127
  };
  gl::info!( "Exponent: {:?}", exponent );

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  eye *= dist;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 );
  let far = near * 100.0f32.powi( exponent.abs() );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera_controls::setup_controls( &canvas, &camera.get_controls() );

  let renderer = Rc::new
  (
    RefCell::new
    (
      Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?
    )
  );
  let renderer1 = renderer.clone();

  let attributes = get_attributes( &gltf )?;

  gl::info!( "{:?}", attributes.keys() );

  let get_buffer = | name | attributes.get( name ).unwrap().buffer.clone();

  let attachments = HashMap::from(
    [
      ( GBufferAttachment::Position, vec![ get_buffer( "positions" ) ] ),
      ( GBufferAttachment::Albedo, vec![] ),
      ( GBufferAttachment::Uv1, vec![] ),
      ( GBufferAttachment::Normal, vec![ get_buffer( "normals" ) ] ),
      ( GBufferAttachment::PbrInfo, vec![ get_buffer( "texture_coordinates_2" ) ] ),
      ( GBufferAttachment::ObjectColor, vec![] )
    ]
  );

  let gbuffer = Rc::new
  (
    RefCell::new
    (
      GBuffer::new
      (
        &gl,
        canvas.width(),
        canvas.height(),
        attachments
      )?
    )
  );
  let gbuffer_rc = gbuffer.clone();

  let swap_buffer = Rc::new
  (
    RefCell::new
    (
      SwapFramebuffer::new( &gl, canvas.width(), canvas.height() )
    )
  );

  let sw1 = swap_buffer.clone();
  let sw2 = swap_buffer.clone();

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let outline_thickness = Rc::new( RefCell::new( 5.0f32 ) );
  let outline_thickness_1 = outline_thickness.clone();
  let outline_thickness_2 = outline_thickness.clone();

  let narrow_outline = Rc::new
  (
    RefCell::new
    (
      NarrowOutlinePass::new
      (
        &gl,
        gbuffer.borrow().get_texture( GBufferAttachment::Position ),
        gbuffer.borrow().get_texture( GBufferAttachment::ObjectColor ),
        *outline_thickness.borrow(),
        canvas.width(),
        canvas.height()
      )?
    )
  );

  let normal_depth_outline = Rc::new
  (
    RefCell::new
    (
      NormalDepthOutlinePass::new
      (
        &gl,
        gbuffer.borrow().get_texture( GBufferAttachment::Position ),
        gbuffer.borrow().get_texture( GBufferAttachment::Normal ),
        gbuffer.borrow().get_texture( GBufferAttachment::ObjectColor ),
        *outline_thickness.borrow(),
        canvas.width(),
        canvas.height()
      )?
    )
  );

  let wide_outline = Rc::new
  (
    RefCell::new
    (
      WideOutlinePass::new
      (
        &gl,
        gbuffer.borrow()
        .get_texture( GBufferAttachment::ObjectColor ).unwrap(),
        *outline_thickness.borrow(),
        canvas.width(),
        canvas.height()
      )?
    )
  );

  let object_colors = generate_object_colors( gltf.meshes.len() as u32 );

  let gl = Rc::new( RefCell::new( gl ) );
  let gl1 = gl.clone();
  let gl2 = gl.clone();

  let select_texture = move | select_value : &str |
  {
    let current_outline_thickness = *outline_thickness_1.borrow();

    match select_value
    {
      "position" => gbuffer_rc.borrow().get_texture( GBufferAttachment::Position ),
      "normal" => gbuffer_rc.borrow().get_texture( GBufferAttachment::Normal ),
      "albedo" => gbuffer_rc.borrow().get_texture( GBufferAttachment::Albedo ),
      "object_color" => gbuffer_rc.borrow().get_texture( GBufferAttachment::ObjectColor ),
      "narrow_outline" =>
      {
        let narrow_outline_1 = narrow_outline.clone();
        narrow_outline_1.borrow_mut()
        .set_outline_thickness( current_outline_thickness );
        narrow_outline.clone().borrow_mut()
        .render( &gl1.borrow(), sw1.borrow().get_input(), sw1.borrow().get_output() )
        .expect( "Failed to render outline pass" )
      },
      "normal_depth_outline" =>
      {
        let normal_depth_outline_1 = normal_depth_outline.clone();
        normal_depth_outline_1.borrow_mut()
        .set_outline_thickness( current_outline_thickness );
        normal_depth_outline.clone().borrow_mut()
        .render( &gl1.borrow(), sw1.borrow().get_input(), sw1.borrow().get_output() )
        .expect( "Failed to render outline pass" )
      },
      _ if select_value.starts_with( "wide_outline" ) =>
      {
        let wide_outline_1 = wide_outline.clone();
        if let Some( passes ) = select_value.strip_prefix( "wide_outline" )
        {
          if let Ok( passes ) = u32::from_str_radix( passes, 10 )
          {
            wide_outline_1.borrow_mut().set_num_passes( passes );
          }
        }

        wide_outline_1.borrow_mut()
        .set_outline_thickness( current_outline_thickness );
        let texture = wide_outline_1.borrow_mut()
        .render( &gl1.borrow(), sw1.borrow().get_input(), sw1.borrow().get_output() )
        .expect( "Failed to render outline pass" );

        texture
      },
      _ => None
    }
  };

  let select_value = Rc::new( RefCell::new( String::new() ) );
  let select_value_clone = select_value.clone();

  let select_change_closure = Closure::wrap
  (
    Box::new
    (
    move | event: web_sys::Event |
    {
      let select_element_target = event.target()
      .and_then( | t | t.dyn_into::< HtmlSelectElement >().ok() );
      if let Some( select_elem ) = select_element_target
      {
        *select_value_clone.borrow_mut() = select_elem.value().to_string();
      }
      else
      {
        gl::warn!( "Failed to cast event target to HtmlSelectElement" );
      }
    }
    )
    as Box< dyn FnMut( _ ) >
  );

  let select_element = get_html_element_by_id( "displayOption" );
  let _ = select_element.add_event_listener_with_callback( "change", select_change_closure.as_ref().unchecked_ref() );
  select_change_closure.forget();

  // --- Slider Event Listener ---
  let outline_thickness_slider_element = get_html_input_element_by_id( "outlineThicknessSlider" );
  let outline_thickness_display_span = get_html_span_element_by_id( "outlineThicknessValue" );

  // Set initial value of the display span
  let _ = outline_thickness_display_span.set_text_content( Some( &outline_thickness.borrow().to_string() ) );

  let slider_change_closure =
  Closure::wrap
  (
    Box::new(
      move | event : web_sys::Event |
      {
        let input_element_target = event.target()
        .and_then( | t | t.dyn_into::< HtmlInputElement >().ok() );
        if let Some(input_elem) = input_element_target
        {
          if let Ok( value ) = input_elem.value().parse::<f32>()
          {
            *outline_thickness_2.borrow_mut() = value;
            let _ = outline_thickness_display_span.set_text_content( Some( &value.to_string() ) );
          }
          else
          {
            gl::warn!( "Failed to parse slider value to f32" );
          }
        }
        else
        {
          gl::warn!( "Failed to cast event target to HtmlInputElement" );
        }
      }
    ) as Box< dyn FnMut( _ ) >
  );

  let _ = outline_thickness_slider_element.add_event_listener_with_callback( "input", slider_change_closure.as_ref().unchecked_ref() );
  slider_change_closure.forget();

  let fps_value = get_html_span_element_by_id( "fpsValue" );
  let mut last_time = 0.0;
  let mut fps = 0;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = ( t / 1000.0 ) as f32;

      // Update fps text when a whole second elapsed
      if time as u32 > last_time as u32
      {
        fps_value.set_text_content( Some( &format!( "{}", fps ) ) );
        fps = 0;
      }
      last_time = time;
      fps += 1;

      gbuffer.clone()
      .borrow_mut()
      .render( &gl2.borrow(), &mut scenes[ 0 ].borrow_mut(), Some( &object_colors ), &camera )
      .expect( "Failed to render gbuffer" );

      renderer1.borrow_mut().render( &gl2.borrow(), &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      sw2.borrow_mut().reset();
      sw2.borrow_mut().bind( &gl2.borrow() );
      sw2.borrow_mut().set_input( renderer1.borrow().get_main_texture() );

      if let Some( t ) = select_texture( &select_value.borrow() )
      {
        sw2.borrow_mut().bind( &gl2.borrow() );
        sw2.borrow_mut().set_output( Some( t ) );
        sw2.borrow_mut().swap();
      }

      let t = tonemapping.render( &gl2.borrow(), sw2.borrow().get_input(), sw2.borrow().get_output() )
      .expect( "Failed to render tonemapping pass" );

      sw2.borrow_mut().set_output( t );
      sw2.borrow_mut().swap();

      let _t = to_srgb.render( &gl2.borrow(), sw2.borrow().get_input(), sw2.borrow().get_output() )
      .expect( "Failed to render to srgb pass" );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
