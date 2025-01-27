//! 
//! # PEC format reader
//! Original implementation refers to https://github.com/EmbroidePy/pyembroidery/blob/main/pyembroidery/PecReader.py
//! 

mod private
{
  use crate::*;
  use error::EmbroideryError;
  use format::pec::pec_threads;
  use metadata::Graphics;
  use embroidery_file::EmbroideryFile;
  use thread::Thread;
  use std::{ collections::HashMap, io, path::Path };
  use io::{ Cursor, Read, Seek, SeekFrom, BufReader, ErrorKind };
  use byteorder::{ ReadBytesExt, LE };

  // These are PEC instruction encodings 
  const JUMP_CODE : u8 = 0x10;
  const TRIM_CODE : u8 = 0x20;
  const FLAG_LONG : u8 = 0x80;

  /// Reads PEC file at `path`
  pub fn read_file< P >( path : P ) -> Result< EmbroideryFile, EmbroideryError >
  where
    P : AsRef< Path >
  {
    let file = std::fs::File::open( path )?;
    let mut reader = BufReader::new( file );
    read( &mut reader )
  }

  /// Reads PEC file from byte slice
  pub fn read_memory( mem : &[ u8 ] ) -> Result< EmbroideryFile, EmbroideryError >
  {
    let mut reader = Cursor::new( mem );
    read( &mut reader )
  }

  /// Reads PEC file from `reader`
  pub fn read< R >( reader : &mut R ) -> Result< EmbroideryFile, EmbroideryError >
  where
    R : Read + Seek
  {
    // Header string
    // Should be "#PEC0001", so maybe return Error if it is not
    // TODO: Decide later
    let mut header = [ 0; 8 ];
    reader.read_exact( &mut header )?;
    let header = String::from_utf8_lossy( &header );
    if header != "#PEC0001"
    {
      return Err( EmbroideryError::DecodingError( "Not PEC header encountered".into() ) );
    }
    
    let mut emb = EmbroideryFile::new();
    read_content( &mut emb, reader, &[] )?;
    
    Ok( emb )
  }

  /// Reads PEC file's content, should be used when PEC file is embedded into something else,
  /// so it doesn't have header string at the beginning
  /// 
  /// # Parameters
  /// - `reader`: Read object positioned at the beginning of PEC file
  ///
  /// - `pes_chart`: Thread chart from PES file. Can be empty
  pub fn read_content< R >( emb : &mut EmbroideryFile, reader : &mut R, pes_chart : &[ Thread ] )
  ->
  Result< (), EmbroideryError >
  where
    R : Read + Seek
  {
    // Specs - https://github.com/frno7/libpes/wiki/PEC-section
    // All seek operations skip either undocumented or unused parts of PEC

    // skips "LA:"
    reader.seek( SeekFrom::Current( 3 ) )?;
    // reads 16-byte label
    let mut label = [ 0; 16 ];
    reader.read_exact( &mut label )?;
    let label = String::from_utf8_lossy( &label ).trim_end().to_owned();
    emb.get_mut_metadata().set_name( Some( label ) );

    reader.seek( SeekFrom::Current( 0xF ) )?;

    let pec_graphics_byte_stride = reader.read_u8()?;
    let pec_graphics_icon_height = reader.read_u8()?;

    reader.seek( SeekFrom::Current( 0xC ) )?;

    let color_changes = reader.read_u8()?;
    // PEC uses color_changes - 1, so 0xFF actually means 0.
    let count_colors = color_changes.wrapping_add( 1 ) as usize;

    // array of indices into thread palette
    let mut color_bytes = vec![ 0; count_colors ];
    reader.read_exact( &mut color_bytes )?;

    // Actually this variable could be eliminated
    // but it is saved to preserve similarity with original implementation 
    let mut threads = vec![];

    map_pec_colors( emb, &color_bytes, pes_chart, &mut threads );

    reader.seek( SeekFrom::Current( ( 0x1D0 - color_changes as u16 ) as i64 ) )?;
    let stitch_block_len = reader.read_u24::< LE >()? as u64;
    let stitch_block_end = stitch_block_len - 5 + reader.stream_position()?;

    reader.seek( SeekFrom::Current( 0x0B ) )?;
    read_pec_instructions( emb, reader )?;

    reader.seek( SeekFrom::Start( stitch_block_end ) )?;

    let byte_size = pec_graphics_byte_stride as usize * pec_graphics_icon_height as usize;
    // PEC stores one general thumbnail and one for each thread
    read_pec_graphics( emb, reader, byte_size, pec_graphics_byte_stride, &threads );

    emb.interpolate_duplicate_color_as_stop();

    Ok( () )
  }

