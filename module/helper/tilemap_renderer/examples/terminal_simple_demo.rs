//! Simple terminal renderer demonstration.
//!
//! This example shows the terminal backend adapter rendering basic shapes.

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::needless_return ) ]

use tilemap_renderer::
{
  adapters::TerminalRenderer,
  ports::{ Renderer, PrimitiveRenderer, RenderContext },
  commands::{ Point2D, StrokeStyle, LineCommand, CurveCommand, LineCap, LineJoin },
};

fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Create a terminal renderer with smaller dimensions for demo
  let mut renderer = TerminalRenderer::with_dimensions( 40, 15 );
  
  // Enable Unicode and colors
  renderer.set_unicode_enabled( true );
  renderer.set_color_enabled( true );
  
  // Initialize with a default context
  let context = RenderContext::default();
  renderer.initialize( &context )?;
  renderer.begin_frame( &context )?;
  
  // Create and render a horizontal line
  let line = LineCommand
  {
    start: Point2D { x: 5.0, y: 3.0 },
    end: Point2D { x: 35.0, y: 3.0 },
    style: StrokeStyle
    {
      color: [ 1.0, 0.0, 0.0, 1.0 ], // Red
      width: 2.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  
  renderer.render_line( &line )?;
  
  // Create and render a vertical line
  let vertical_line = LineCommand
  {
    start: Point2D { x: 10.0, y: 5.0 },
    end: Point2D { x: 10.0, y: 12.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 1.0, 0.0, 1.0 ], // Green
      width: 1.0,
      cap_style: LineCap::Butt,
      join_style: LineJoin::Miter,
    },
  };
  
  renderer.render_line( &vertical_line )?;
  
  // Create and render a curve
  let curve = CurveCommand
  {
    start: Point2D { x: 15.0, y: 8.0 },
    control1: Point2D { x: 25.0, y: 5.0 },
    control2: Point2D { x: 25.0, y: 11.0 },
    end: Point2D { x: 35.0, y: 8.0 },
    style: StrokeStyle
    {
      color: [ 0.0, 0.0, 1.0, 1.0 ], // Blue
      width: 1.0,
      cap_style: LineCap::Round,
      join_style: LineJoin::Round,
    },
  };
  
  renderer.render_curve( &curve )?;
  
  // End frame and get output
  renderer.end_frame()?;
  let output = renderer.output()?;
  
  // Display the result
  println!( "Terminal Renderer Demo:" );
  println!( "{output}" );
  
  // Export to file
  renderer.export_to_file( "-terminal_simple_demo.txt" )?;
  println!( "Output saved to -terminal_simple_demo.txt" );
  
  Ok( () )
}