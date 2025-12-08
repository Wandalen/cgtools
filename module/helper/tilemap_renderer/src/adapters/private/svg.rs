#![ allow( clippy::implicit_return ) ]

//! SVG backend adapter implementation.
//!
//! This adapter renders scenes to SVG format, providing scalable vector output
//! suitable for web display, printing, and vector graphics workflows.

use core::fmt::Write as _;
use base64::Engine;
use rustc_hash::FxHashMap;
use crate::scene::Scene;
use crate::ports::
{
  RenderContext,
  Renderer,
  RendererCapabilities,
  RenderError,
  PrimitiveRenderer
};
use crate::commands::
{
  CurveCommand,
  LineCap,
  LineCommand,
  LineJoin,
  ParticleEmitterCommand,
  RenderCommand,
  TextAnchor,
  TextCommand,
  TilemapCommand,
  Transform2D
};

/// SVG renderer backend adapter.
///
/// Renders scenes to SVG (Scalable Vector Graphics) format with comprehensive
/// support for lines, curves, text, and basic shapes.
#[ derive( Debug, Clone ) ]
pub struct SvgRenderer
{
  /// SVG content buffer.
  svg_content : String,
  /// Current rendering state.
  initialized : bool,
  /// Whether a frame is currently active.
  frame_active : bool,
  /// Rendering context for the current frame.
  context : Option< RenderContext >,
  images : FxHashMap< String, ( u32, u32 ) >,
  framebegin_index : usize,
  frameend_index : usize,
}