  /// Uploads thread palette
  fn map_pec_colors
  (
    emb : &mut EmbroideryFile, 
    color_bytes : &[ u8 ],
    chart : &[ Thread ],
    values : &mut Vec< Thread >
  )
  {
    // Maps color palette in next way:
    // If `pes_chart` is empty then default PEC palette is used
    // If `pes_chart` is equal or larger than amount of used colors in PEC file
    // then it completely overrides default palette
    // If it is not empty but smaller than amount of used colors
    // then it gets mixed with default palette

    if chart.is_empty()
    {
      process_pec_colors( emb, color_bytes, values );
    }
    else if chart.len() >= color_bytes.len()
    {
      for thread in chart
      {
        emb.add_thread( thread.clone() );
        values.push( thread.clone() );
      }
    }
    else
    {
      process_pec_table( emb, color_bytes, chart.to_vec(), values );
    }
  }

  /// Uploads default PEC threads
  fn process_pec_colors( emb : &mut EmbroideryFile, color_bytes : &[ u8 ], values : &mut Vec< Thread > )
  {
    let threads = pec_threads();
    let max_value = threads.len();
    for byte in color_bytes
    {
      let thread = &threads[ *byte as usize % max_value ];
      emb.add_thread( thread.clone() );
      values.push( thread.clone() );
    }
  }

  /// Merges default PEC threads and chart from PES together
  fn process_pec_table
  (
    emb : &mut EmbroideryFile, 
    color_bytes : &[ u8 ],
    mut chart : Vec< Thread >,
    values : &mut Vec< Thread >  
  )
  {
    // Basically, drains threads from chart, and when it is empty
    // takes threads from default palette

    let threads = pec_threads();
    let max_value = threads.len();
    let mut thread_map : HashMap< usize, Thread > = HashMap::new();

    for byte in color_bytes
    {
      let color_index = *byte as usize % max_value;
      let thread_value = thread_map.get( &color_index );
      
      match thread_value
      {
        Some( thread ) =>
        {
          emb.add_thread( thread.clone() );
          values.push( thread.clone() );
        }
        None =>
        {
          let thread = if chart.len() > 0
          {
            chart.remove( 0 )
          }
          else 
          {
            threads[ color_index ].clone()
          };
          thread_map.insert( color_index, thread );
        }    
      }
    }
  }

  /// Reads machine instructions section
  fn read_pec_instructions< R >( emb : &mut EmbroideryFile, reader : &mut R )
  ->
  Result< (), std::io::Error >
  where
    R : Read + Seek
  {
    // used to read value or break from loop if there's nothing to read
    macro_rules! read_val
    {
      () =>
      {
        match reader.read_u8()
        {
          Ok( val ) => val,
          Err( e ) if e.kind() == ErrorKind::UnexpectedEof => 
          {
            break;
          }
          Err( e ) =>
          {
            return Err( e );
          }
        }
      };
    }

    loop
    {
      let val1 : u8 = reader.read_u8()?;
      let mut val2 : u8 = read_val!();

      // This means end of Instruction section
      if val1 == 0xFF && val2 == 0x00 { break; }

      if val1 == 0xFE && val2 == 0xB0
      {
        reader.seek( SeekFrom::Current( 1 ) )?;
        emb.color_change( 0, 0 );
        continue;
      }

      let mut jump = false;
      let mut trim = false;

      // if FLAG_LONG is set then the instruction is 2 bytes long
      let x = if val1 & FLAG_LONG != 0
      {
        trim = ( val1 & TRIM_CODE ) != 0;
        jump = ( val1 & JUMP_CODE ) != 0;
        // convert val1 and val2 into 2-byte instruction
        let code = ( ( val1 as u16 ) << 8 ) | val2 as u16;
        
        // value 2 became part of the `code` so we need to read it again
        val2 = read_val!();

        signed12( code )
      }
      else
      {
        signed7( val1 )
      };

      let y = if val2 & FLAG_LONG != 0
      {
        trim = ( val2 & TRIM_CODE ) != 0;
        jump = ( val2 & JUMP_CODE ) != 0;
        let val3 = read_val!();
        let code = ( ( val2 as u16 ) << 8 ) | val3 as u16;

        signed12( code )
      }
      else
      {
        signed7( val2 )
      };

      if jump
      {
        emb.jump( x, y );
      }
      else if trim
      {
        emb.trim();
        emb.jump( x, y );
      }
      else
      {
        emb.stitch( x, y );
      }
    }
    emb.end();
    Ok( () )
  }

