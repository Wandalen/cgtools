//!
//! List demo in Embroidery API (command part).
//!

mod private
{
  use crate::*;
  use clap::{ Subcommand, Parser };
  use super::clusters::{ Hierarchical, Config as ColorConfig };
  use super::layers::{ Config as LayersConfig, ColorDifference };
  use commands::raster::common::color_parser;

  /// Vectorize API commands.
  ///
  /// This enum defines the available commands for vectorizing raster images,
  /// including the color and layers methods, each with their own configuration parameters.
  ///
  /// # Variants
  /// * `Color` - Vectorizes a raster image using the color method.
  ///   - `args` - Configuration parameters for the color vectorization process.
  /// * `Layers` - Vectorizes a raster image using the layers method.
  ///   - `args` - Configuration parameters for the layers vectorization process.
  #[ derive ( Debug, Subcommand ) ]
  pub enum Command
  {
    /// Color command
    Clusters ( super::clusters::CLIArgs ),
    /// Layers command
    Layers( super::layers::CLIArgs )
  }

  /// Fit modes available when creating a path.
  ///
  /// This enum defines the different modes for fitting paths during the vectorization process.
  /// Each mode determines how the vector paths are simplified and represented.
  ///
  /// # Variants
  /// * `Pixel` - Uses straight lines to trace pixels. This mode is the most basic and creates paths that follow the pixel grid closely, resulting in a more jagged appearance.
  /// * `Polygon` - Uses polygons to fit the path. This mode creates smoother paths by approximating the shapes with polygons, reducing the number of points and creating a cleaner look.
  /// * `Spline` - Uses splines to fit the path. This mode provides the smoothest paths by using spline curves, which can create very fluid and natural-looking shapes.
  #[ derive( Copy, Clone, Debug, clap::ValueEnum ) ]
  pub enum PathSimplifyMode
  {
    /// Use straight lines to trace pixels
    Pixel,
    /// Use poligons to fit the path
    Polygon,
    /// Use splines to fit the path
    Spline
  }

  impl Default for PathSimplifyMode 
  {
    fn default() -> Self 
    {
      Self::Spline
    }    
  }

  impl From< PathSimplifyMode > for visioncortex::PathSimplifyMode
  {
    fn from( value: PathSimplifyMode ) -> Self 
    {
        match value
        {
          PathSimplifyMode::Pixel => visioncortex::PathSimplifyMode::None,
          PathSimplifyMode::Polygon => visioncortex::PathSimplifyMode::Polygon,
          PathSimplifyMode::Spline => visioncortex::PathSimplifyMode::Spline
        }
    }
  }

  /// This enum defines the different vectorizer methos available for vectorization of an image.

  /// # Variants
  /// * `Clusters` - Uses the clustarization method for vectorizing an image.
  /// * `Layers` - Uses the layring method for vectorizing an image.
  #[ derive( Copy, Clone, Debug, clap::ValueEnum ) ]
  pub enum VectorizationMethods
  {
    /// Specifies the `raster vectorize color` command to be used
    Clusters,
    /// Specifies the `raster vectorize layers` command to be used
    Layers
  }

