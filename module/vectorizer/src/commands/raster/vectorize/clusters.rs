//!
//! List demo in Embroidery API (command part).
//!

mod private
{

  use crate::*;
  use clap::Parser;
  use commands::InputOutput;
  pub use commands::raster::
  {
    vectorize::PathSimplifyMode,
    common::color_parser
  };

  /// Parameters for the tracer
  #[ derive ( Debug, Parser ) ]
  pub struct CLIArgs
  {
    /// Input and output files' paths
    #[ clap( flatten ) ]
    pub io : InputOutput,
    /// Vectorization parameters
    #[ clap( flatten ) ]
    pub config : Config, 
  }

  /// Parameters for the tracer
  ///
  /// This struct contains the necessary parameters for configuring the vectorization process,
  /// including color precision, speckle filtering, gradient step, and other settings.
  ///
  /// # Fields
  /// * `color_precision` - The amount of significant bits to use when comparing `closeness` of two colors.
  ///   This parameter reduces the number of unique colors in the image by shifting the color bits.
  ///   Higher values retain more color detail, while lower values reduce the color palette.
  /// * `filter_speckle` - Discard patches smaller than X pixels in size.
  /// * `gradient_step` - Color difference between gradient layers.
  /// * `mode` - Curve fitting mode to use when creating a path.
  /// * `corner_threshold` - Minimum momentary angle (in degrees) to be considered a corner.
  /// * `segment_length` - Perform iterative subdivide smooth until all segments are shorter than this length.
  /// * `splice_threshold` - Minimum angle displacement (in degrees) to splice a spline.
  /// * `hierarchical` - Clustering mode to use for hierarchical clustering.
  /// * `remove_background` - Removes background color from the image
  /// * `background_color` - Manually specify the background color to remove
  /// * `background_similarity` - Threshold for colors to be considered as similar to `background`
  #[ derive ( Debug, Parser ) ]
  pub struct Config
  {
    /// The amount of significant bits to use when comparing `closeness` of two colors
    #[ arg( long, short = 'p', default_value = "8" ) ]
    pub color_precision : u8,
    /// Discard patches smaller than X px in size
    #[ arg( long, short, default_value = "4" ) ]
    pub filter_speckle : usize,
    /// Color difference between gradient layers
    #[ arg( long, short, default_value = "16" ) ]
    pub gradient_step : usize,
    /// Curve fitting mode
    #[ arg( long, short, value_enum, default_value_t = PathSimplifyMode::Spline ) ]
    pub mode : PathSimplifyMode,
    /// Minimum momentary angle ( in degrees ) to be considered a corner.
    #[ arg( long, default_value = "60.0" ) ]
    pub corner_threshold : f64,
    /// Perform iterative subdivide smooth until all segments are shorter than this length
    #[ arg( long, default_value = "4.0" ) ]
    pub segment_length : f64,
    /// Minimum angle displacement ( in degrees ) to splice a spline
    #[ arg( long, default_value = "45.0" ) ]
    pub splice_threshold : f64,
    /// Clustering mode
    #[ arg( long, value_enum, default_value_t = Hierarchical::Cutout ) ]
    pub hierarchical : Hierarchical,
    /// Specifies whether to remove the background or not from the image
    #[ arg( long, default_value = "false", verbatim_doc_comment ) ]
    pub remove_background : bool,
    /// If omitted, the background color will be calculated automatically
    #[ arg( long, value_parser = color_parser, verbatim_doc_comment ) ]
    pub background_color : Option< [ u8; 3 ] >,
    /// Specifies similarity threshold for colors to be considered as `background`
    /// Works best with the `cutout` clustering mode
    #[ arg( long, default_value = "10.0", verbatim_doc_comment ) ]
    pub background_similarity : f32,
  }

  impl Default for Config 
  {
    fn default() -> Self 
    {
      Self
      {
        color_precision : 8,
        filter_speckle : 4,
        gradient_step : 16,
        corner_threshold : 60.0,
        segment_length : 4.0,
        splice_threshold : 45.0,
        hierarchical : Default::default(),
        mode : Default::default(),
        remove_background : false,
        background_color : None,
        background_similarity : 10.0
      }
    }   
  }

  /// Hierarchical clustering
  #[ derive( Copy, Clone, Debug, clap::ValueEnum, PartialEq ) ]
  pub enum Hierarchical
  {
    /// Clusters are stacked on top of eachout
    Stacked,
    /// Clusters are cutout to remove overlap
    Cutout
  }

  impl Default for Hierarchical
  {
    fn default() -> Self 
    {
      Self::Cutout
    }   
  }

  /// Executes command with providing `args`.
  pub async fn command( args : CLIArgs )
  {
    let result = actions::clusters::action( args ).await;

    match result
    {
      Err ( error ) => println!( "{}", error ),
      Ok ( report ) => println!( "{}", report )
    }
  }
}

crate::mod_interface!
{
  own use
  { 
    command,
    Config,
    CLIArgs,
    PathSimplifyMode,
    Hierarchical
  };
}