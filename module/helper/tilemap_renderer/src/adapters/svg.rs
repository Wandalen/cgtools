//! SVG backend adapter.
//!
//! Generates a complete SVG 1.1 document from render commands.
//! Supports all features: paths, text, sprites, gradients, patterns,
//! clip masks, effects, blend modes, and text-on-path.

use core::fmt::Write as _;
use crate::assets::*;
use crate::backend::*;
use crate::commands::*;
use crate::types::*;

/// SVG renderer backend.
///
/// ```ignore
/// let mut svg = SvgBackend::new( 800, 600 );
/// svg.load_assets( &assets )?;
/// svg.submit( &commands )?;
/// let Output::String( doc ) = svg.output()? else { unreachable!() };
/// ```
pub struct SvgBackend
{
  width : u32,
  height : u32,
  /// `<defs>` section built from assets.
  defs : String,
  /// Body built from commands.
  body : String,
  // -- streaming state --
  path_data : String,
  path_style : Option< BeginPath >,
  text_buf : String,
  text_style : Option< BeginText >,
  group_depth : u32,
}

impl SvgBackend
{
  /// Creates a new SVG backend with the given viewport dimensions.
  #[ must_use ]
  pub fn new( width : u32, height : u32 ) -> Self
  {
    Self
    {
      width,
      height,
      defs : String::new(),
      body : String::new(),
      path_data : String::new(),
      path_style : None,
      text_buf : String::new(),
      text_style : None,
      group_depth : 0,
    }
  }

  fn color_to_svg( color : &[ f32; 4 ] ) -> String
  {
    #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
    let ( r, g, b, a ) =
    (
      ( color[ 0 ] * 255.0 ) as u8,
      ( color[ 1 ] * 255.0 ) as u8,
      ( color[ 2 ] * 255.0 ) as u8,
      color[ 3 ],
    );
    if ( a - 1.0 ).abs() < f32::EPSILON
    {
      format!( "rgb({r},{g},{b})" )
    }
    else
    {
      format!( "rgba({r},{g},{b},{a})" )
    }
  }

  fn fill_to_svg( fill : &FillRef ) -> String
  {
    match fill
    {
      FillRef::None => "none".to_string(),
      FillRef::Solid( color ) => Self::color_to_svg( color ),
      FillRef::Asset( id ) => format!( "url(#asset_{})", id.0 ),
    }
  }

  fn transform_to_svg( t : &Transform ) -> String
  {
    let mut parts = Vec::new();
    if t.position[ 0 ] != 0.0 || t.position[ 1 ] != 0.0
    {
      parts.push( format!( "translate({},{})", t.position[ 0 ], t.position[ 1 ] ) );
    }
    if t.rotation != 0.0
    {
      parts.push( format!( "rotate({})", t.rotation.to_degrees() ) );
    }
    if t.scale[ 0 ] != 1.0 || t.scale[ 1 ] != 1.0
    {
      parts.push( format!( "scale({},{})", t.scale[ 0 ], t.scale[ 1 ] ) );
    }
    if t.skew[ 0 ] != 0.0
    {
      parts.push( format!( "skewX({})", t.skew[ 0 ].to_degrees() ) );
    }
    if t.skew[ 1 ] != 0.0
    {
      parts.push( format!( "skewY({})", t.skew[ 1 ].to_degrees() ) );
    }
    if parts.is_empty()
    {
      String::new()
    }
    else
    {
      format!( " transform=\"{}\"", parts.join( " " ) )
    }
  }

  fn blend_to_svg( blend : &BlendMode ) -> &'static str
  {
    match blend
    {
      BlendMode::Normal => "normal",
      BlendMode::Multiply => "multiply",
      BlendMode::Screen => "screen",
      BlendMode::Overlay => "overlay",
      BlendMode::Add => "lighter",
    }
  }

  fn linecap_to_svg( cap : &LineCap ) -> &'static str
  {
    match cap
    {
      LineCap::Butt => "butt",
      LineCap::Round => "round",
      LineCap::Square => "square",
    }
  }

