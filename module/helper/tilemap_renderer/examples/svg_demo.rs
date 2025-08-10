//! SVG renderer demonstration example.
//!
//! This example shows how to use the SVG backend adapter to generate
//! vector graphics output from rendering commands.

#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::uninlined_format_args ) ]

use tilemap_renderer::adapters::svg::SvgRenderer;
use tilemap_renderer::ports::*;
use tilemap_renderer::scene::*;
use tilemap_renderer::commands::*;

fn main() -> Result<(), Box<dyn std::error::Error>>
{
  // Create SVG renderer
  let mut renderer = SvgRenderer::new();
  
  // Set up rendering context
  let mut context = RenderContext::default();
  context.width = 400;
  context.height = 300;
  context.background_color = [ 0.95, 0.95, 0.95, 1.0 ]; // Light gray background
  
  // Create a scene with various primitives
  let mut scene = Scene::with_id( "demo_scene" );
  
  // Add a red line
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 50.0, y: 50.0 },
    end: Point2D { x: 350.0, y: 100.0 },
    style: StrokeStyle
    {
      width: 3.0,
      color: [ 1.0, 0.0, 0.0, 1.0 ], // Red
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  } ) );
  
  // Add a blue curve
  scene.add( RenderCommand::Curve( CurveCommand
  {
    start: Point2D { x: 50.0, y: 150.0 },
    control1: Point2D { x: 150.0, y: 100.0 },
    control2: Point2D { x: 250.0, y: 200.0 },
    end: Point2D { x: 350.0, y: 150.0 },
    style: StrokeStyle
    {
      width: 2.5,
      color: [ 0.0, 0.5, 1.0, 1.0 ], // Blue
      cap_style: LineCap::Square,
      join_style: LineJoin::Miter,
    },
  } ) );
  
  // Add green title text
  scene.add( RenderCommand::Text( TextCommand::new(
    "SVG Rendering Demo",
    Point2D { x: 200.0, y: 30.0 },
    FontStyle
    {
      size: 18.0,
      color: [ 0.0, 0.7, 0.0, 1.0 ], // Green
      weight: 600, // Semi-bold
      italic: false,
      family_id: 3, // Helvetica
    },
    TextAnchor::Center
  ) ) );
  
  // Add some descriptive text
  scene.add( RenderCommand::Text( TextCommand::new(
    "This demonstrates line, curve, and text rendering",
    Point2D { x: 200.0, y: 250.0 },
    FontStyle
    {
      size: 14.0,
      color: [ 0.3, 0.3, 0.3, 1.0 ], // Dark gray
      weight: 400, // Normal
      italic: true,
      family_id: 1, // Times New Roman
    },
    TextAnchor::Center
  ) ) );
  
  // Add a purple decorative line
  scene.add( RenderCommand::Line( LineCommand
  {
    start: Point2D { x: 100.0, y: 200.0 },
    end: Point2D { x: 300.0, y: 200.0 },
    style: StrokeStyle
    {
      width: 1.5,
      color: [ 0.6, 0.2, 0.8, 0.7 ], // Semi-transparent purple
      cap_style: LineCap::Butt,
      join_style: LineJoin::Bevel,
    },
  } ) );
  
  // Render the scene
  renderer.initialize( &context )?;
  renderer.begin_frame( &context )?;
  renderer.render_scene( &scene )?;
  renderer.end_frame()?;
  
  // Get the SVG output
  let svg_output = renderer.output()?;
  
  // Print the SVG content
  println!( "Generated SVG:" );
  println!( "{}", svg_output );
  
  // Also save to file
  std::fs::write( "-demo_output.svg", &svg_output )?;
  println!( "\nSVG saved to -demo_output.svg" );
  println!( "You can open this file in a web browser or SVG viewer to see the rendered graphics." );
  
  Ok( () )
}