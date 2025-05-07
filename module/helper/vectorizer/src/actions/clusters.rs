//!
//! List files in Embroidery API (action part).
//!

mod private
{
  use crate::*;
  pub use actions::
  {
    Result,
    Error,
    common::
    {
        euclid_difference,
        read_image,
        write_svg,
        background_color,
        find_unused_color_in_image,
        should_key_image
    }
  };
  use commands::raster::vectorize::clusters::{ CLIArgs, Config, Hierarchical };
  use image::DynamicImage;
  use svg::SvgFile;
  use visioncortex::color_clusters;

  /// Represents a report for the raster to vector conversion using the color method.
  ///
  /// This struct contains details about the conversion process, including the size of the input image,
  /// whether keying was used, and the total number of clusters generated during the conversion.
  ///
  /// # Fields
  /// * `image_size` - An array containing the width and height of the input image.
  /// * `keying_used` - A boolean indicating whether keying was used during the conversion process.
  /// * `total_clusters` - The total number of clusters generated during the conversion process.
  ///
  #[ derive( Debug, Default ) ]
  pub struct Report
  {
    image_size : [ usize; 2 ],
    keying_used : bool,
    total_clusters : usize,
    /// The time taken for the conversion process in seconds.
    pub work_time : f32,
  }

  impl std::fmt::Display for Report
  {
    fn fmt( &self, f: &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result 
    {
      write!( f, "Raster vectorize clusters report:\n" )?;
      write!( f, "Working time: {}\n", self.work_time )?;
      write!( f, "Keying used: {}\n", self.keying_used )?;
      write!( f, "Total clusters used: {}\n", self.total_clusters )?;

      Ok( () )
    }
  }

  /// Executes the raster vectorize command using the color method.
  ///
  /// This asynchronous function reads a raster image from the specified input, converts it to an SVG file
  /// using the provided configuration, and saves the resulting SVG file to the specified output location.
  /// It also generates a report detailing the conversion process.
  ///
  /// # Arguments
  /// * `args` - A `CLIArgs` struct containing the input/output paths and configuration parameters.
  ///
  /// # Returns
  /// * `Result<()>` - An empty result if the operation is successful, or an error if any step fails.
  ///
  /// # Errors
  /// This function will return an error if the image reading, conversion, or SVG writing fails.
  pub async fn action( args : CLIArgs ) -> Result < Report >
  {
    let start_work_time = std::time::Instant::now();
    let img = read_image( &args.io )?;

    let mut report = Report::default();
    let svg =  convert_to_vector( img, &args.config, &mut report )?;

    // Save on disk
    write_svg( &args.io, &svg )?;

    report.work_time = start_work_time.elapsed().as_secs_f32();
    
    Ok( report )
  }