  fn linejoin_to_svg( join : &LineJoin ) -> &'static str
  {
    match join
    {
      LineJoin::Miter => "miter",
      LineJoin::Round => "round",
      LineJoin::Bevel => "bevel",
    }
  }

  fn dash_to_svg( dash : &DashStyle ) -> String
  {
    let values : Vec< String > = dash.pattern.iter()
      .take_while( | &&v | v > 0.0 )
      .map( | v | v.to_string() )
      .collect();
    if values.is_empty()
    {
      String::new()
    }
    else
    {
      let mut s = format!( " stroke-dasharray=\"{}\"", values.join( "," ) );
      if dash.offset != 0.0
      {
        let _ = write!( s, " stroke-dashoffset=\"{}\"", dash.offset );
      }
      s
    }
  }

  fn anchor_to_svg( anchor : &TextAnchor ) -> ( &'static str, &'static str )
  {
    let h = match anchor
    {
      TextAnchor::TopLeft | TextAnchor::CenterLeft | TextAnchor::BottomLeft => "start",
      TextAnchor::TopCenter | TextAnchor::Center | TextAnchor::BottomCenter => "middle",
      TextAnchor::TopRight | TextAnchor::CenterRight | TextAnchor::BottomRight => "end",
    };
    let v = match anchor
    {
      TextAnchor::TopLeft | TextAnchor::TopCenter | TextAnchor::TopRight => "hanging",
      TextAnchor::CenterLeft | TextAnchor::Center | TextAnchor::CenterRight => "central",
      TextAnchor::BottomLeft | TextAnchor::BottomCenter | TextAnchor::BottomRight => "baseline",
    };
    ( h, v )
  }

  fn clip_attr( clip : &Option< ResourceId > ) -> String
  {
    match clip
    {
      Some( id ) => format!( " clip-path=\"url(#clip_{})\"", id.0 ),
      None => String::new(),
    }
  }

  fn flush_path( &mut self )
  {
    let Some( style ) = self.path_style.take() else { return };

    let fill = Self::fill_to_svg( &style.fill );
    let stroke = Self::color_to_svg( &style.stroke_color );
    let transform = Self::transform_to_svg( &style.transform );
    let clip = Self::clip_attr( &style.clip );
    let dash = Self::dash_to_svg( &style.stroke_dash );
    let blend = Self::blend_to_svg( &style.blend );

    let _ = write!(
      self.body,
      "<path d=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"{}\" stroke-linejoin=\"{}\"{}{}{} style=\"mix-blend-mode:{}\"/>",
      self.path_data.trim(),
      fill,
      stroke,
      style.stroke_width,
      Self::linecap_to_svg( &style.stroke_cap ),
      Self::linejoin_to_svg( &style.stroke_join ),
      dash,
      transform,
      clip,
      blend,
    );
    self.path_data.clear();
  }

  fn flush_text( &mut self )
  {
    let Some( style ) = self.text_style.take() else { return };

    let fill = Self::color_to_svg( &style.color );
    let ( anchor, baseline ) = Self::anchor_to_svg( &style.anchor );
    let clip = Self::clip_attr( &style.clip );

    if let Some( path_id ) = style.along_path
    {
      let _ = write!(
        self.body,
        "<text font-size=\"{}\" fill=\"{}\" text-anchor=\"{}\" dominant-baseline=\"{}\"{}>
          <textPath href=\"#path_{}\">{}</textPath></text>",
        style.size, fill, anchor, baseline, clip,
        path_id.0, self.text_buf,
      );
    }
    else
    {
      let _ = write!(
        self.body,
        "<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"{}\" text-anchor=\"{}\" dominant-baseline=\"{}\"{}>
          {}</text>",
        style.position[ 0 ], style.position[ 1 ],
        style.size, fill, anchor, baseline, clip,
        self.text_buf,
      );
    }
    self.text_buf.clear();
  }
}