  /// Parameters for the tracer
  ///
  /// This struct contains all of the necessary parameters for configuring the vectorization process.
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
  /// * `gradient_step` - Color difference between gradient layers.
  /// * `mode` - Curve fitting mode to use when creating a path.
  /// * `corner_threshold` - Minimum momentary angle (in degrees) to be considered a corner.
  /// * `segment_length` - Perform iterative subdivide smooth until all segments are shorter than this length.
  /// * `splice_threshold` - Minimum angle displacement (in degrees) to splice a spline.
  /// * `hierarchical` - Clustering mode to use for hierarchical clustering.
  /// * `remove_background` - Removes background color from the image
  /// * `background_color` - Manually specify the background color to remove
  /// * `background_similarity` - Threshold for colors to be considered as similar to `background`
  /// * `only_chroma` - discard lightnetss when caclulating the difference between colors
  #[ derive ( Debug, Parser ) ]
  pub struct CombinedConfig
  {
    /// Specifies the vectorizer method to use. 
    /// Some parameters below belong to a specific command. Some are shared.
    /// The type of command the parameter belongs to is specified in the description of the parameter.
    #[ arg( long, value_enum, default_value_t = VectorizationMethods::Layers, verbatim_doc_comment ) ]
    pub method : VectorizationMethods,
    /// Vectorization method: clusters, layers  
    /// The amount of significant bits to use when comparing `closeness` of two colors
    #[ arg( long, default_value = "8", verbatim_doc_comment ) ]
    pub color_precision : u8,
    /// Vectorization method: clusters, layers
    /// Discard patches smaller than X px in size
    #[ arg( long, default_value = "4", verbatim_doc_comment ) ]
    pub filter_speckle : usize,
    /// Vectorization method: clusters, layers
    /// Curve fitting mode
    #[ arg( long, value_enum, default_value_t = PathSimplifyMode::Spline, verbatim_doc_comment ) ]
    pub mode : PathSimplifyMode,
    /// Vectorization method: clusters, layers  
    /// Minimum momentary angle ( in degrees ) to be considered a corner.
    #[ arg( long, default_value = "60.0", verbatim_doc_comment ) ]
    pub corner_threshold : f64,
    /// Vectorization method: clusters, layers
    /// Perform iterative subdivide smooth until all segments are shorter than this length
    #[ arg( long, default_value = "4.0", verbatim_doc_comment ) ]
    pub segment_length : f64,
    /// Vectorization method: clusters, layers
    /// Minimum angle displacement ( in degrees ) to splice a spline
    #[ arg( long, default_value = "45.0", verbatim_doc_comment ) ]
    pub splice_threshold : f64,

    /// Vectorization method: clusters
    /// Color difference between gradient layers
    #[ arg( long, default_value = "16", verbatim_doc_comment ) ]
    pub gradient_step : usize,
    /// Vectorization method: clusters
    /// Clustering mode
    #[ arg( long, value_enum, default_value_t = Hierarchical::Cutout, verbatim_doc_comment ) ]
    pub hierarchical : Hierarchical,

    /// Vectorization method: layers
    /// The number of layers determines how many unique color will be used( each color will be moved to a separate layer ).
    /// If ommited, the amount will be chosen based on image statistics.
    /// Ignored, if custom_colors are provided
    #[ arg( long, verbatim_doc_comment ) ]
    pub num_layers : Option< usize >,
    /// Vectorization method: layers
    /// Determines the threshold when comparing colors.
    /// Colors are deemed similiar, if the distance between them is less than `similiarity`
    /// Default value depends on the `color_difference` method selected
    #[ arg( long, verbatim_doc_comment ) ]
    pub similarity : Option< f32 >,
    /// Vectorization method: layers
    /// Each layer will be increased in size, using circular brush, by the amount of pixels specified
    #[ arg( long, default_value = "0", verbatim_doc_comment ) ]
    pub grow : u32,
    /// Vectorization method: layers
    /// A list of custom colors( layers ) to use for the image.
    /// If not provided, layers will be computed automatically from the image.
    #[ arg( long, value_parser = color_parser, verbatim_doc_comment ) ]
    pub custom_colors : Vec< [ u8; 3 ] >,
    /// Vectorization method: layers
    /// Each layer will only consume pixels whose color is `similiar` to the color of the layer
    #[ arg( long, default_value = "false", verbatim_doc_comment ) ]
    pub strict : bool,
    /// Vectorization method: layers
    /// Method to calculate the difference between two colors.
    #[ arg( long, value_enum, default_value_t = ColorDifference::Ciede, verbatim_doc_comment ) ]
    pub color_difference : ColorDifference,
    /// Vectorization method: clusters, layers
    /// Specifies whether to remove the background or not from the image
    #[ arg( long, default_value = "false", verbatim_doc_comment ) ]
    pub remove_background : bool,
    /// Vectorization method: clusters, layers
    /// If omitted, the background color will be calculated automatically
    #[ arg( long, value_parser = color_parser, verbatim_doc_comment ) ]
    pub background_color : Option< [ u8; 3 ] >,
    /// Vectorization method: clusters, layers
    /// Specifies similarity threshold for colors to be considered as `background`
    #[ arg( long, default_value = "10.0", verbatim_doc_comment ) ]
    pub background_similarity : f32,
    /// Vectorization method: layers
    /// Specifies whether or not to compare colors using only chromaticity value( Hue on HSL cylinder )
    #[ arg( long, default_value = "false", verbatim_doc_comment ) ]
    pub only_chroma : bool,
    /// Vectorization method: layers
    /// Specifies whether or not to merge small clusters into big ones
    #[ arg( long, default_value = "false", verbatim_doc_comment ) ]
    pub retain_speckle_detail : bool,
    /// Vectorization method: layers
    /// Specifies the minimun size of a cluster to be grown when `--grow` option is used
    #[ arg( long, default_value = "0", verbatim_doc_comment ) ]
    pub min_grow_speckle : usize,
    
  }

