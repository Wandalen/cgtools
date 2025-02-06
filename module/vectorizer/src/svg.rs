//!
//! List demo in Embroidery API (command part).
//!

mod private
{
  use std::fmt;

  /// Wrapper around attributes needed to construct an SVG file.
  ///
  /// This struct represents an SVG file and contains attributes such as paths, width, height,
  /// and the number of decimal places to use in the path string.
  ///
  /// # Fields
  /// * `paths` - A vector of `SvgPath` representing the paths in the SVG file.
  /// * `width` - The width of the SVG file.
  /// * `height` - The height of the SVG file.
  /// * `path_precision` - The number of decimal places to use in the path string.
  #[ derive( Debug, Clone ) ]
  pub struct SvgFile 
  {
    /// <path> attributes in svg file
    pub paths : Vec< SvgPath >,
    /// Width of svg file in pixels
    pub width : usize,
    /// Height of svg file in pixels
    pub height : usize,
    /// Number of decimal places to use in path string
    pub path_precision : Option< u32 >,
  }

  /// Represents a <path> attribute in an SVG file.
  ///
  /// This struct contains a collection of `Path` and `Spline` that represents a shape with holes,
  /// and the color to fill the path.
  ///
  /// # Fields
  /// * `path` - A collection of `Path` and `Spline` that represents a shape with holes.
  /// * `color` - The color to fill the path.
  #[ derive( Debug, Clone ) ]
  pub struct SvgPath
  {
    /// A collection of `Path` and `Spline` that represents a shape with holes
    pub path : visioncortex::CompoundPath,
    /// Color to fill the path
    pub color : visioncortex::Color,
  }

  impl SvgFile 
  {
    /// Constructs a new instance of `SvgFile`
    pub fn new( width : usize, height : usize, path_precision : Option< u32 > ) -> Self 
    {
      SvgFile 
      {
        paths: vec![],
        width,
        height,
        path_precision,
      }
    }

    /// Add a path to the file
    pub fn add_path( &mut self, path : visioncortex::CompoundPath, color : visioncortex::Color ) 
    {
      self.paths.push( SvgPath { path, color } )
    }
  }

  impl fmt::Display for SvgFile 
  {
    /// Displays `SvgFile` in proper xml representation
    fn fmt( &self, f : &mut fmt::Formatter::< '_ > ) -> fmt::Result 
    {
      writeln!( f, r#"<?xml version="1.0" encoding="UTF-8"?>"# )?;
      writeln!
      (
        f,
        r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
        self.width, 
        self.height
      )?;

      for path in &self.paths 
      {
        path.fmt_with_precision( f, self.path_precision )?;
      }

      writeln!( f, "</svg>" )
    }
  }

  impl fmt::Display for SvgPath 
  {
    fn fmt( &self, f : &mut fmt::Formatter::< '_ > ) -> fmt::Result 
    {
      self.fmt_with_precision( f, None )
    }
  }

  impl SvgPath 
  {
    /// Convert `SvgPath` to a <path> string
    fn fmt_with_precision( &self, f : &mut fmt::Formatter::< '_ >, precision : Option< u32 > ) -> fmt::Result 
    {
      let ( string, offset ) = self.path.to_svg_string( true, visioncortex::PointF64::default(), precision );
      writeln!
      (
        f,
        "<path d=\"{}\" fill=\"{}\" transform=\"translate({},{})\"/>",
        string,
        self.color.to_hex_string(),
        offset.x,
        offset.y
      )
    }
  }
}

crate::mod_interface!
{
  own use 
  {
    SvgFile,
    SvgPath
  };
}