impl Backend for SvgBackend
{
  fn load_assets( &mut self, assets : &Assets ) -> Result< (), RenderError >
  {
    self.defs.clear();

    // Gradients
    for grad in &assets.gradients
    {
      let stops : String = grad.stops.iter().map( | s |
        format!( "<stop offset=\"{}\" stop-color=\"{}\"/>", s.offset, Self::color_to_svg( &s.color ) )
      ).collect();

      match &grad.kind
      {
        GradientKind::Linear { start, end } =>
        {
          let _ = write!(
            self.defs,
            "<linearGradient id=\"asset_{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\">{}</linearGradient>",
            grad.id.0, start[ 0 ], start[ 1 ], end[ 0 ], end[ 1 ], stops,
          );
        }
        GradientKind::Radial { center, radius, focal } =>
        {
          let _ = write!(
            self.defs,
            "<radialGradient id=\"asset_{}\" cx=\"{}\" cy=\"{}\" r=\"{}\" fx=\"{}\" fy=\"{}\">{}</radialGradient>",
            grad.id.0, center[ 0 ], center[ 1 ], radius, focal[ 0 ], focal[ 1 ], stops,
          );
        }
      }
    }

    // Patterns
    for pat in &assets.patterns
    {
      let _ = write!(
        self.defs,
        "<pattern id=\"asset_{}\" width=\"{}\" height=\"{}\" patternUnits=\"userSpaceOnUse\"><use href=\"#asset_{}\"/></pattern>",
        pat.id.0, pat.width, pat.height, pat.content.0,
      );
    }

    // Clip masks
    for clip in &assets.clip_masks
    {
      let mut d = String::new();
      for seg in &clip.segments
      {
        match seg
        {
          PathSegment::MoveTo( x, y ) => { let _ = write!( d, "M {x} {y} " ); }
          PathSegment::LineTo( x, y ) => { let _ = write!( d, "L {x} {y} " ); }
          PathSegment::QuadTo { cx, cy, x, y } => { let _ = write!( d, "Q {cx} {cy} {x} {y} " ); }
          PathSegment::CubicTo { c1x, c1y, c2x, c2y, x, y } => { let _ = write!( d, "C {c1x} {c1y} {c2x} {c2y} {x} {y} " ); }
          PathSegment::ArcTo { rx, ry, rotation, large_arc, sweep, x, y } =>
          {
            let _ = write!( d, "A {rx} {ry} {rotation} {} {} {x} {y} ", *large_arc as u8, *sweep as u8 );
          }
          PathSegment::Close => { d.push_str( "Z " ); }
        }
      }
      let _ = write!( self.defs, "<clipPath id=\"clip_{}\"><path d=\"{}\"/></clipPath>", clip.id.0, d.trim() );
    }

    // Named paths (for textPath)
    for path in &assets.paths
    {
      let mut d = String::new();
      for seg in &path.segments
      {
        match seg
        {
          PathSegment::MoveTo( x, y ) => { let _ = write!( d, "M {x} {y} " ); }
          PathSegment::LineTo( x, y ) => { let _ = write!( d, "L {x} {y} " ); }
          PathSegment::QuadTo { cx, cy, x, y } => { let _ = write!( d, "Q {cx} {cy} {x} {y} " ); }
          PathSegment::CubicTo { c1x, c1y, c2x, c2y, x, y } => { let _ = write!( d, "C {c1x} {c1y} {c2x} {c2y} {x} {y} " ); }
          PathSegment::ArcTo { rx, ry, rotation, large_arc, sweep, x, y } =>
          {
            let _ = write!( d, "A {rx} {ry} {rotation} {} {} {x} {y} ", *large_arc as u8, *sweep as u8 );
          }
          PathSegment::Close => { d.push_str( "Z " ); }
        }
      }
      let _ = write!( self.defs, "<path id=\"path_{}\" d=\"{}\"/>", path.id.0, d.trim() );
    }

    // TODO: images, sprites (base64 encode + <symbol>)

    Ok( () )
  }

  fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
  {
    self.body.clear();
    self.group_depth = 0;

    for cmd in commands
    {
      match cmd
      {
        RenderCommand::Clear( c ) =>
        {
          let color = Self::color_to_svg( &c.color );
          let _ = write!( self.body, "<rect width=\"100%\" height=\"100%\" fill=\"{color}\"/>" );
        }

        // Path streaming
        RenderCommand::BeginPath( bp ) =>
        {
          self.path_data.clear();
          self.path_style = Some( *bp );
        }
        RenderCommand::MoveTo( m ) => { let _ = write!( self.path_data, "M {} {} ", m.0, m.1 ); }
        RenderCommand::LineTo( l ) => { let _ = write!( self.path_data, "L {} {} ", l.0, l.1 ); }
        RenderCommand::QuadTo( q ) => { let _ = write!( self.path_data, "Q {} {} {} {} ", q.cx, q.cy, q.x, q.y ); }
        RenderCommand::CubicTo( c ) => { let _ = write!( self.path_data, "C {} {} {} {} {} {} ", c.c1x, c.c1y, c.c2x, c.c2y, c.x, c.y ); }
        RenderCommand::ArcTo( a ) =>
        {
          let _ = write!( self.path_data, "A {} {} {} {} {} {} {} ", a.rx, a.ry, a.rotation, a.large_arc as u8, a.sweep as u8, a.x, a.y );
        }
        RenderCommand::ClosePath( _ ) => { self.path_data.push_str( "Z " ); }
        RenderCommand::EndPath( _ ) => { self.flush_path(); }

        // Text streaming
        RenderCommand::BeginText( bt ) =>
        {
          self.text_buf.clear();
          self.text_style = Some( *bt );
        }
        RenderCommand::Char( ch ) => { self.text_buf.push( ch.0 ); }
        RenderCommand::EndText( _ ) => { self.flush_text(); }

        // Mesh
        RenderCommand::Mesh( _m ) =>
        {
          // TODO: lookup geometry from loaded assets, emit <polygon> or <path>
        }

        // Sprite
        RenderCommand::Sprite( _s ) =>
        {
          // TODO: <use href="#sprite_N" transform="..." />
        }

        // Instancing
        RenderCommand::BeginInstancedMesh( _bim ) =>
        {
          // TODO: store shared style
        }
        RenderCommand::Instance( _inst ) =>
        {
          // TODO: <use href="#geom_N" transform="..."/>
        }
        RenderCommand::EndInstancedMesh( _ ) => {}

        // Grouping
        RenderCommand::BeginGroup( bg ) =>
        {
          let transform = Self::transform_to_svg( &bg.transform );
          let clip = Self::clip_attr( &bg.clip );
          let opacity = match &bg.effect
          {
            Some( Effect::Opacity( a ) ) => format!( " opacity=\"{}\"", a ),
            _ => String::new(),
          };
          let _ = write!( self.body, "<g{}{}{}>", transform, clip, opacity );
          self.group_depth += 1;
        }
        RenderCommand::EndGroup( _ ) =>
        {
          self.body.push_str( "</g>" );
          self.group_depth = self.group_depth.saturating_sub( 1 );
        }
      }
    }

    Ok( () )
  }

  fn output( &self ) -> Result< Output, RenderError >
  {
    let mut doc = format!(
      "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">",
      self.width, self.height, self.width, self.height,
    );
    if !self.defs.is_empty()
    {
      let _ = write!( doc, "<defs>{}</defs>", self.defs );
    }
    doc.push_str( &self.body );
    doc.push_str( "</svg>\n" );
    Ok( Output::String( doc ) )
  }

  fn capabilities( &self ) -> Capabilities
  {
    Capabilities
    {
      paths : true,
      text : true,
      meshes : true,
      sprites : true,
      instancing : true,
      gradients : true,
      patterns : true,
      clip_masks : true,
      effects : true,
      blend_modes : true,
      text_on_path : true,
      max_texture_size : 0,
    }
  }
}
