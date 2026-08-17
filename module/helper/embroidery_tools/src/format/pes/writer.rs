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
    emb.color_count_fix();
    emb.stop_interpolate_as_duplicate_color();

    match version
    {
      PESVersion::V1 => version1_write( emb, writer ),
      PESVersion::V6 => version6_write( emb, writer ),
    }
  }

  /// Writes PES version 1 into `writer`
  fn version1_write< W >( emb : &mut EmbroideryFile, writer : &mut W )
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
      header_version1_write( writer, 0 )?;
      // 0000 0000 means no more sections
      writer.write_u16::< LE >( 0x0000 )?;
      writer.write_u16::< LE >( 0x0000 )?;
    }
    else
    {
      header_version1_write( writer, 1 )?;
      // ffff 0000 means more sections
      writer.write_u16::< LE >( 0xFFFF )?;
      writer.write_u16::< LE >( 0x0000 )?;

      let threads = pec::pec_threads();
      _ = pes_block_write( emb, writer, &threads, DesignBounds { left, top, right, bottom, cx, cy } )?;
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

    _ = pec::content_write( emb, writer )?;
    Ok( () )
  }

  fn header_version1_write< W >( writer : &mut W, distinct_block_objects : u16 )
  -> Result< (), EmbroideryError >
  where
    W : Write + Seek
  {
    writer.write_u16::< LE >( 0x01 )?; // scale to fit
    writer.write_u16::< LE >( 0x01 )?; // 0 = 100x100, 1 = 130x180 hoop
    writer.write_u16::< LE >( distinct_block_objects )?;
    
    Ok( () )
  }

  fn version6_write< W >( emb : &mut EmbroideryFile, writer : &mut W )
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
      header_version6_write( emb, writer, 0 )?;
      writer.write_u16::< LE >( 0x0000 )?;
      writer.write_u16::< LE >( 0x0000 )?;
    }
    else
    {
      header_version6_write( emb, writer, 1 )?;
      writer.write_u16::< LE >( 0xFFFF )?;
      writer.write_u16::< LE >( 0x0000 )?;
      let log = pes_block_write( emb, writer, emb.threads(), DesignBounds { left, top, right, bottom, cx, cy } )?;
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
    // See the analogous conversion in `version1_write`: stream position is `u64`,
    // the placeholder field is `u32`, and this is not provably bounded.
    let current_pos_u32 = u32::try_from( current_pos )
    .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "PES stream position exceeds u32 range" ) )?;
    writer.write_u32::< LE >( current_pos_u32 )?;
    writer.seek( SeekFrom::Start( current_pos ) )?;
    let color_info = pec::content_write( emb, writer )?;
    let rgb_list : Vec< _ > = emb.threads().iter().map( | v | v.color ).collect();
    pes_addendum_write( writer, &color_info, &rgb_list )?; // is it really necessary?
    writer.write_u16::< LE >( 0x0000 )?;

    Ok( () )
  }

  fn header_version6_write< W >
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

    pes_string8_write( writer, emb.metadata_get().name_get().unwrap_or_default() )?;
    pes_string8_write( writer, emb.metadata_get().text_get( "category" ).unwrap_or_default() )?;
    pes_string8_write( writer, emb.metadata_get().text_get( "author" ).unwrap_or_default() )?;
    pes_string8_write( writer, emb.metadata_get().text_get( "keywords" ).unwrap_or_default() )?;
    pes_string8_write( writer, emb.metadata_get().text_get( "comments" ).unwrap_or_default() )?;
    
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
      pes_thread_write( writer, thread )?;
    }

    writer.write_u16::< LE >( distinct_block_objects )?; // number of distinct blocks

    Ok( () )
  }

  fn pes_thread_write< W >( writer : &mut W, thread : &Thread ) -> Result< (), EmbroideryError >
  where
    W : Write
  {
    // Specs: https://github.com/frno7/libpes/wiki/PES-header-section#color-subsection

    pes_string8_write( writer, &thread.catalog_number )?;
    writer.write_u8( thread.color.r )?;
    writer.write_u8( thread.color.g )?;
    writer.write_u8( thread.color.b )?;
    writer.write_u8( 0 )?;
    writer.write_u32::< LE >( 0xA )?;
    pes_string8_write( writer, &thread.description )?;
    pes_string8_write( writer, &thread.brand )?;
    pes_string8_write( writer, &thread.chart )?;

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
  fn pes_block_write< W >
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

    pes_string16_write( writer, "CEmbOne" )?;
    let placeholder = pes_sewseg_header_write( writer, bounds )?;
    writer.write_u16::< LE >( 0xFFFF )?;
    writer.write_u16::< LE >( 0x0000 )?; // FFFF0000 means more blocks exist

    pes_string16_write( writer, "CSewSeg" )?;
    let ( sections, colorlog ) = pes_embsewseg_segments_write( emb, writer, threads, bounds )?;

    let current_pos = writer.stream_position()?;
    writer.seek( SeekFrom::Start( placeholder ) )?;
    writer.write_u16::< LE >( sections )?;
    writer.seek( SeekFrom::Start( current_pos ) )?;

    writer.write_u16::< LE >( 0x0000 )?;
    writer.write_u16::< LE >( 0x0000 )?;

    Ok( colorlog )
  }

  /// Writes SewSeg header
  fn pes_sewseg_header_write< W >( writer : &mut W, bounds : DesignBounds )
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
  fn pes_embsewseg_segments_write< W >
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
    let mut current_thread = emb.thread_or_filler_get( color_index );
    color_index += 1;
    // Fix(BUG-235)
    // Root cause: `chart` is built from `threads` (== `emb.threads()` for PES v6), which is
    // empty for any design that never had a thread added and has no Stitch/SewTo/NeedleAt
    // instruction for `color_count_fix` to backfill (e.g. a jump-only design, or even just
    // a bare `emb.end()`). `nearest_color_find` against an empty chart returns `None` by
    // contract, and `.unwrap()` here paniced unconditionally before the block-processing
    // loop below even started -- regardless of what the design's stitches actually were.
    // Pitfall: `current_thread` (from `thread_or_filler_get`) is always a real, valid thread
    // even when `emb.threads()` is empty (it falls back to `random_thread_get`), but `chart`
    // has no entries to match it against in that case -- there is no meaningful "index into
    // the design's own (empty) thread palette" to report, so falling back to `0` (this
    // section is write-only informational data for external PES consumers; this codebase's
    // own reader never reads it back, see `pes/reader.rs`) is the same "substitute something
    // reasonable instead of erroring" convention `thread_or_filler_get` itself already uses.
    let mut color_code = thread::nearest_color_find( &current_thread.color, &chart ).unwrap_or( 0 );
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
          current_thread = emb.thread_or_filler_get( color_index );
          color_index += 1;
          // Fix(BUG-235): same empty-`chart` fallback as this function's initial `color_code` above.
          color_code = thread::nearest_color_find( &current_thread.color, &chart ).unwrap_or( 0 );
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

  fn pes_addendum_write< W >( writer : &mut W, color_indices : &[ usize ], rgb_list : &[ Color ] )
  ->
  Result< (), EmbroideryError >
  where
    W : Write
  {
    let count = color_indices.len();
    // Fix(BUG-234)
    // Root cause: PES v6's addendum color-index field is a fixed 128-byte slot, but the
    // only existing guard on `color_indices`'s length (`pec_header_write`'s own "too many
    // color changes" check) allows up to 255 -- for any `count` in `129..=255`,
    // `128_usize.wrapping_sub( count )` silently underflowed to a value near `usize::MAX`,
    // which then became a `vec![0x20u8; ...]` allocation size far past `isize::MAX`,
    // panicking with "capacity overflow" instead of returning a catchable error.
    // Pitfall: every other "value exceeds this format's capacity" case in this file reports
    // via `try_from`/an explicit bounds check and a real `EmbroideryError` -- `wrapping_sub`
    // fed straight into an allocation size was the one place that convention wasn't
    // followed, and unsigned wraparound turned a bounds miss into an unhandled panic instead
    // of a `Result::Err`.
    if count > 128
    {
      let msg = format!( "Too many thread/color-change entries for PES addendum. {count} is unsupported value. Maximum: 128" );
      return Err( EmbroideryError::CompatibilityError( msg.into() ) );
    }
    // `color_indices` comes from `pec::content_write`, whose values are indices into
    // the fixed 65-entry thread palette (see `pec::pec_header_write`), so every value
    // is < 65 and fits in `u8`.
    let color_indices : Vec< _ > = color_indices.iter().map( | v | *v as u8 ).collect();
    // Guarded above: `count <= 128` here, so this subtraction cannot underflow.
    let spaces = vec![ 0x20_u8; 128 - count ];

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
  fn pes_string16_write< W >( writer : &mut W, str : &str ) -> Result< (), std::io::Error >
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
  fn pes_string8_write< W >( writer : &mut W, str : &str ) -> Result< (), std::io::Error >
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