impl SvgRenderer
{
  /// Creates a new SVG renderer instance.
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    Self
    {
      svg_content : String::new(),
      initialized : false,
      frame_active : false,
      context : None,
      images : FxHashMap::default(),
      framebegin_index : 0,
      frameend_index : 0,
    }
  }

  #[ inline ]
  #[ must_use ]
  /// Returns render context
  pub const fn context( &self ) -> Option< &RenderContext >
  {
    self.context.as_ref()
  }

  /// Converts a color array to SVG color string.
  #[ inline ]
  #[ allow( clippy::cast_possible_truncation ) ]
  #[ allow( clippy::cast_sign_loss ) ]
  fn color_to_svg( color : &[ f32; 4 ] ) -> String
  {
    let red = ( color[ 0 ] * 255.0 ) as u8;
    let green = ( color[ 1 ] * 255.0 ) as u8;
    let blue = ( color[ 2 ] * 255.0 ) as u8;
    let alpha = color[ 3 ];

    if ( alpha - 1.0 ).abs() < f32::EPSILON
    {
      return format!( "rgb({red},{green},{blue})" );
    }

    format!( "rgba({red},{green},{blue},{alpha})" )
  }

  /// Converts line cap style to SVG stroke-linecap.
  #[ inline ]
  fn line_cap_to_svg( cap : LineCap ) -> &'static str
  {
    match cap
    {
      LineCap::Butt => "butt",
      LineCap::Round => "round",
      LineCap::Square => "square",
    }
  }

  /// Converts line join style to SVG stroke-linejoin.
  #[ inline ]
  fn line_join_to_svg( join : LineJoin ) -> &'static str
  {
    match join
    {
      LineJoin::Miter => "miter",
      LineJoin::Round => "round",
      LineJoin::Bevel => "bevel",
    }
  }

  /// Resolves font family from font family ID.
  /// For now, this is a simple mapping. In a full implementation,
  /// this would lookup from a font registry.
  #[ inline ]
  fn resolve_font_family( family_id : u32 ) -> &'static str
  {
    match family_id
    {
      0 => "Arial",
      1 => "Times New Roman",
      2 => "Courier New",
      3 => "Helvetica",
      4 => "Georgia",
      _ => "sans-serif", // Fallback
    }
  }

  /// Renders a 2D polygon with a given transformation and style.
  ///
  /// This method converts the provided points and style into an SVG `<polygon>` element,
  /// applies the necessary transformations (including viewport offset, zoom, and centering),
  /// and appends it to the current frame's SVG content.
  ///
  /// ## Arguments
  ///
  /// * `points` - A slice of `f32` representing the polygon's vertices as `[x1, y1, x2, y2, ...]`.
  /// * `transform` - The `Transform2D` to apply to the polygon.
  /// * `style` - The `GeometryStyle` defining the fill, stroke, and stroke width.
  ///
  /// ## Errors
  ///
  /// Returns error if renderer is not initialized.
  #[ inline ]
  #[ allow( clippy::cast_precision_loss ) ]
  pub fn render_geometry( &mut self, points : &[ f32 ], mut transform : Transform2D, style : GeometryStyle ) -> Result< (), RenderError >
  {
    if points.len() < 6
    {
      // Not a valid polygon, but we can just ignore it.
      return Ok( () );
    }

    let ctx = self.context.ok_or( RenderError::RenderFailed( "Render is not initialized".to_owned() ) )?;

    transform.position[ 0 ] += ctx.viewport_offset.x;
    transform.position[ 1 ] += ctx.viewport_offset.y;

    transform.position[ 1 ] = -transform.position[ 1 ];

    transform.position[ 0 ] += ctx.width as f32 / 2.0;
    transform.position[ 1 ] += ctx.height as f32 / 2.0;

    transform.rotation = transform.rotation.to_degrees();
    let zoom = ctx.viewport_scale;
    // Convert the vector of points into an SVG-compatible string
    let mut points_str = String::with_capacity( points.len() * 2 );
    for chunk in points.chunks_exact( 2 )
    {
      write!
      (
        &mut points_str,
        "{},{} ", chunk[ 0 ], chunk[ 1 ]
      ).map_err( | e | RenderError::RenderFailed( e.to_string() ) )?; // this should never panic
    }
    points_str.pop();

    let fill = style.fill_color.map_or( "none".to_string(), | col | Self::color_to_svg( &col ) );

    let stroke = style.stroke_color.map_or( "none".to_string(), | col | Self::color_to_svg( &col ) );

    self.push_frame_content
    (
      &format!
      (
        r#"<polygon points="{}" fill="{}" stroke="{}" stroke-width="{}" transform="scale({}) translate({}, {}) rotate({}) scale({}, {})"/>"#,
        points_str, fill, stroke, style.stroke_width, zoom,
        transform.position[ 0 ], transform.position[ 1 ],
        transform.rotation,
        transform.scale[ 0 ], transform.scale[ 1 ]
      )
    );

    Ok( () )
  }

  /// Loads image data into the SVG's `<defs>` section as a reusable `<symbol>`.
  ///
  /// The image data is Base64 encoded and embedded directly into the SVG. This allows the image
  /// to be referenced by its `id` and rendered multiple times without re-uploading the data.
  ///
  /// ## Arguments
  ///
  /// * `bytes` - The raw byte data of the image.
  /// * `width` - The width of the image.
  /// * `height` - The height of the image.
  /// * `format` - The `ImageFormat` of the data (e.g., PNG, JPEG).
  /// * `id` - A unique string identifier for this image.
  #[ inline ]
  pub fn load_image( &mut self, bytes : &[ u8 ], width : u32, height : u32, format : ImageFormat, id : &str )
  {
    let img = base64::prelude::BASE64_STANDARD.encode( bytes );
    let def = format!
    (
      r#"<defs><symbol id="{id}" viewBox="-{} -{} {width} {height}"><image href="data:image/{};base64,{img}" x="-{}" y="-{}" width="{width}" height="{height}"/></symbol></defs>"#,
      width / 2, height / 2, format.as_ref(), width / 2, height / 2
    );
    self.svg_content.insert_str( self.framebegin_index, &def );
    self.framebegin_index += def.len();
    self.frameend_index += def.len();
    _ = self.images.insert( id.to_string(), ( width, height ) );
  }

  /// Renders a previously loaded image (symbol) with a given transformation.
  ///
  /// This method creates an SVG `<use>` element to instance a symbol from the `<defs>` section.
  /// It applies the necessary transformations to position, scale, and rotate the image correctly.
  ///
  /// ## Arguments
  ///
  /// * `id` - The string identifier of the image to render.
  /// * `transform` - The `Transform2D` to apply to the image.
  ///
  /// ## Errors
  ///
  /// Returns error if renderer is not initialized.
  #[ inline ]
  #[ allow( clippy::cast_precision_loss ) ]
  pub fn render_image( &mut self, id : &str, mut transform : Transform2D ) -> Result< (), RenderError >
  {
    let ctx = self.context.ok_or( RenderError::RenderFailed( "Render is not initialized".to_owned() ) )?;

    transform.position[ 0 ] += ctx.viewport_offset.x;
    transform.position[ 1 ] += ctx.viewport_offset.y;

    transform.position[ 1 ] = -transform.position[ 1 ];

    transform.position[ 0 ] += ctx.width as f32 / 2.0;
    transform.position[ 1 ] += ctx.height as f32 / 2.0;

    transform.rotation = transform.rotation.to_degrees();
    let zoom = ctx.viewport_scale;
    let ( width, height ) = self.images.get( id ).ok_or( RenderError::RenderFailed( format!( "Image with id: {id} is not loaded" ) ) )?;
    let str = format!
    (
      "<g transform=\"scale({}) translate({}, {}) rotate({}) scale({}, {})\"><use href=\"#{id}\" x=\"-{}\" y=\"-{}\" width=\"{width}\" height=\"{height}\"/></g>",
      zoom,
      transform.position[ 0 ],
      transform.position[ 1 ],
      transform.rotation,
      transform.scale[ 0 ],
      transform.scale[ 1 ],
      width / 2,
      height / 2,
    );

    self.push_frame_content( &str );

    Ok( () )
  }

  fn push_frame_content( &mut self, str : &str )
  {
    self.svg_content.insert_str( self.frameend_index, str );
    self.frameend_index += str.len();
  }

  /// Clears all rendered content from the current frame.
  ///
  /// This method removes all SVG elements that were added between the frame begin and end markers,
  /// preparing the renderer for the next frame.
  #[ inline ]
  pub fn clear( &mut self )
  {
    let begin_index = self.framebegin_index + "<!--framebegin-->".len();
    self.svg_content.replace_range( begin_index..self.frameend_index, "" );
    self.frameend_index = begin_index;
  }
}