  impl Default for CombinedConfig
  {
    fn default() -> Self 
    {
      let method = VectorizationMethods::Layers;

      let LayersConfig 
      {
        color_precision,
        filter_speckle,
        mode,
        corner_threshold,
        segment_length,
        splice_threshold,
        num_layers,
        similarity,
        grow,
        custom_colors,
        strict,
        color_difference,
        background_color,
        remove_background,
        only_chroma,
        background_similarity,
        retain_speckle_detail,
        min_grow_speckle,
        ..
      } = LayersConfig::default();

      let ColorConfig
      {
        hierarchical,
        gradient_step,
        ..
      } = ColorConfig::default();

      Self
      {
        method,
        color_precision,
        filter_speckle,
        mode,
        corner_threshold,
        segment_length,
        splice_threshold,
        gradient_step,
        hierarchical,
        num_layers,
        similarity,
        grow,
        custom_colors,
        strict,
        color_difference,
        background_color,
        remove_background,
        only_chroma,
        background_similarity,
        retain_speckle_detail,
        min_grow_speckle
      }
    }
  }

  impl From< CombinedConfig > for ColorConfig
  {
    fn from( value : CombinedConfig ) -> Self 
    {
      let CombinedConfig
      {
        color_precision,
        filter_speckle,
        mode,
        corner_threshold,
        segment_length,
        splice_threshold,
        hierarchical,
        gradient_step,
        remove_background,
        background_color,
        background_similarity,
        ..
      } = value;  

      Self
      {
        color_precision,
        filter_speckle,
        mode,
        corner_threshold,
        segment_length,
        splice_threshold,
        hierarchical,
        gradient_step,
        remove_background,
        background_color,
        background_similarity
      }
    }
  }

  impl From< CombinedConfig > for LayersConfig
  {
    fn from( value : CombinedConfig ) -> Self 
    {
      let CombinedConfig
      {
        color_precision,
        filter_speckle,
        mode,
        corner_threshold,
        segment_length,
        splice_threshold,
        num_layers,
        similarity,
        grow,
        custom_colors,
        strict,
        color_difference,
        background_color,
        remove_background,
        only_chroma,
        background_similarity,
        retain_speckle_detail,
        min_grow_speckle,
        ..
      } = value;  

      Self
      {
        color_precision,
        filter_speckle,
        mode,
        corner_threshold,
        segment_length,
        splice_threshold,
        num_layers,
        similarity,
        grow,
        custom_colors,
        strict,
        color_difference,
        background_color,
        remove_background,
        only_chroma,
        background_similarity,
        retain_speckle_detail,
        min_grow_speckle
      }
    }
  }

  /// Executes providing `command`
  pub async fn command( command : Command )
  {
    match command
    {
      Command::Clusters( args ) =>
      {
        super::clusters::command( args ).await;
      }
      Command::Layers( args ) =>
      {
        super::layers::command( args ).await;
      },
    }
  }
}

crate::mod_interface!
{
  layer layers;
  layer clusters;

  own use 
  {
    command,
    Command,
    PathSimplifyMode,
    CombinedConfig,
    VectorizationMethods
  };
}