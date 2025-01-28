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
  /// including color precision, number of layers, color similarity threshold, and other settings.
  ///
  /// # Fields
  /// * `color_precision` - The amount of significant bits to use when reading colors. Up to 8.
  ///   This parameter reduces the number of unique colors in the image by shifting the color bits.
  ///   Higher values retain more color detail, while lower values reduce the color palette.
  /// * `num_layers` - The number of layers determines how many unique colors will be used (each color will be moved to a separate layer).
  ///   If omitted, the amount will be chosen based on image statistics. Ignored if custom_colors are provided.
  /// * `similiarity` - Determines the threshold when comparing colors. Colors are deemed similar if the distance between them is less than `similiarity`.
  /// * `filter_speckle` - Discard patches smaller than X pixels in size.
  /// * `grow` - Each layer will be increased in size, using a circular brush, by the amount of pixels specified.
  /// * `custom_colors` - A list of custom colors (layers) to use for the image. If not provided, layers will be computed automatically from the image.
  /// * `strict` - Each layer will only consume pixels whose color is `similiar` to the color of the layer
  /// * `color_difference` - Method to use when calculating the difference between two colors.
  /// * `mode` - Curve fitting mode to use when creating a path.
  /// * `corner_threshold` - Minimum momentary angle (in degrees) to be considered a corner.
  /// * `segment_length` - Perform iterative subdivide smooth until all segments are shorter than this length.
  /// * `splice_threshold` - Minimum angle displacement (in degrees) to splice a spline.
  /// * `remove_background` - Removes background color from the image
  /// * `background_color` - Manually specify the background color to remove
  /// * `only_chroma` - discard lightnetss when caclulating the difference between colors
  #[ derive ( Debug, Parser ) ]
  pub struct Config
  {
    /// The amount of significant bits to use when reading colors. Up to 8.
    /// This parameter reduces the number of unique colors in the image by shifting the color bits.
    /// Higher values retain more color detail, while lower values reduce the color palette.
    #[ arg( long, short = 'p', default_value = "8", verbatim_doc_comment ) ]
    pub color_precision : u8,
    /// The number of layers determines how many unique color will be used( each color will be moved to a separate layer ).
    /// If ommited, the amount will be chosen based on image statistics.
    /// Ignored, if custom_colors are provided
    #[ arg( long, short = 'l', verbatim_doc_comment ) ]
    pub num_layers : Option< usize >,
    /// Determines the threshold when comparing colors.
    /// Colors are deemed similiar, if the distance between them is less than `similiarity`
    /// Default value depends on the `color_difference` method selected
    #[ arg( long, short = 's', verbatim_doc_comment ) ]
    pub similarity : Option< f32 >,
    /// Discard patches smaller than X px in size
    #[ arg( long, short, default_value = "4", verbatim_doc_comment ) ]
    pub filter_speckle : usize,
    /// Each layer will be increased in size, using circular brush, by the amount of pixels specified
    #[ arg( long, short, default_value = "0", verbatim_doc_comment ) ]
    pub grow : u32,
    /// A list of custom colors( layers ) to use for the image.
    /// If not provided, layers will be computed automatically from the image.
    #[ arg( long, short, value_parser = color_parser, verbatim_doc_comment ) ]
    pub custom_colors : Vec< [ u8; 3 ] >,
    /// Each layer will only consume pixels whose color is `similiar` to the color of the layer
    #[ arg( long, default_value = "false", verbatim_doc_comment ) ]
    pub strict : bool,
    /// Method to calculate the difference between two colors.
    #[ arg( long, value_enum, default_value_t = ColorDifference::Ciede, verbatim_doc_comment ) ]
    pub color_difference : ColorDifference,
    /// Curve fitting mode
    #[ arg( long, short, value_enum, default_value_t = PathSimplifyMode::Spline, verbatim_doc_comment ) ]
    pub mode : PathSimplifyMode,
    /// Minimum momentary angle ( in degrees ) to be considered a corner.
    #[ arg( long, default_value = "60.0", verbatim_doc_comment ) ]
    pub corner_threshold : f64,
    /// Perform iterative subdivide smooth until all segments are shorter than this length
    #[ arg( long, default_value = "4.0", verbatim_doc_comment ) ]
    pub segment_length : f64,
    /// Minimum angle displacement ( in degrees ) to splice a spline
    #[ arg( long, default_value = "45.0", verbatim_doc_comment ) ]
    pub splice_threshold : f64,

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
    /// Specifies whether or not to compare colors using only chromaticity value( Hue on HSL cylinder )
    /// Recommended to be used with "Ciede" color difference
    #[ arg( long, default_value = "false", verbatim_doc_comment ) ]
    pub only_chroma : bool,
    /// Specifies whether or not to merge small clusters into big ones
    #[ arg( long, default_value = "false", verbatim_doc_comment ) ]
    pub retain_speckle_detail : bool,
    /// Specifies the minimun size of a cluster to be grown when `--grow` option is used
    #[ arg( long, default_value = "0", verbatim_doc_comment ) ]
    pub min_grow_speckle : usize,
  }

  impl Config 
  {
    /// If `similarity is not set, returns the default value based on the `color_difference` method.
    /// Otherwise return the set value.
    pub fn get_similarity( &self ) -> f32
    {
      if let Some( s ) = self.similarity
      {
        s
      }
      else 
      {
        match self.color_difference  
        {
          ColorDifference::Hybrid => 9.5,
          ColorDifference::Ciede => 3.5
        }   
      }
    }   
  }

  impl Default for Config
  {
    fn default() -> Self 
    {
      Self
      {
        color_precision : 8,
        num_layers : None,
        similarity : None,
        filter_speckle : 4,
        grow : 0,
        custom_colors : Vec::new(),
        strict : false,
        corner_threshold : 60.0,
        segment_length : 4.0,
        splice_threshold : 45.0,
        mode : Default::default(),
        color_difference : Default::default(),
        background_color : None,
        remove_background : false,
        only_chroma : false,
        background_similarity : 10.0,
        retain_speckle_detail : false,
        min_grow_speckle : 0
      }
    }   
  }

  /// Available methods for calculating color difference between two colors
  #[ derive( Copy, Clone, Debug, clap::ValueEnum ) ]
  pub enum ColorDifference
  {
    /// Use improved CIEDE2000 Delta E color difference. Great for small differences
    /// Default similarity: 3.5
    Ciede,
    /// Use HyAB color difference. Great for large color differences 
    /// Default similarity: 9.5
    Hybrid
  }

  impl Default for ColorDifference
  {
    fn default() -> Self 
    {
      Self::Ciede
    }
  }
  
  /// Executes command with providing `args`.
  /// Prints result of operation into standard output
  pub async fn command( args : CLIArgs )
  {
    let result = actions::layers::action( args ).await;

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
    ColorDifference
  };
}