impl Default for SvgRenderer
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

impl Renderer for SvgRenderer
{
  type Output = String;

  #[ inline ]
  fn capabilities( &self ) -> RendererCapabilities
  {
    RendererCapabilities
    {
      backend_name : "SVG".to_string(),
      backend_version : "1.0".to_string(),
      max_texture_size : 0, // SVG doesn't have texture size limits
      supports_transparency : true,
      supports_antialiasing : true, // SVG browsers handle this
      supports_custom_fonts : true,
      supports_particles : false, // Not implemented for SVG
      supports_realtime : false, // SVG is static
      max_scene_complexity : 10000, // Large scenes supported
      ..Default::default()
    }
  }

  /// Initializes the SVG renderer with the given context.
  ///
  /// # Errors
  /// Returns `InitializationFailed` if the renderer cannot be initialized.
  #[ inline ]
  fn initialize( &mut self, context : &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if self.initialized
    {
      return Err( RenderError::InitializationFailed( "SVG renderer already initialized".to_string() ) );
    }

    self.context = Some( *context );
    self.initialized = true;
    self.frame_active = true;
    self.svg_content.clear();

    // Start SVG document
    write!
    (
      &mut self.svg_content,
      r#"<?xml version="1.0" encoding="UTF-8"?> <svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
      context.width, context.height, context.width, context.height
    ).map_err( | e | RenderError::RenderFailed( e.to_string() ) )?; // this should never ever panic

    // Add background if needed
    if context.clear_background
    {
      let bg_color = Self::color_to_svg( &context.background_color );
      write!
      (
        &mut self.svg_content,
        r#"<rect width="100%" height="100%" fill="{bg_color}"/>"#
      ).map_err( | e | RenderError::RenderFailed( e.to_string() ) )?;
    }

    self.framebegin_index = self.svg_content.len();
    self.svg_content.push_str( "<!--framebegin-->" );
    self.frameend_index = self.svg_content.len();
    self.svg_content.push_str( "<!--frameend-->" );
    self.svg_content.push_str( "</svg>\n" );

    Ok( () )
  }