  /// Extracts 12-byte signed integer stored in `u16` into `i32`
  fn signed12( mut b : u16 ) -> i32
  {
    // Only first 12 bits remain
    b &= 0x0FFF;
    
    if b > 0x7FF
    {
      -0x1000 + b as i32
    }
    else
    {
      b as i32
    }
  }
  
  /// Extracts 7-byte signed integer stored in `u8` into `i32`
  fn signed7( b : u8 ) -> i32
  {
    if b > 63
    {
      -128 + b as i32
    }
    else
    {
      b as i32
    }
  }

  /// Reads thumbnail images section
  fn read_pec_graphics< R >
  (
    emb : &mut EmbroideryFile,
    reader : &mut R,
    size : usize,
    stride : u8,
    values : &[ Thread ]
  )
  where
    R : Read
  {
    let mut threads : Vec< Option< Thread > > = values.iter().map( | t | Some( t.clone() ) ).collect();
    // this is needed for first general thumbnail which does not correspond to any thread
    threads.insert
    (
      0,
      None
    );

    for ( i, thread ) in threads.into_iter().enumerate()
    {
      let mut image = vec![ 0; size ];
      let res = reader.read_exact( &mut image );
      match res
      {
        Ok( _ ) =>
        {
          let name = "pec_graphic_".to_string() + &i.to_string();
          let graphics = Graphics::PecGraphics { image, stride, thread };
          emb.get_mut_metadata().insert_graphics( &name, graphics );
        }
        Err( _ ) => {}
      }
    }
  }

  #[ cfg( test ) ]
  mod tests
  {
    use crate::*;
    use format::pec;
    use stitch_instruction::{ Instruction, Stitch };
    use super::read_file;

    #[ test ]
    fn test_read_stitches()
    {
      let emb = read_file( "../../assets/read_sample.pec" ).unwrap();
      let stitches = emb.stitches();

      // these instructions should match instructions when reading with pyembroidery
      assert_eq!( stitches[ 0 ], Stitch { x : 10, y : 20, instruction : Instruction::Jump } );
      assert_eq!( stitches[ 1 ], Stitch { x : 10, y : 20, instruction : Instruction::Stitch } );
      
      assert_eq!( stitches[ 2 ], Stitch { x : 40, y : 60, instruction : Instruction::Stitch } );
      assert_eq!( stitches[ 3 ], Stitch { x : 40, y : 60, instruction : Instruction::ColorChange } );
      assert_eq!( stitches[ 4 ], Stitch { x : 40, y : 60, instruction : Instruction::Trim } );
      
      assert_eq!( stitches[ 5 ], Stitch { x : 43, y : 64, instruction : Instruction::Jump } );
      assert_eq!( stitches[ 6 ], Stitch { x : 43, y : 64, instruction : Instruction::Stitch } );

      assert_eq!( stitches[ 7 ], Stitch { x : 43, y : 64, instruction : Instruction::Stop } );
      assert_eq!( stitches[ 8 ], Stitch { x : 43, y : 64, instruction : Instruction::Trim } );
      
      assert_eq!( stitches[ 9 ], Stitch { x : 63, y : 74, instruction : Instruction::Jump } );
      assert_eq!( stitches[ 10 ], Stitch { x : 63, y : 74, instruction : Instruction::Trim } );
      
      assert_eq!( stitches[ 11 ], Stitch { x : 64, y : 75, instruction : Instruction::Jump } );
      assert_eq!( stitches[ 12 ], Stitch { x : 64, y : 75, instruction : Instruction::Stitch } );
      assert_eq!( stitches[ 13 ], Stitch { x : 64, y : 75, instruction : Instruction::End } );
    }

    #[ test ]
    fn test_read_threads()
    {
      let emb = read_file( "../../assets/read_sample.pec" ).unwrap();
      let threads = emb.threads();
      let default_palette = pec::pec_threads();
      
      assert_eq!( threads[ 0 ], default_palette[ 14 ] );
      assert_eq!( threads[ 1 ], default_palette[ 10 ] );
    }
  }
}

crate::mod_interface!
{
  orphan use read_content;
  orphan use read_file;
  orphan use read_memory;
  orphan use read;
}
