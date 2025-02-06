//!
//! List files in Embroidery API (action part).
//!

mod private
{
  use crate::*;  
  pub use actions::{ Error, Result };
  use actions::common::{ euclid_difference, background_color, read_image, write_svg };
  use commands::raster::vectorize::layers::CLIArgs;
  use std::collections::{ HashMap, HashSet };
  use commands::raster::vectorize::layers::{ Config, ColorDifference };
  use visioncortex::clusters::Cluster;
  use palette::
  { 
    color_difference::{ EuclideanDistance, HyAb, ImprovedCiede2000 }, white_point::A, IntoColor, Lab, Lch, LinSrgb, Srgb
  };
  use crate::svg::SvgFile;


  const LAYER_CUTOFF_THRESHOLD : f32 = 0.90;
  
  /// Structure that bundles a cluster with a useful information about the cluster
  #[ derive( Default ) ]
  pub struct ClusterPack
  {
    pub cluster : Cluster,
    pub color : LinSrgb
  }

  /// Represents a report for the raster to vector conversion using the layer method.
  ///
  /// This struct contains details about the conversion process, including the size of the input and output images,
  /// color mask used, total pixels processed, layers generated, unique colors before and after merging, 
  /// color merge iterations, and the time taken for processing.
  ///
  /// # Fields
  /// * `image_size_in` - An array containing the width and height of the input image.
  /// * `image_size_out` - An array containing the width and height of the output image.
  /// * `color_mask` - A bitmask representing the color channels used during the conversion process.
  /// * `total_pixels` - The total number of pixels processed.
  /// * `layers` - A vector of tuples representing the RGB values of the layers generated.
  /// * `unique_colors` - The number of unique colors in the input image.
  /// * `merged_unique_colors` - The number of unique colors after merging similar colors.
  /// * `color_merge_iterations` - The number of iterations performed to merge similar colors.
  /// * `work_time` - The time taken for the conversion process in seconds.
  ///
  #[ derive( Debug, Default ) ]
  pub struct Report
  {
    image_size_in : [ usize; 2 ],
    image_size_out : [ usize; 2 ],
    color_mask : u8,
    total_pixels : usize,
    layers : Vec< ( u8, u8, u8 ) >,
    unique_colors : usize,
    merged_unique_colors : usize,
    color_merge_iterations : usize,
    /// The time taken for the conversion process in seconds.
    pub work_time : f32,
  }

  impl std::fmt::Display for Report
  {
    fn fmt( &self, f: &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result 
    {
      write!( f, "Raster vectorize layers report:\n" )?;
      write!( f, "Working time: {}\n", self.work_time )?;
      write!( f, "Image size in: [ {}, {} ]\n", self.image_size_in[ 0 ], self.image_size_in[ 1 ] )?;
      write!( f, "Image size out: [ {}, {} ]\n",self.image_size_out[ 0 ], self.image_size_out[ 1 ] )?;
      write!( f, "Color mask: {:b}\n", self.color_mask )?;
      write!( f, "Total pixels used: {}\n", self.total_pixels )?;
      write!( f, "Amount of unique colors in the image: {}\n", self.unique_colors )?;
      write!( f, "Amount of unique colors in the image after merge: {}\n", self.merged_unique_colors )?;
      write!( f, "Color merge iterations: {}\n", self.color_merge_iterations )?;
      write!( f, "Amount of layers used: {}\n", self.layers.len() )?;
      write!( f, "Final colors: \n" )?;

      for col in self.layers.iter()
      {
        write!( f, "-- Color: [ {}, {}, {} ]\n", col.0, col.1, col.2 )?;
      }

      Ok( () )
    }
  }

