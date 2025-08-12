//! Interactive SVG-in-browser renderer demonstration example.
//!
//! This example shows how to use the SVG-browser backend adapter to generate
//! interactive HTML documents with embedded SVG and JavaScript functionality.

#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_precision_loss ) ]

use tilemap_renderer::
{
  adapters::SvgBrowserRenderer,
  ports::{ Renderer, RenderContext },
  scene::Scene,
  commands::{ Point2D, StrokeStyle, FontStyle, TextAnchor, LineCap, LineJoin, RenderCommand, LineCommand, CurveCommand, TextCommand },
};

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Create interactive SVG-browser renderer
  let mut renderer = SvgBrowserRenderer::with_dimensions( 800, 600 );
  
  // Enable all interactivity features
  renderer.set_mouse_picking_enabled( true );
  renderer.set_hover_effects_enabled( true );
  renderer.set_animation_enabled( true );
  
  // Set up rendering context
  let mut context = RenderContext::default();
  context.width = 800;
  context.height = 600;
  context.background_color = [ 0.98, 0.98, 0.98, 1.0 ]; // Very light gray background
  
  // Create an interactive scene
  let mut scene = Scene::with_id( "interactive_demo_scene" );
  
  // Add title text
  let mut title_text = [ 0u8; 64 ];
  let title = b"Interactive SVG Demo";
  title_text[ ..title.len() ].copy_from_slice( title );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: title_text,
    text_len: title.len() as u8,
    position: Point2D { x: 400.0, y: 40.0 },
    font_style: FontStyle
    {
      color: [ 0.1, 0.1, 0.5, 1.0 ], // Dark blue
      size: 28.0,
      weight: 700, // Bold
      italic: false,
      family_id: 0,
    },
    anchor: TextAnchor::Center,
  } ) );
  
  // Add interactive instruction text
  let mut instruction_text = [ 0u8; 64 ];
  let instruction = b"Click and hover over elements!";
  instruction_text[ ..instruction.len() ].copy_from_slice( instruction );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: instruction_text,
    text_len: instruction.len() as u8,
    position: Point2D { x: 400.0, y: 70.0 },
    font_style: FontStyle
    {
      color: [ 0.3, 0.3, 0.3, 1.0 ], // Gray
      size: 16.0,
      weight: 400,
      italic: true,
      family_id: 0,
    },
    anchor: TextAnchor::Center,
  } ) );
  
  // Add colorful interactive lines
  let line_colors = [
    [ 1.0, 0.2, 0.2, 1.0 ], // Red
    [ 0.2, 1.0, 0.2, 1.0 ], // Green
    [ 0.2, 0.2, 1.0, 1.0 ], // Blue
    [ 1.0, 0.8, 0.0, 1.0 ], // Orange
    [ 0.8, 0.0, 1.0, 1.0 ], // Purple
  ];
  
  for ( i, color ) in line_colors.iter().enumerate()
  {
    scene.add( RenderCommand::Line( LineCommand
    {
      start: Point2D { x: 100.0 + i as f32 * 120.0, y: 150.0 },
      end: Point2D { x: 180.0 + i as f32 * 120.0, y: 200.0 },
      style: StrokeStyle
      {
        color: *color,
        width: 4.0,
        cap_style: LineCap::Round,
        join_style: LineJoin::Round,
      },
    } ) );
  }
  
  // Add interactive curves
  let curve_colors = [
    [ 0.0, 0.8, 0.8, 0.8 ], // Cyan with transparency
    [ 0.8, 0.4, 0.0, 0.8 ], // Orange with transparency
    [ 0.6, 0.0, 0.6, 0.8 ], // Magenta with transparency
  ];
  
  for ( i, color ) in curve_colors.iter().enumerate()
  {
    scene.add( RenderCommand::Curve( CurveCommand
    {
      start: Point2D { x: 150.0 + i as f32 * 200.0, y: 350.0 },
      control1: Point2D { x: 250.0 + i as f32 * 200.0, y: 280.0 },
      control2: Point2D { x: 350.0 + i as f32 * 200.0, y: 420.0 },
      end: Point2D { x: 450.0 + i as f32 * 200.0, y: 350.0 },
      style: StrokeStyle
      {
        color: *color,
        width: 3.0,
        cap_style: LineCap::Round,
        join_style: LineJoin::Round,
      },
    } ) );
  }
  
  // Add interactive text elements
  let interactive_texts = [
    ( "Hover Me!", Point2D { x: 200.0, y: 300.0 }, [ 1.0, 0.0, 0.5, 1.0 ] ),
    ( "Click Me!", Point2D { x: 400.0, y: 300.0 }, [ 0.0, 0.7, 0.0, 1.0 ] ),
    ( "Try Me!", Point2D { x: 600.0, y: 300.0 }, [ 0.0, 0.3, 1.0, 1.0 ] ),
  ];
  
  for ( text_str, position, color ) in &interactive_texts
  {
    let mut text_array = [ 0u8; 64 ];
    let text_bytes = text_str.as_bytes();
    text_array[ ..text_bytes.len() ].copy_from_slice( text_bytes );
    
    scene.add( RenderCommand::Text( TextCommand
    {
      text: text_array,
      text_len: text_bytes.len() as u8,
      position: *position,
      font_style: FontStyle
      {
        color: *color,
        size: 20.0,
        weight: 600,
        italic: false,
        family_id: 0,
      },
      anchor: TextAnchor::Center,
    } ) );
  }
  
  // Add decorative border
  let border_style = StrokeStyle
  {
    color: [ 0.4, 0.4, 0.4, 0.6 ],
    width: 2.0,
    cap_style: LineCap::Round,
    join_style: LineJoin::Round,
  };
  
  // Top border
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 50.0, y: 100.0 },
    end: Point2D { x: 750.0, y: 100.0 },
    style: border_style,
  } ) );
  
  // Bottom border
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 50.0, y: 500.0 },
    end: Point2D { x: 750.0, y: 500.0 },
    style: border_style,
  } ) );
  
  // Left border
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 50.0, y: 100.0 },
    end: Point2D { x: 50.0, y: 500.0 },
    style: border_style,
  } ) );
  
  // Right border
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 750.0, y: 100.0 },
    end: Point2D { x: 750.0, y: 500.0 },
    style: border_style,
  } ) );
  
  // Add footer text
  let mut footer_text = [ 0u8; 64 ];
  let footer = b"Generated with SVG-Browser Backend";
  footer_text[ ..footer.len() ].copy_from_slice( footer );
  
  scene.add( RenderCommand::Text( TextCommand
  {
    text: footer_text,
    text_len: footer.len() as u8,
    position: Point2D { x: 400.0, y: 550.0 },
    font_style: FontStyle
    {
      color: [ 0.5, 0.5, 0.5, 1.0 ], // Medium gray
      size: 12.0,
      weight: 300,
      italic: true,
      family_id: 0,
    },
    anchor: TextAnchor::Center,
  } ) );
  
  // Render the interactive scene
  renderer.initialize( &context )?;
  renderer.begin_frame( &context )?;
  renderer.render_scene( &scene )?;
  renderer.end_frame()?;
  
  // Get the complete HTML output
  let html_output = renderer.output()?;
  
  // Save to file for viewing in browser
  std::fs::write( "-interactive_demo.html", &html_output )?;
  
  println!( "Interactive SVG demo generated!" );
  println!( "Open -interactive_demo.html in your web browser to see the interactive elements." );
  println!( "Features included:" );
  println!( "  • Hover effects on all elements" );
  println!( "  • Click handling with visual feedback" );
  println!( "  • Interactive mouse picking" );
  println!( "  • JavaScript-powered interactivity" );
  println!( "  • Responsive design elements" );
  
  Ok( () )
}