  /// Begins a new rendering frame.
  ///
  /// # Errors
  /// Returns `InvalidContext` if the context is invalid or `RenderFailed` if frame setup fails.
  #[ inline ]
  fn begin_frame( &mut self, _ : &RenderContext ) -> core::result::Result< (), RenderError >
  {
    if !self.initialized
    {
      return Err( RenderError::InvalidContext( "SVG renderer not initialized".to_string() ) );
    }
    Ok( () )
  }

  /// Renders a complete scene to the output.
  ///
  /// # Errors
  /// Returns various errors including `UnsupportedCommand`, `ComplexityLimitExceeded`, or `RenderFailed`.
  #[ inline ]
  fn render_scene( &mut self, scene : &Scene ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }

    self.validate_scene( scene )?;

    for command in scene.commands()
    {
      match command
      {
        RenderCommand::Line( line_cmd ) => self.render_line( line_cmd )?,
        RenderCommand::Curve( curve_cmd ) => self.render_curve( curve_cmd )?,
        RenderCommand::Text( text_cmd ) => self.render_text( text_cmd )?,
        RenderCommand::Tilemap( _ ) => return Err( RenderError::UnsupportedCommand( "Tilemap".to_string() ) ),
        RenderCommand::ParticleEmitter( _ ) => return Err( RenderError::UnsupportedCommand( "ParticleEmitter".to_string() ) ),
        RenderCommand::Geometry2DCommand( _ ) => return Err( RenderError::UnsupportedCommand( "Geometry2DCommand".into() ) ),
        RenderCommand::SpriteCommand( _ ) => return Err( RenderError::UnsupportedCommand( "SpriteCommand".into() ) ),
      }
    }

    Ok( () )
  }

  /// Ends the current rendering frame and finalizes the output.
  ///
  /// # Errors
  /// Returns `RenderFailed` if frame finalization fails.
  #[ inline ]
  fn end_frame( &mut self ) -> core::result::Result< (), RenderError >
  {
    if !self.frame_active
    {
      return Err( RenderError::RenderFailed( "No active frame".to_string() ) );
    }

    Ok( () )
  }

  /// Retrieves the rendered output.
  ///
  /// # Errors
  /// Returns `OutputError` if the output cannot be retrieved.
  #[ inline ]
  fn output( &self ) -> core::result::Result< Self::Output, RenderError >
  {
    if !self.initialized
    {
      return Err( RenderError::OutputError( "Renderer not initialized".to_string() ) );
    }

    Ok( self.svg_content.clone() )
  }

  /// Performs cleanup and releases resources.
  ///
  /// # Errors
  /// Returns `RenderFailed` if cleanup operations fail.
  #[ inline ]
  fn cleanup( &mut self ) -> core::result::Result< (), RenderError >
  {
    self.initialized = false;
    self.frame_active = false;
    self.svg_content.clear();
    self.context = None;
    Ok( () )
  }

  #[ inline ]
  fn supports_tilemaps( &self ) -> bool
  {
    false // SVG adapter doesn't support tilemaps
  }

  #[ inline ]
  fn supports_particles( &self ) -> bool
  {
    false // SVG adapter doesn't support particle effects
  }
}

impl PrimitiveRenderer for SvgRenderer
{
  /// Renders a line command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if line rendering fails.
  #[ inline ]
  fn render_line( &mut self, command : &LineCommand ) -> core::result::Result< (), RenderError >
  {
    let stroke_color = Self::color_to_svg( &command.style.color );
    let line_cap = Self::line_cap_to_svg( command.style.cap_style );
    let line_join = Self::line_join_to_svg( command.style.join_style );

    self.push_frame_content( &format!(
      r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}"/>"#,
      command.start.x, command.start.y,
      command.end.x, command.end.y,
      stroke_color, command.style.width,
      line_cap, line_join
    ) );

