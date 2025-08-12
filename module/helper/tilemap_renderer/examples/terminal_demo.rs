//! Terminal renderer demonstration.
//!
//! This example shows how to use the terminal backend adapter to render 
//! scenes as ASCII art with Unicode line drawing and ANSI colors.

#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::implicit_return ) ]

use tilemap_renderer::
{
  adapters::TerminalRenderer,
  ports::{ Renderer, RenderContext },
  commands::{ Point2D, StrokeStyle, FontStyle, TextAnchor, LineCap, LineJoin, RenderCommand, LineCommand, CurveCommand, TextCommand },
  scene::Scene,
};

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Create a terminal renderer with custom dimensions
  let mut renderer = TerminalRenderer::with_dimensions( 60, 25 );
  
  // Enable Unicode characters and ANSI colors
  renderer.set_unicode_enabled( true );
  renderer.set_color_enabled( true );
  
  // Initialize the renderer
  let context = RenderContext::default();
  renderer.initialize( &context ).map_err( |e| format!( "Initialization failed: {:?}", e ) )?;
  
  // Begin a new frame
  renderer.begin_frame( &context ).map_err( |e| format!( "Begin frame failed: {:?}", e ) )?;
  
  // Create a demonstration scene
  let mut scene = Scene::new();
  
  // Add title
  let mut title_text = [ 0u8; 64 ];
  let title = b"Terminal Backend Demonstration";
  title_text[ ..title.len() ].copy_from_slice( title );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: title_text,
    text_len: title.len() as u8,
    position: Point2D { x: 15.0, y: 2.0 },
    font_style: FontStyle
    {
      color: [ 1.0, 1.0, 0.0, 1.0 ], // Yellow
      size: 16.0,
      family_id: 0,
      weight: 700,
      italic: false,
    },
    anchor: TextAnchor::TopLeft,
  } ) );
  
  // Add decorative border
  let border_color = StrokeStyle
  {
    color: [ 0.0, 1.0, 0.0, 1.0 ], // Green
    width: 1.0,
    cap_style: LineCap::Butt,
    join_style: LineJoin::Miter,
  };
  
  // Top and bottom borders
  scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 5.0, y: 4.0 }, end: Point2D { x: 55.0, y: 4.0 }, style: border_color } ) );
  scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 5.0, y: 20.0 }, end: Point2D { x: 55.0, y: 20.0 }, style: border_color } ) );
  
  // Left and right borders
  scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 5.0, y: 4.0 }, end: Point2D { x: 5.0, y: 20.0 }, style: border_color } ) );
  scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 55.0, y: 4.0 }, end: Point2D { x: 55.0, y: 20.0 }, style: border_color } ) );
  
  // Add some diagonal lines for visual interest
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 10.0, y: 8.0 },
    end: Point2D { x: 25.0, y: 16.0 },
    style: StrokeStyle { color: [ 1.0, 0.0, 0.0, 1.0 ], width: 1.0, cap_style: LineCap::Butt, join_style: LineJoin::Miter }
  } ) );
  
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 35.0, y: 8.0 },
    end: Point2D { x: 50.0, y: 16.0 },
    style: StrokeStyle { color: [ 0.0, 0.0, 1.0, 1.0 ], width: 1.0, cap_style: LineCap::Butt, join_style: LineJoin::Miter }
  } ) );
  
  // Add a curved line
  scene.add( RenderCommand::Curve( CurveCommand
  {
    start: Point2D { x: 10.0, y: 18.0 },
    control1: Point2D { x: 20.0, y: 10.0 },
    control2: Point2D { x: 40.0, y: 10.0 },
    end: Point2D { x: 50.0, y: 18.0 },
    style: StrokeStyle { color: [ 1.0, 0.0, 1.0, 1.0 ], width: 1.0, cap_style: LineCap::Butt, join_style: LineJoin::Miter }
  } ) );
  
  // Add feature descriptions
  let features = [
    ( "✓ Unicode line drawing characters", Point2D { x: 8.0, y: 6.0 }, [ 0.0, 1.0, 1.0, 1.0 ] ),
    ( "✓ ANSI color support", Point2D { x: 8.0, y: 8.0 }, [ 1.0, 0.5, 0.0, 1.0 ] ),
    ( "✓ Text rendering with anchoring", Point2D { x: 8.0, y: 10.0 }, [ 0.5, 0.0, 1.0, 1.0 ] ),
    ( "✓ Line and curve approximation", Point2D { x: 8.0, y: 12.0 }, [ 1.0, 0.0, 0.5, 1.0 ] ),
    ( "✓ Configurable dimensions", Point2D { x: 8.0, y: 14.0 }, [ 0.0, 0.8, 0.2, 1.0 ] ),
  ];
  
  for ( text, pos, color ) in &features
  {
    let mut text_array = [ 0u8; 64 ];
    let text_bytes = text.as_bytes();
    let copy_len = text_bytes.len().min( 64 );
    text_array[ ..copy_len ].copy_from_slice( &text_bytes[ ..copy_len ] );
    
    scene.add( RenderCommand::Text( TextCommand
    {
      text: text_array,
      text_len: copy_len as u8,
      position: *pos,
      font_style: FontStyle
      {
        color: *color,
        size: 12.0,
        family_id: 0,
        weight: 400,
        italic: false,
      },
      anchor: TextAnchor::TopLeft,
    } ) );
  }
  
  // Add footer note
  let mut footer_text = [ 0u8; 64 ];
  let footer = b"Rendered with Terminal Backend";
  footer_text[ ..footer.len() ].copy_from_slice( footer );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: footer_text,
    text_len: footer.len() as u8,
    position: Point2D { x: 17.0, y: 22.0 },
    font_style: FontStyle
    {
      color: [ 0.7, 0.7, 0.7, 1.0 ], // Gray
      size: 10.0,
      family_id: 0,
      weight: 300,
      italic: true,
    },
    anchor: TextAnchor::TopLeft,
  } ) );
  
  // Render the scene
  renderer.render_scene( &scene ).map_err( |e| format!( "Scene rendering failed: {:?}", e ) )?;
  
  // End the frame
  renderer.end_frame().map_err( |e| format!( "End frame failed: {:?}", e ) )?;
  
  // Get and display output
  let output = renderer.output().map_err( |e| format!( "Output failed: {:?}", e ) )?;
  println!( "{}", output );
  println!( "\n" );
  
  // Also save to file for inspection
  std::fs::write( "-terminal_demo_output.txt", &output ).map_err( |e| format!( "Export failed: {:?}", e ) )?;
  println!( "Terminal output also saved to -terminal_demo_output.txt" );
  
  // Demonstrate ASCII-only mode
  println!( "\n--- ASCII-Only Mode (no Unicode) ---\n" );
  
  let mut ascii_renderer = TerminalRenderer::with_dimensions( 60, 15 );
  ascii_renderer.set_unicode_enabled( false ); // Disable Unicode
  ascii_renderer.set_color_enabled( false );   // Disable colors
  
  ascii_renderer.initialize( &context ).map_err( |e| format!( "ASCII initialization failed: {:?}", e ) )?;
  
  ascii_renderer.begin_frame( &context ).map_err( |e| format!( "ASCII begin frame failed: {:?}", e ) )?;
  
  // Create a simple ASCII-compatible scene
  let mut ascii_scene = Scene::new();
  
  let mut ascii_title = [ 0u8; 64 ];
  let ascii_title_text = b"ASCII Mode Demo";
  ascii_title[ ..ascii_title_text.len() ].copy_from_slice( ascii_title_text );
  
  ascii_scene.add( RenderCommand::Text( TextCommand
  {
    text: ascii_title,
    text_len: ascii_title_text.len() as u8,
    position: Point2D { x: 23.0, y: 2.0 },
    font_style: FontStyle
    {
      color: [ 0.0, 0.0, 0.0, 1.0 ], // Black (no color)
      size: 14.0,
      family_id: 0,
      weight: 400,
      italic: false,
    },
    anchor: TextAnchor::TopLeft,
  } ) );
  
  // Simple box using ASCII characters
  let ascii_style = StrokeStyle
  {
    color: [ 0.0, 0.0, 0.0, 1.0 ],
    width: 1.0,
    cap_style: LineCap::Butt,
    join_style: LineJoin::Miter,
  };
  
  ascii_scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 10.0, y: 5.0 }, end: Point2D { x: 50.0, y: 5.0 }, style: ascii_style } ) );
  ascii_scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 10.0, y: 10.0 }, end: Point2D { x: 50.0, y: 10.0 }, style: ascii_style } ) );
  ascii_scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 10.0, y: 5.0 }, end: Point2D { x: 10.0, y: 10.0 }, style: ascii_style } ) );
  ascii_scene.add( RenderCommand::Line( LineCommand { start: Point2D { x: 50.0, y: 5.0 }, end: Point2D { x: 50.0, y: 10.0 }, style: ascii_style } ) );
  
  let mut ascii_content = [ 0u8; 64 ];
  let ascii_content_text = b"Compatible with any terminal!";
  ascii_content[ ..ascii_content_text.len() ].copy_from_slice( ascii_content_text );
  
  ascii_scene.add( RenderCommand::Text( TextCommand
  {
    text: ascii_content,
    text_len: ascii_content_text.len() as u8,
    position: Point2D { x: 15.0, y: 7.0 },
    font_style: FontStyle
    {
      color: [ 0.0, 0.0, 0.0, 1.0 ],
      size: 12.0,
      family_id: 0,
      weight: 400,
      italic: false,
    },
    anchor: TextAnchor::TopLeft,
  } ) );
  
  ascii_renderer.render_scene( &ascii_scene ).map_err( |e| format!( "ASCII scene rendering failed: {:?}", e ) )?;
  ascii_renderer.end_frame().map_err( |e| format!( "ASCII end frame failed: {:?}", e ) )?;
  let ascii_output = ascii_renderer.output().map_err( |e| format!( "ASCII output failed: {:?}", e ) )?;
  println!( "{}", ascii_output );
  
  println!( "\nDemonstration complete!" );
  
  Ok( () )
}