  /// Executes the raster vectorize command using the layer method.
  ///
  /// This asynchronous function reads a raster image from the specified input, converts it to an SVG file
  /// using the provided configuration, and saves the resulting SVG file to the specified output location.
  /// It also generates a report detailing the conversion process, including the time taken for the process.
  ///
  /// # Arguments
  /// * `args` - A `CLIArgs` struct containing the input/output paths and configuration parameters.
  ///
  /// # Returns
  /// * `Result<Report>` - A result containing the report of the conversion process if successful, or an error if any step fails.
  ///
  /// # Errors
  /// This function will return an error if the image reading, conversion, or SVG writing fails.
  pub async fn action( args : CLIArgs ) -> Result< Report >
  {
    let start_work_time = std::time::Instant::now();

    let img = read_image( &args.io )?;
    let img = img.into_rgba8();
    let ( width, height ) = ( img.width() as usize, img.height() as usize );
    let img = visioncortex::ColorImage
    {
      pixels : img.into_vec(),
      width,
      height
    };

    let mut report : Report = Default::default();
    let svg = convert_to_vector( &img, &args.config, &mut report )?;

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
  pub fn convert_to_vector( img : &visioncortex::ColorImage, config : &Config, report : &mut Report ) -> Result< SvgFile >
  {
    let ( width, height ) = ( img.width, img.height );
    report.image_size_in = [ width, height ];

    // Quantization of the color
    let quant = 8 - config.color_precision;
    // Mask to apply to the color
    let mask : u8 = 0xFF << quant;
    report.color_mask = mask;

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

    let layers = get_layers( &img, quant, config, report, bg_color );
    for l in layers.iter()
    {
      let c = Srgb::< u8 >::from_linear( ( *l ).into_color() );
      report.layers.push( c.into_components() );
    }

    // For each color in the image, find the color in the `freq` list, that has the smallest `distance` to the current color.
    // Distance can be computed in a low of different ways.
    let mut ucolor_map = HashMap::new();
    for c in img.iter()
    {
      if c.a == 0 { continue; }
      if bg_color.is_some_and( | bg_color | euclid_difference( bg_color, c ) < config.background_similarity )
      {
        ucolor_map.insert( [ c.r, c.g, c.b ], -1 );
        continue;
      }

      let c = visioncortex::Color::new( c.r & mask, c.g & mask, c.b & mask );
      // Convert from Srgb to Linear space
      let lin_c = Srgb::new( c.r, c.g, c.b ).into_linear::< f32 >();
      // Convert form Linear to Lab space
      let mut lab_c : Lab = lin_c.into_color();

      let mut lcc : Lch = lab_c.into_color();

      if config.only_chroma
      {
        lcc.chroma = 128.0;
        lcc.l = 50.0;
        lab_c = lcc.into_color();
      }

      // Find the color with the smallest distance
      let ( id, lc ) = layers.iter().enumerate().min_by( | ( _, &a ), ( _, &b ) |
      {
        let mut lab_a : Lab = a.into_color();
        let mut lab_b : Lab = b.into_color();

        if config.only_chroma
        {
          let mut lca : Lch = lab_a.into_color();
          let mut lcb : Lch = lab_b.into_color();

          lca.chroma = 128.0;
          lca.l = 50.0;

          lcb.chroma = 128.0;
          lcb.l = 50.0;

          lab_a = lca.into_color();
          lab_b = lcb.into_color();
        }

        let d_a = color_difference( lab_a, lab_c, config );
        let d_b = color_difference( lab_b, lab_c, config );
        
        d_a.partial_cmp( &d_b ).unwrap()
      }).unwrap();

      let mut id = id as i32;
      if config.strict
      {
        let mut lab_lc : Lab = ( *lc ).into_color();

        if config.only_chroma
        {
          lab_lc.l = lab_c.l;
        }

        if color_difference( lab_c, lab_lc, config ) > config.get_similarity()
        {
          id = -1;
        }
      }

      // Map the current color to the `id` that corresponds to the color in `layers` list with the smallest distance
      ucolor_map.insert( [ c.r, c.g, c.b ], id );
    }

    let grow_size = config.grow * 2;
    let min_cluster_area = config.filter_speckle * config.filter_speckle;
    let min_grow_cluster_area = config.min_grow_speckle * config.min_grow_speckle;
    let width = width + grow_size as usize;
    let height = height + grow_size as usize;

    report.image_size_out = [ width, height ];
    let mut svg = SvgFile::new( width, height, Some( 2 ) );
    let mut big_cluster_map = visioncortex::MonoImageBig::new_w_h( width, height );
    let mut all_clusters = vec![ ClusterPack::default() ];
    let mut cluster_index_offset = 1;

    for ( i, l ) in layers.into_iter().enumerate()
    {
      // For the current layer, use all pixels, whose color is the most similiar to the color of the layer
      let img = img.to_binary_image( | c | 
      { 
        if c.a == 0 { return false; }
        let id = *ucolor_map.get( &[ c.r & mask, c.g & mask, c.b & mask ] ).unwrap();
        i as i32 == id
      });

      let mut clusters = img.to_clusters_big( false, &mut big_cluster_map, cluster_index_offset );
      cluster_index_offset += clusters.clusters.len() as u32;
      let mut clusters : Vec< ClusterPack > = clusters
      .into_iter()
      .map( | c | ClusterPack{ cluster : c, color : l } )
      .collect();
      all_clusters.append( &mut clusters );
    }


    if !config.retain_speckle_detail
    {
      let mut merged = true;

      while merged
      {
        merged = false;
        // Merge small clusters into big oness
        for id in 0..all_clusters.len()
        {
          let size = all_clusters[ id ].cluster.size();
          if size != 0 && size < min_cluster_area
          {
            merged = true;
            let mut neighbours = HashMap::new();
            for point in all_clusters[ id ].cluster.points.iter()
            {
              if point.y > 0 
              { 
                let up =  big_cluster_map.get_pixel( point.x as usize, point.y as usize - 1 );
                neighbours.entry( up ).and_modify( | v | *v += 1 ).or_insert( 1 );
              }

              if point.y  < height as i32 - 1 
              { 
                let down =  big_cluster_map.get_pixel( point.x as usize, point.y as usize + 1 );
                neighbours.entry( down ).and_modify( | v | *v += 1 ).or_insert( 1 );
              }

              if point.x > 0
              {
                let left =  big_cluster_map.get_pixel( point.x as usize - 1, point.y as usize );
                neighbours.entry( left ).and_modify( | v | *v += 1 ).or_insert( 1 );
              }

              if point.x < width as i32 - 1 
              {
                let right =  big_cluster_map.get_pixel( point.x as usize + 1, point.y as usize );
                neighbours.entry( right ).and_modify( | v | *v += 1 ).or_insert( 1 );
              }
            }

            let mut neighbours_stats : Vec< ( u32, u32 ) > = neighbours
            .into_iter()
            .filter( | ( cluster, _ ) |
            {
              *cluster != id as u32
            })
            .collect();

          
            {
              let col = all_clusters[ id ].color;
              let lab_cur : Lab = col.into_color();

              neighbours_stats.sort_by( | a, b | 
              {
                // if a.1.abs_diff( b.1 ) > 100 //a.1 == b.1
                // {
                //   let a = all_clusters[ a.0 as usize ].color;
                //   let b = all_clusters[ b.0 as usize ].color;

                //   let lab_a : Lab = a.into_color();
                //   let lab_b : Lab = b.into_color();

                //   let d_a = lab_cur.improved_difference( lab_a );
                //   let d_b = lab_cur.improved_difference( lab_b );
                  
                //   d_a.partial_cmp( &d_b ).unwrap()
                // } 
                // else 
                {
                  a.1.cmp( &b.1 )    
                }

              });

              if let Some( ( closest, _ ) ) = neighbours_stats.last()
              {
                for point in all_clusters[ id ].cluster.points.iter()
                {
                  big_cluster_map.set_pixel( point.x as usize, point.y as usize, *closest );
                }

                let mut drain = std::mem::take( &mut all_clusters[ id ].cluster.points );
                all_clusters[ *closest as usize ].cluster.points.append( &mut drain );
                let rect = all_clusters[ id ].cluster.rect;
                all_clusters[ *closest as usize ].cluster.rect.merge( rect );
              }
              else 
              {
                std::mem::take( &mut all_clusters[ id ].cluster.points );   
              }
            }
 
            // if let Some( ( to , _ ) ) = neighbours_stats.first().copied()
            // {
            //   for point in all_clusters[ id ].cluster.points.iter()
            //   {
            //     big_cluster_map.set_pixel( point.x as usize, point.y as usize, to );
            //   }

            //   let mut drain = std::mem::take( &mut all_clusters[ id ].cluster.points );
            //   all_clusters[ to as usize ].cluster.points.append( &mut drain );
            //   let rect = all_clusters[ id ].cluster.rect;
            //   all_clusters[ to as usize ].cluster.rect.merge( rect );
            // }
          }
        }
      }
    }
  
    
    if config.grow > 0
    {
      let mut grown_clusters = Vec::new();
      let mut non_grown_clusters = Vec::new();

      // Grow clusters that are bigger than the min_cluster_area
      for ( i, pack ) in all_clusters.iter().enumerate()
      {
        let size = pack.cluster.size();
        if size >= min_grow_cluster_area
        {
          let mut img = pack.cluster.to_binary_image();
          img = img.stroke( grow_size );
          let mut new_clusters = img.to_clusters( false );

          for c in new_clusters.clusters.iter_mut()
          {
            c.offset( visioncortex::PointI32 { x: pack.cluster.rect.left, y: pack.cluster.rect.top });
          }

          grown_clusters.push( ( new_clusters, pack.color ) );
        }
        else 
        {
          non_grown_clusters.push( i );
        }
      }

      // Convert clusters to svg paths
      for ( clusters, color ) in grown_clusters.iter()
      {
        for cluster in clusters.iter()
        {
          if cluster.size() >= min_cluster_area 
          {
            let path = trace( &cluster, &config );
            svg.add_path( path, linear_to_vision( *color ) );
          }
        }
      }

      for id in non_grown_clusters
      {
        let pack = &all_clusters[ id ];
        if pack.cluster.size() >= min_cluster_area 
        {
          let path = trace( &pack.cluster, &config );
          svg.add_path( path, linear_to_vision( pack.color ) );
        }
      }
    }
    else 
    {
      // Convert cluster, that are bigger than the min_cluster_are, to the svg paths
      for pack in all_clusters.iter()
      {
        if pack.cluster.size() >= min_cluster_area 
        {
          let path = trace( &pack.cluster, &config );
          svg.add_path( path, linear_to_vision( pack.color ) );
        }
      }
    }

    Ok( svg )
  }

  /// A shortcut to convert a cluster into a path
  pub fn trace
  ( 
    cluster : &visioncortex::clusters::Cluster, 
    config : &Config 
  ) -> visioncortex::CompoundPath
  {
    cluster.to_compound_path_mine
    (
      config.mode.into(), 
      config.corner_threshold.to_radians(), 
      config.segment_length, 
      10, 
      config.splice_threshold.to_radians()
    )
  }

  /// Converts color from linear color space into Srgb inside the visioncrotex::Color structure
  pub fn linear_to_vision( color : LinSrgb ) -> visioncortex::Color
  {
    let vis_l = Srgb::from_linear( color.into_color() );
    let vis_l = visioncortex::Color::new( vis_l.red, vis_l.green, vis_l.blue );
    vis_l
  }

  /// `colors_stats` is an array of `( color_sum, color_count )`.
  /// `color_sum` is the sum of `color_count` amount of colors, so you can get the average color by just dividing color_sum / color_count.
  /// `reduce_colors` takes the information and for each entry calculates the similiarity of the average color. If similiarity is smaller
  /// than the provided threshold, then colors are combined by adding `color_sum` and `color_count` of each entry respectively.
  /// Several passes are made, each using averages that are produced by the previous pass. 
  /// The loop terminates when no colors can be combined anymore.
  pub fn reduce_colors( mut color_stats : Vec< ( LinSrgb, f32 ) >, config : &Config, report : &mut Report ) -> Vec< ( LinSrgb, f32 ) >
  {
    let mut len_before = color_stats.len();
    let mut len_after = 0;
    let threshold = config.get_similarity();

    color_stats.sort_unstable_by( | k1, k2 | k2.1.partial_cmp( &k1.1 ).unwrap() );
    let mut iter_count = 0;

    while len_before != len_after
    {
      iter_count += 1;
      len_before = color_stats.len();
      // Keep track of colors that have already been seen
      let mut seen : HashSet< usize > = HashSet::new();
      let mut merged_colors = Vec::new();

      for ( id1, &c1 ) in color_stats.iter().enumerate()
      {
        // List of similiar colors
        let mut sim = Vec::new();

        // Average color
        let lin_a = c1.0 / c1.1;
        if !seen.insert( id1 ) { continue; }
        
        let lab_a : Lab = lin_a.into_color();

        sim.push( c1 );

        for ( id2, &c2 ) in color_stats.iter().enumerate()
        {
          // Average color
          let lin_b = c2.0 / c2.1;
          if !seen.contains( &id2 )
          {
            let lab_b : Lab = lin_b.into_color();

            if color_difference( lab_a, lab_b, config ) <= threshold
            {
              sim.push( c2 );
              seen.insert( id2 );
            }
          }
        }
        
        // total_color += color, total_count += count
        let res = sim.into_iter().reduce( | acc, e | ( acc.0 + e.0, acc.1 + e.1 ) ).unwrap();

        merged_colors.push( res );
      }

      len_after = merged_colors.len();
      color_stats = merged_colors;
    } 

    report.color_merge_iterations = iter_count;

    color_stats
  }

  /// Extracts layers from a raster image based on color frequency and similarity.
  ///
  /// This function processes a raster image to identify and group colors into layers. It builds a frequency map of colors,
  /// merges similar colors, and generates a list of unique colors (layers) to be used for vectorization. The function also
  /// updates the provided report with details about the process, including the total number of pixels, unique colors before
  /// and after merging, and the number of color merge iterations.
  ///
  /// # Arguments
  /// * `img` - A reference to a `visioncortex::ColorImage` representing the input raster image.
  /// * `mask` - A bitmask used to reduce the number of unique colors by masking color channels.
  /// * `config` - A reference to a `Config` struct containing the vectorization configuration parameters.
  /// * `report` - A mutable reference to a `Report` struct to be updated with details about the layer extraction process.
  ///
  /// # Returns
  /// * `Vec<LinSrgb>` - A vector of unique colors (layers) represented in linear RGB space.
  pub fn get_layers
  ( 
    img : &visioncortex::ColorImage, 
    quant : u8,
    config : &Config, 
    report : &mut Report, 
    bg_color : Option< visioncortex::Color >
  ) -> Vec< LinSrgb >
  {
    let mask = 0xFF << quant;
    let mut total_pixels = 0;
    // Build a map of frequencies for each color
    let mut freq : HashMap< [ u8; 3 ], i32 > = HashMap::new();
    for c in img.iter()
    {
      if c.a == 0 { continue; }
      if bg_color.is_some_and( | bg_color | euclid_difference( bg_color, c ) < config.background_similarity )
      {
        continue;
      }

      total_pixels += 1;
      freq.entry( [ c.r & mask, c.g & mask, c.b & mask ] )
      .and_modify( | count | *count += 1 )
      .or_insert( 1 );
    }

    report.total_pixels = total_pixels;
    report.unique_colors = freq.len();

    if config.custom_colors.len() == 0
    {
      // Map each color to linear space, and store total sum of each color
      let color_stats : Vec< ( LinSrgb, f32 ) > = freq
      .into_iter()
      .map( | ( col, count ) |
      {
        let count = count as f32;

        let col =  Srgb::from( col ).into_linear::< f32 >();
        ( col * count,  count )
      })
      .collect();

      // Merge similiar colors with each other
      let mut merged_colors = reduce_colors( color_stats, config, report );

      report.merged_unique_colors = merged_colors.len();
      // Sort by the amount of each color in descending order
      merged_colors.sort_by( | k1, k2 | k2.1.partial_cmp( &k1.1 ).unwrap() );

      let layers : Vec< LinSrgb > = 
      if let Some( num ) = config.num_layers
      {
        merged_colors.into_iter()
        .take( num )
        .map( | ( col, count ) | col / count )
        .collect()
      }
      else
      {
        let mut count_pixels = 0.0;
        merged_colors.into_iter()
        .take_while( | ( _, count ) |
        {
          let r = ( count_pixels / total_pixels as f32 ) < LAYER_CUTOFF_THRESHOLD;
          count_pixels += count;
          r
        })
        .map( | ( col, count ) | col / count )
        .collect()
      };

      layers
    }
    else 
    {
      let layers : Vec< LinSrgb >  = config.custom_colors
      .iter()  
      .map( | c | Srgb::from( *c ).into_linear::< f32 >() )
      .collect();

      layers
    }
  }

  #[ inline ]
  pub fn color_difference(c1 : Lab, c2 : Lab, config : &Config ) -> f32
  {
    match config.color_difference
    {
      ColorDifference::Hybrid => c1.hybrid_distance( c2 ),
      ColorDifference::Ciede => c1.improved_difference( c2 )
    }
  } 
}


crate::mod_interface!
{
  own use
  {
    action,
    convert_to_vector,
    Result,
    Error,
    Report
  };
}