    Ok( () )
  }

  /// Renders a curve command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if curve rendering fails.
  #[ inline ]
  fn render_curve( &mut self, command : &CurveCommand ) -> core::result::Result< (), RenderError >
  {
    let stroke_color = Self::color_to_svg( &command.style.color );
    let line_cap = Self::line_cap_to_svg( command.style.cap_style );
    let line_join = Self::line_join_to_svg( command.style.join_style );

    // SVG cubic bezier curve
    let path_data = format!(
      "M {} {} C {} {}, {} {}, {} {}",
      command.start.x, command.start.y,
      command.control1.x, command.control1.y,
      command.control2.x, command.control2.y,
      command.end.x, command.end.y
    );

    self.push_frame_content
    (
      &format!
      (
        r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}"/>"#,
        path_data, stroke_color, command.style.width,
        line_cap, line_join
      )
    );

    Ok( () )
  }

  /// Renders a text command.
  ///
  /// # Errors
  /// Returns `RenderFailed` if text rendering fails.
  #[ inline ]
  fn render_text( &mut self, command: &TextCommand ) -> core::result::Result< (), RenderError >
  {
    let fill_color = Self::color_to_svg( &command.font_style.color );
    let text_content = command.text();

    // Convert text anchor to SVG text-anchor
    let text_anchor = match command.anchor
    {
      TextAnchor::TopLeft | TextAnchor::CenterLeft | TextAnchor::BottomLeft => "start",
      TextAnchor::TopCenter | TextAnchor::Center | TextAnchor::BottomCenter => "middle",
      TextAnchor::TopRight | TextAnchor::CenterRight | TextAnchor::BottomRight => "end",
    };

    // Convert vertical alignment to dominant-baseline
    let dominant_baseline = match command.anchor
    {
      TextAnchor::TopLeft | TextAnchor::TopCenter | TextAnchor::TopRight => "hanging",
      TextAnchor::CenterLeft | TextAnchor::Center | TextAnchor::CenterRight => "central",
      TextAnchor::BottomLeft | TextAnchor::BottomCenter | TextAnchor::BottomRight => "baseline",
    };

    self.push_frame_content( &format!(
      r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" text-anchor="{}" dominant-baseline="{}">{}</text>"#,
      command.position.x, command.position.y,
      Self::resolve_font_family( command.font_style.family_id ), command.font_style.size,
      fill_color, text_anchor, dominant_baseline,
      text_content
    ) );

    Ok( () )
  }

  /// Renders a tilemap command.
  ///
  /// # Errors
  /// Returns `FeatureNotImplemented` if tilemap rendering is not supported.
  #[ inline ]
  fn render_tilemap( &mut self, _command: &TilemapCommand ) -> core::result::Result< (), RenderError >
  {
    Err( RenderError::FeatureNotImplemented( "Tilemap rendering not supported in SVG backend".to_string() ) )
  }

  /// Renders a particle emitter command.
  ///
  /// # Errors
  /// Returns `FeatureNotImplemented` if particle rendering is not supported.
  #[ inline ]
  fn render_particle_emitter( &mut self, _command: &ParticleEmitterCommand ) -> core::result::Result< (), RenderError >
  {
    Err( RenderError::FeatureNotImplemented( "Particle rendering not supported in SVG backend".to_string() ) )
  }
}

/// Defines the visual styling for rendering 2D geometry, including fill and stroke properties.
#[ derive( Debug, Clone, Copy ) ]
#[ allow( clippy::exhaustive_structs ) ]
pub struct GeometryStyle
{
  /// The optional RGBA color used to fill the shape. If `None`, the shape will not be filled.
  pub fill_color : Option< [ f32; 4 ] >,
  /// The optional RGBA color used for the shape's outline. If `None`, there will be no outline.
  pub stroke_color : Option< [ f32; 4 ] >,
  /// The width or thickness of the shape's outline.
  pub stroke_width : f32,
}

/// An enumeration of supported image formats.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
#[ allow( clippy::exhaustive_enums ) ]
pub enum ImageFormat
{
  /// PNG format.
  Png,
  /// JPEG format.
  Jpeg,
}

impl AsRef< str > for ImageFormat
{
  #[ inline ]
  fn as_ref( &self ) -> &str
  {
    match self
    {
      ImageFormat::Png => "png",
      ImageFormat::Jpeg => "jpeg",
    }
  }
}
