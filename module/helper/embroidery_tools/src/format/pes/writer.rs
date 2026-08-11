//! 
//! # PES format writer.
//! Original implementation refers to <https://github.com/EmbroidePy/pyembroidery/blob/main/pyembroidery/PesWriter.py>
//! 

mod private
{
  use crate::{ embroidery_file, error, format, stitch_instruction, thread };
  use embroidery_file::EmbroideryFile;
  use error::EmbroideryError;
  use format::{ pec, pes::PESVersion };
  use thread::{ Color, Thread };
  use stitch_instruction::Instruction;
  use std::io::{ Seek, SeekFrom, Write };
  use byteorder::{ WriteBytesExt as _, LE };

  /// Writes PES format into `writer`
  /// # Errors
  /// Propagates any error returned by the version-specific writer.
  #[ inline ]
  pub fn write< W >( emb : &mut EmbroideryFile, writer : &mut W, version : PESVersion )
  -> Result< (), EmbroideryError >
  where
    W : Write + Seek
  {
    emb.fix_color_count();
    emb.interpolate_stop_as_duplicate_color();
    
    match version
    {
      PESVersion::V1 => write_version1( emb, writer ),
      PESVersion::V6 => write_version6( emb, writer ),
    }
  }

  /// Writes PES version 1 into `writer`
  fn write_version1< W >( emb : &mut EmbroideryFile, writer : &mut W )
  -> Result< (), EmbroideryError >
  where
    W : Write + Seek
  {
    writer.write_all( "#PES0001".as_bytes() )?;
    let extends = emb.bounds();
    let cx = ( extends.2 + extends.0 ) / 2;
    let cy = ( extends.3 + extends.1 ) / 2;
    // these are bounding cooridantes of the design
    let left = extends.0 - cx;
    let top = extends.1 - cy;
    let right = extends.2 - cx;
    let bottom = extends.3 - cy;
    let pec_block_placeholder = writer.stream_position()?;
    writer.write_u32::< LE >( 0 )?; // placeholder

    if emb.stitches().is_empty()
    {
      write_header_version1( writer, 0 )?;
      // 0000 0000 means no more sections
      writer.write_u16::< LE >( 0x0000 )?;
      writer.write_u16::< LE >( 0x0000 )?;
    }
    else
    {
      write_header_version1( writer, 1 )?;
      // ffff 0000 means more sections
      writer.write_u16::< LE >( 0xFFFF )?;
      writer.write_u16::< LE >( 0x0000 )?;

      let threads = pec::pec_threads();
      _ = write_pes_block( emb, writer, &threads, DesignBounds { left, top, right, bottom, cx, cy } )?;
    }

    let current_position = writer.stream_position()?;
    writer.seek( SeekFrom::Start( pec_block_placeholder ) )?;
    // Stream position is a `u64`; PES's placeholder field is `u32`. Not provably bounded
    // by any invariant (a pathologically large output stream could exceed it), so this
    // is reported as a real error instead of silently truncated.
    let current_position_u32 = u32::try_from( current_position )
    .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "PES stream position exceeds u32 range" ) )?;
    writer.write_u32::< LE >( current_position_u32 )?;
    writer.seek( SeekFrom::Start( current_position ) )?;

    _ = pec::write_content( emb, writer )?;
    Ok( () )
  }

  fn write_header_version1< W >( writer : &mut W, distinct_block_objects : u16 )
  -> Result< (), EmbroideryError >
  where
    W : Write + Seek
  {
    writer.write_u16::< LE >( 0x01 )?; // scale to fit
    writer.write_u16::< LE >( 0x01 )?; // 0 = 100x100, 1 = 130x180 hoop
    writer.write_u16::< LE >( distinct_block_objects )?;
    
    Ok( () )
  }

  fn write_version6< W >( emb : &mut EmbroideryFile, writer : &mut W )
  -> Result< (), EmbroideryError >
  where
    W : Write + Seek
  {
    let signature = "#PES0060";
    writer.write_all( signature.as_bytes() )?;
    let extends = emb.bounds();
    let cx = ( extends.2 + extends.0 ) / 2;
    let cy = ( extends.3 + extends.1 ) / 2;

    let left = extends.0 - cx;
    let top = extends.1 - cy;
    let right = extends.2 - cx;
    let bottom = extends.3 - cy;

    let pec_block_placeholder = writer.stream_position()?;
    writer.write_u32::< LE >( 0 )?;

    if emb.stitches().is_empty()
    {
      write_header_version6( emb, writer, 0 )?;
      writer.write_u16::< LE >( 0x0000 )?;
      writer.write_u16::< LE >( 0x0000 )?;
    }
    else
    {
      write_header_version6( emb, writer, 1 )?;
      writer.write_u16::< LE >( 0xFFFF )?;
      writer.write_u16::< LE >( 0x0000 )?;
      let log = write_pes_block( emb, writer, emb.threads(), DesignBounds { left, top, right, bottom, cx, cy } )?;
      writer.write_u32::< LE >( 0 )?;
      writer.write_u32::< LE >( 0 )?;
      for i in 0..log.len()
      {
        // `log.len()` is bounded by the number of stitch segments in the design —
        // realistically far below `u32::MAX` — but not by a type-level invariant,
        // so an out-of-range index is reported rather than silently truncated.
        let i_u32 = u32::try_from( i )
        .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "segment index exceeds u32 range" ) )?;
        writer.write_u32::< LE >( i_u32 )?;
        writer.write_u32::< LE >( 0 )?;
      }
    }

    let current_pos = writer.stream_position()?;
    writer.seek( SeekFrom::Start( pec_block_placeholder ) )?;
    // See the analogous conversion in `write_version1`: stream position is `u64`,
    // the placeholder field is `u32`, and this is not provably bounded.
    let current_pos_u32 = u32::try_from( current_pos )
    .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "PES stream position exceeds u32 range" ) )?;
    writer.write_u32::< LE >( current_pos_u32 )?;
    writer.seek( SeekFrom::Start( current_pos ) )?;
    let color_info = pec::write_content( emb, writer )?;
    let rgb_list : Vec< _ > = emb.threads().iter().map( | v | v.color ).collect();
    write_pes_addendum( writer, &color_info, &rgb_list )?; // is it really necessary?
    writer.write_u16::< LE >( 0x0000 )?;

    Ok( () )
  }

  fn write_header_version6< W >
  (
    emb : &EmbroideryFile,
    writer : &mut W,
    distinct_block_objects : u16,
  )
  -> Result< (), EmbroideryError >
  where
    W : Write + Seek
  {
    // Specs: https://github.com/frno7/libpes/wiki/PES-header-section#version-6-header-section
    writer.write_u16::< LE >( 0x01 )?;
    writer.write_all( b"02" )?;

    write_pes_string8( writer, emb.get_metadata().get_name().unwrap_or_default() )?;
    write_pes_string8( writer, emb.get_metadata().get_text( "category" ).unwrap_or_default() )?;
    write_pes_string8( writer, emb.get_metadata().get_text( "author" ).unwrap_or_default() )?;
    write_pes_string8( writer, emb.get_metadata().get_text( "keywords" ).unwrap_or_default() )?;
    write_pes_string8( writer, emb.get_metadata().get_text( "comments" ).unwrap_or_default() )?;
    
    writer.write_u16::< LE >( 0 )?;    // OptimizeHoopChange = False
    writer.write_u16::< LE >( 0 )?;    // Design Page Is Custom = False
    writer.write_u16::< LE >( 0x64 )?; // Hoop Width
    writer.write_u16::< LE >( 0x64 )?; // Hoop Height
    writer.write_u16::< LE >( 0 )?;    // Use Existing Design Area = False
    writer.write_u16::< LE >( 0xC8 )?; // designWidth
    writer.write_u16::< LE >( 0xC8 )?; // designHeight
    
    writer.write_u16::< LE >( 0x64 )?; // designPageSectionWidth
    writer.write_u16::< LE >( 0x64 )?; // designPageSectionHeight
    writer.write_u16::< LE >( 0x64 )?; // p6 # 100
    writer.write_u16::< LE >( 0x07 )?; // designPageBackgroundColor
    writer.write_u16::< LE >( 0x13 )?; // designPageForegroundColor
    writer.write_u16::< LE >( 0x01 )?; // ShowGrid
    writer.write_u16::< LE >( 0x01 )?; // WithAxes
    writer.write_u16::< LE >( 0x00 )?; // SnapToGrid
    writer.write_u16::< LE >( 100 )?;  // GridInterval
    writer.write_u16::< LE >( 0x01 )?; // p9 curves
    writer.write_u16::< LE >( 0x00 )?; // OptimizeEntryExitPoints
    writer.write_u8( 0 )?;             // fromImageStringLength

    writer.write_f32::< LE >( 1.0 )?;
    writer.write_f32::< LE >( 0.0 )?;
    writer.write_f32::< LE >( 0.0 )?;
    writer.write_f32::< LE >( 1.0 )?;
    writer.write_f32::< LE >( 0.0 )?;
    writer.write_f32::< LE >( 0.0 )?;
    writer.write_u16::< LE >( 0 )?;    // number of ProgrammableFillPatterns
    writer.write_u16::< LE >( 0 )?;    // number of MotifPatterns
    writer.write_u16::< LE >( 0 )?;    // feather pattern count
    
    // Bounded by PES/PEC's own format limits (thread palettes never realistically
    // approach `u16::MAX` entries), but not by a type-level invariant, so this is
    // reported as a real error instead of silently truncated.
    let thread_count = u16::try_from( emb.threads().len() )
    .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "thread count exceeds PES's u16 encoding range" ) )?;
    writer.write_u16::< LE >( thread_count )?; // number of colors
    for thread in emb.threads()
    {
      write_pes_thread( writer, thread )?;
    }

    writer.write_u16::< LE >( distinct_block_objects )?; // number of distinct blocks

    Ok( () )
  }

  fn write_pes_thread< W >( writer : &mut W, thread : &Thread ) -> Result< (), EmbroideryError >
  where
    W : Write
  {
    // Specs: https://github.com/frno7/libpes/wiki/PES-header-section#color-subsection

    write_pes_string8( writer, &thread.catalog_number )?;
    writer.write_u8( thread.color.r )?;
    writer.write_u8( thread.color.g )?;
    writer.write_u8( thread.color.b )?;
    writer.write_u8( 0 )?;
    writer.write_u32::< LE >( 0xA )?;
    write_pes_string8( writer, &thread.description )?;
    write_pes_string8( writer, &thread.brand )?;
    write_pes_string8( writer, &thread.chart )?;

    Ok( () )
  }

  /// Design bounding box together with its center, used when laying out PES sections
  #[ derive( Debug, Clone, Copy ) ]
  struct DesignBounds
  {
    left : i32,
    top : i32,
    right : i32,
    bottom : i32,
    cx : i32,
    cy : i32,
  }

  /// This function writes CEmbOne and CEmbSewSeg sections of PES file
  fn write_pes_block< W >
  (
    emb : &EmbroideryFile,
    writer : &mut W,
    threads : &[ Thread ],
    bounds : DesignBounds
  )
  -> Result< Vec< ( u16, usize ) >, EmbroideryError >
  where
    W : Write + Seek
  {
    if emb.stitches().is_empty()
    {
      return Ok( vec![] );
    }

    write_pes_string16( writer, "CEmbOne" )?;
    let placeholder = write_pes_sewseg_header( writer, bounds )?;
    writer.write_u16::< LE >( 0xFFFF )?;
    writer.write_u16::< LE >( 0x0000 )?; // FFFF0000 means more blocks exist

    write_pes_string16( writer, "CSewSeg" )?;
    let ( sections, colorlog ) = write_pes_embsewseg_segments( emb, writer, threads, bounds )?;

    let current_pos = writer.stream_position()?;
    writer.seek( SeekFrom::Start( placeholder ) )?;
    writer.write_u16::< LE >( sections )?;
    writer.seek( SeekFrom::Start( current_pos ) )?;

    writer.write_u16::< LE >( 0x0000 )?;
    writer.write_u16::< LE >( 0x0000 )?;

    Ok( colorlog )
  }

  /// Writes SewSeg header
  fn write_pes_sewseg_header< W >( writer : &mut W, bounds : DesignBounds )
  -> Result< u64, EmbroideryError >
  where
    W : Write + Seek
  {
    // Specs https://github.com/frno7/libpes/wiki/PES-CSewSeg-section#header
    let width = bounds.right - bounds.left;
    let height = bounds.bottom - bounds.top;
    let hoop_height : f32 = 1800.0;
    let hoop_width : f32 = 1300.0;

    writer.write_u16::< LE >( 0 )?;
    writer.write_u16::< LE >( 0 )?;
    writer.write_u16::< LE >( 0 )?;
    writer.write_u16::< LE >( 0 )?;
    writer.write_u16::< LE >( 0 )?;
    writer.write_u16::< LE >( 0 )?;
    writer.write_u16::< LE >( 0 )?;
    writer.write_u16::< LE >( 0 )?;

    let mut trans_x : f32 = 350.0;
    // `height`/`width` are design bounds derived from stitch coordinates; real
    // embroidery designs stay well under 2^24 units, so this never loses meaningful
    // precision even though `i32 -> f32` is not lossless in the general case.
    let mut trans_y : f32 = 100.0 + height as f32;
    trans_x += hoop_width / 2.0;
    trans_y += hoop_height / 2.0;
    // Same bound as `trans_y`'s initializer above: `width`/`height` are design bounds
    // that never realistically approach 2^24 units.
    let neg_width = -width as f32;
    let neg_height = -height as f32;
    trans_x += neg_width / 2.0;
    trans_y += neg_height / 2.0;

    writer.write_f32::< LE >( 1.0 )?;
    writer.write_f32::< LE >( 0.0 )?;
    writer.write_f32::< LE >( 0.0 )?;
    writer.write_f32::< LE >( 1.0 )?;
    writer.write_f32::< LE >( trans_x )?;
    writer.write_f32::< LE >( trans_y )?;

    writer.write_u16::< LE >( 1 )?;
    writer.write_u16::< LE >( 0 )?;
    writer.write_u16::< LE >( 0 )?;
    // Not provably bounded (a design wider/taller than PES's `u16` field is possible
    // in principle), so this is reported as a real error instead of corrupted silently.
    let width_u16 = u16::try_from( width )
    .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "design width exceeds PES's u16 encoding range" ) )?;
    let height_u16 = u16::try_from( height )
    .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "design height exceeds PES's u16 encoding range" ) )?;
    writer.write_u16::< LE >( width_u16 )?;
    writer.write_u16::< LE >( height_u16 )?;

    writer.write_all( b"\x00\x00\x00\x00\x00\x00\x00\x00" )?;

    let placeholder_needs_section_data = writer.stream_position()?;
    writer.write_u16::< LE >( 0 )?; // placeholder

    Ok( placeholder_needs_section_data )
  }

  /// Writes PES CSewSeg, specs: https://github.com/frno7/libpes/wiki/PES-CSewSeg-section
  fn write_pes_embsewseg_segments< W >
  (
    emb : &EmbroideryFile,
    writer : &mut W,
    threads : &[ Thread ],
    bounds : DesignBounds
  )
  ->
  Result< ( u16, Vec< ( u16, usize ) > ), EmbroideryError >
  where
    W : Write
  {
    let mut section = 0;
    let mut colorlog = vec![];

    let mut previous_color_code = None;
    let mut flag = None;
    let adjust_x = bounds.left + bounds.cx;
    let adjust_y = bounds.bottom + bounds.cy;

    for seg in as_segment_blocks( emb, threads, adjust_x, adjust_y )
    {
      if flag.is_some()
      {
        writer.write_u16::< LE >( 0x8003 )?; // section end
      }
      let ( segments, color_code, flag_ ) = seg;
      flag = Some( flag_ );

      if previous_color_code.is_none() || matches!( previous_color_code, Some( code ) if code != color_code )
      {
        colorlog.push( ( section, color_code ) );
        previous_color_code = Some( color_code );
      }
      // Thread-palette index and segment length: bounded by design complexity in
      // practice, but not by a type-level invariant, so reported rather than truncated.
      let color_code_u16 = u16::try_from( color_code )
      .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "thread palette index exceeds PES's u16 encoding range" ) )?;
      let segments_len_u16 = u16::try_from( segments.len() )
      .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "stitch segment length exceeds PES's u16 encoding range" ) )?;
      writer.write_u16::< LE >( flag.unwrap() )?;
      writer.write_u16::< LE >( color_code_u16 )?;
      writer.write_u16::< LE >( segments_len_u16 )?;

      for segment in segments
      {
        // Stitch coordinates are signed deltas; encode via `i16`/`write_i16` (the same
        // bit pattern the original `as u16` cast produced for in-range values) instead
        // of silently wrapping an out-of-range delta into the wrong coordinate.
        let x = i16::try_from( segment.0 )
        .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "stitch x-coordinate exceeds PES's i16 encoding range" ) )?;
        let y = i16::try_from( segment.1 )
        .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "stitch y-coordinate exceeds PES's i16 encoding range" ) )?;
        writer.write_i16::< LE >( x )?;
        writer.write_i16::< LE >( y )?;
      }

      section += 1;
    }

    let colorlog_len_u16 = u16::try_from( colorlog.len() )
    .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "color log length exceeds PES's u16 encoding range" ) )?;
    writer.write_u16::< LE >( colorlog_len_u16 )?;
    for log in &colorlog
    {
      let log_color_code = u16::try_from( log.1 )
      .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "thread palette index exceeds PES's u16 encoding range" ) )?;
      writer.write_u16::< LE >( log.0 )?;
      writer.write_u16::< LE >( log_color_code )?;
    }

    Ok( ( section, colorlog ) )
  }

  /// A single stitch/jump segment block: point list, thread palette index, and PES block flag
  type SegmentBlock = ( Vec< ( i32, i32 ) >, usize, u16 );

  fn as_segment_blocks( emb : &EmbroideryFile, threads : &[ Thread ], adjust_x : i32, adjust_y : i32 )
  ->
  Vec< SegmentBlock >
  {
    let chart : Vec< _ > = threads.iter().map( Some ).collect();

    let mut color_index = 0;
    let mut current_thread = emb.get_thread_or_filler( color_index );
    color_index += 1;
    let mut color_code = thread::find_nearest_color( &current_thread.color, &chart ).unwrap();
    let mut stitched_x = 0;
    let mut stitched_y = 0;

    let mut ret = vec![];
    for command_block in emb.as_command_blocks()
    {
      let mut block = vec![];
      let flag : u16;
      let instruction = command_block[ 0 ].instruction;
      match instruction
      {
        Instruction::Jump =>
        {
          block.push( ( stitched_x - adjust_x, stitched_y - adjust_y ) );
          let last_instruction = command_block.last().unwrap();
          block.push( ( last_instruction.x - adjust_x, last_instruction.y - adjust_y ) );
          flag = 1;
        },
        Instruction::ColorChange =>
        {
          current_thread = emb.get_thread_or_filler( color_index );
          color_index += 1;
          color_code = thread::find_nearest_color( &current_thread.color, &chart ).unwrap();
          // flag = 1;
          continue;
        },
        Instruction::Stitch =>
        {
          for stitch in command_block
          {
            stitched_x = stitch.x;
            stitched_y = stitch.y;
            block.push( ( stitched_x - adjust_x, stitched_y - adjust_y ) );
          }
          flag = 0;
        },
        _ => continue,
      }
      ret.push( ( block, color_code, flag ) );
    }
    ret
  }

  fn write_pes_addendum< W >( writer : &mut W, color_indices : &[ usize ], rgb_list : &[ Color ] )
  ->
  Result< (), EmbroideryError >
  where
    W : Write
  {
    let count = color_indices.len();
    // `color_indices` comes from `pec::write_content`, whose values are indices into
    // the fixed 65-entry thread palette (see `pec::write_pec_header`), so every value
    // is < 65 and fits in `u8`.
    let color_indices : Vec< _ > = color_indices.iter().map( | v | *v as u8 ).collect();
    let spaces = vec![ 0x20_u8; 128_usize.wrapping_sub( count ) ];

    writer.write_all( &color_indices )?;
    writer.write_all( &spaces )?;

    let blank = vec![ 0x00_u8; 0x90 ];
    for _ in 0..rgb_list.len()
    {
      writer.write_all( &blank )?;
    }
    for color in rgb_list
    {
      writer.write_all( &[ color.r, color.g, color.b ] )?;
    }

    Ok( () )
  }

  /// Writes a UTF8 `String` with len of `u16`
  fn write_pes_string16< W >( writer : &mut W, str : &str ) -> Result< (), std::io::Error >
  where
    W : Write
  {
    let len = str.len().min( usize::from( u16::MAX ) );
    // Bounded above by the `.min( usize::from( u16::MAX ) )` clamp on the line above.
    let len_u16 = len as u16;
    writer.write_u16::< LE >( len_u16 )?;
    writer.write_all( &str.as_bytes()[ ..len ] )?;

    Ok( () )
  }

  /// Writes a UTF8 `String` with len of `u8`
  fn write_pes_string8< W >( writer : &mut W, str : &str ) -> Result< (), std::io::Error >
  where
    W : Write
  {
    let len = str.len().min( usize::from( u8::MAX ) );
    // Bounded above by the `.min( usize::from( u8::MAX ) )` clamp on the line above.
    let len_u8 = len as u8;
    writer.write_u8( len_u8 )?;
    writer.write_all( &str.as_bytes()[ ..len ] )?;

    Ok( () )
  }
}

crate::mod_interface!
{
  orphan use write;
}