  /// Converts a raster image into an SVG file using the provided configuration.
  ///
  /// This function takes a raster image, processes it according to the provided configuration,
  /// and outputs the result as an SVG file. It also generates a report detailing the conversion process.
  ///
  /// # Arguments
  /// * `img` - The input raster image to be converted.
  /// * `config` - Configuration parameters for the vectorization process.
  /// * `report` - A mutable reference to a `Report` struct where the conversion details will be stored.
  ///
  /// # Returns
  /// * `Result<SvgFile>` - A result containing the generated SVG file or an error if the conversion fails.
  ///
  /// # Errors
  /// This function will return an error if the image reading, processing, or SVG writing fails.
  pub fn convert_to_vector( img : DynamicImage, config : &Config, report : &mut Report ) -> Result< SvgFile >
  {
    let img = img.into_rgba8();
    let ( width, height ) = ( img.width() as usize, img.height() as usize );

    report.image_size = [ width, height ];

    // Convert to vtrace image
    let mut img = visioncortex::ColorImage
    {
      pixels : img.into_vec(),
      width : width,
      height : height
    };

    let mask = 0xFF;

    let bg_color = if config.remove_background
    { 
      if config.background_color.is_some()
      {
        config.background_color
        .map( | c | visioncortex::Color::new( c[ 0 ], c[ 1 ], c[ 2 ] ) )
      }
      else 
      {
        background_color( &img, mask )
        .map( | c | visioncortex::Color::new( c[ 0 ], c[ 1 ], c[ 2 ] ) )
      }

    }
    else
    {
      None
    };

    // If an image has a lot of transparent pixels, key them out with a color not present in the image
    let key_color = 
    if Hierarchical::Stacked == config.hierarchical && bg_color.is_some()
    {
      report.keying_used = true;
      let bg_color = bg_color.unwrap();
      for y in 0..height 
      {
        for x in 0..width 
        {
          let c = img.get_pixel( x, y );
          if c.a == 0 || euclid_difference( bg_color, c ) < config.background_similarity
          {
            img.set_pixel( x, y, &bg_color );
          }
        }
      }
      bg_color
    }
    else if should_key_image( &img ) 
    {
      report.keying_used = true;
      let key_color = find_unused_color_in_image( &img )?;
      for y in 0..height 
      {
        for x in 0..width 
        {
          if img.get_pixel( x, y ).a == 0 
          {
            img.set_pixel( x, y, &key_color );
          }
        }
      }
      key_color
    } 
    else
    {
      // No keying
      visioncortex::Color::default()
    };

    // Distance between layers
    let gradient_step = config.gradient_step as i32;
    let min_cluster_area = config.filter_speckle * config.filter_speckle;

    let runner = color_clusters::Runner::new
    (
      color_clusters::RunnerConfig 
      {
        diagonal : gradient_step == 0,
        hierarchical : color_clusters::HIERARCHICAL_MAX,
        batch_size : 25600,
        // Remove clasters smaller than the filter_speckle_area
        good_min_area : min_cluster_area,
        good_max_area : ( width * height ),
        // color precision
        is_same_color_a : 8 - config.color_precision as i32,
        is_same_color_b : 1,
        deepen_diff : gradient_step,
        hollow_neighbours : 1,
        key_color,
        keying_action : match config.hierarchical
        {
          Hierarchical::Stacked => color_clusters::KeyingAction::Discard,
          Hierarchical::Cutout => color_clusters::KeyingAction::Keep
        }
      },
      img.clone()
    );

    // Create hierarchical sutrcture of clusters, according to
    // https://www.visioncortex.org/impression-docs
    let mut clusters = runner.run();

    match config.hierarchical
    {
      Hierarchical::Stacked => { },
      Hierarchical::Cutout =>
      {
        let view = clusters.view();
        let image = view.to_color_image();
        let runner = color_clusters::Runner::new
        (
          color_clusters::RunnerConfig 
          {
            diagonal: false,
            hierarchical: 64,
            batch_size: 25600,
            good_min_area: 0,
            good_max_area: ( image.width * image.height ) as usize,
            is_same_color_a: 0,
            is_same_color_b: 1,
            deepen_diff: 0,
            hollow_neighbours: 0,
            key_color,
            keying_action: color_clusters::KeyingAction::Discard,
          },
          image,
        );
        clusters = runner.run();
      }
    }

    let view = clusters.view();
    let mut svg = SvgFile::new( width, height, Some( 2 ) );

    let mut total_cluster = 0;
    for id in view.clusters_output.iter().rev()
    {
      total_cluster += 1;
      let cluster = view.get_cluster( *id );

      if Hierarchical::Cutout == config.hierarchical && config.remove_background
      {
        let bg_color = bg_color.unwrap();
        if euclid_difference( bg_color, cluster.residue_color() ) < config.background_similarity
        {
          continue;
        }
      }

      // Convert cluster to path
      let path = cluster.to_compound_path
      (
        &view, 
        false, 
        config.mode.into(), 
        config.corner_threshold.to_radians(), 
        config.segment_length, 
        10, 
        config.splice_threshold.to_radians()
      );
      // Add path to svg
      svg.add_path( path, cluster.residue_color() );
    }

    report.total_clusters = total_cluster;

    Ok( svg )
  }
}

crate::mod_interface!
{
  own use 
  {
    action,
    convert_to_vector,
    Error,
    Result,
    Report
  };
}