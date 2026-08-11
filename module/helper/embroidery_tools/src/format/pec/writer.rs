//! 
//! # PEC format writer.
//! Original implementation refers to https://github.com/EmbroidePy/pyembroidery/blob/main/pyembroidery/PecWriter.py
//! 

mod private
{
  use crate::{ embroidery_file, stitch_instruction, format, thread, error };
  use embroidery_file::EmbroideryFile;
  use stitch_instruction::Instruction;
  use format::pec::pec_threads;
  use thread::build_unique_palette;
  use byteorder::{ WriteBytesExt as _, LE };
  use std::io::{ Seek, SeekFrom, Write };
  use error::EmbroideryError;

  // Sizes of PEC thumbnail images
  const PEC_ICON_WIDTH : u8 = 48;
  const PEC_ICON_HEIGHT : u8 = 38;
  // Instruction codes
  const JUMP_CODE : u8 = 0b0001_0000;
  const TRIM_CODE : u8 = 0b0010_0000;

  /// Writes embroidery file into writer
  /// # Errors
  /// Returns `EmbroideryError::IOError` if `writer` fails.
  /// Propagates any error returned by [`write_content`].
  #[ inline ]
  pub fn write< W >( emb : &mut EmbroideryFile, writer : &mut W )
  -> Result< (), EmbroideryError >
  where
    W : Write + Seek
  {
    // header
    writer.write_all( "#PEC0001".as_bytes() )?;
    _ = write_content( emb, writer )?;
    Ok( () )
  }

  /// Writes content of embroidery file into writer.
  /// Used standalone when embedding PEC file into something else
  /// # Errors
  /// Returns `EmbroideryError::IOError` if `writer` fails.
  /// Returns `EmbroideryError::CompatibilityError` if the design uses more colors
  /// than PEC's format can encode (255).
  #[ inline ]
  pub fn write_content< W >( emb : &mut EmbroideryFile, writer : &mut W )
  -> Result< Vec< usize >, EmbroideryError >
  where
    W : Write + Seek
  {
    // Specs: https://github.com/frno7/libpes/wiki/PEC-section
    
    // Stitch list should not be empty, at least `end` should be there
    if emb.stitches().is_empty()
    {
      emb.end();
    }
    
    emb.fix_color_count();
    emb.interpolate_stop_as_duplicate_color();
    
    let extends = emb.bounds();
    let color_indices = write_pec_header( emb, writer )?;
    write_pec_block( emb, extends, writer )?;
    write_pec_graphics( emb, writer )?;

    Ok( color_indices )
  }

  /// Writes PEC header into writer
  fn write_pec_header< W >( emb : &EmbroideryFile, writer : &mut W )
  ->
  Result< Vec< usize >, EmbroideryError >
  where
    W : Write + Seek
  {
    // Header layout:
    // https://github.com/frno7/libpes/wiki/PEC-section#:~:text=The%20first%20part%20of%20the%20PEC%20section%20is%20512%20bytes.
    // Write name
    let name = emb.get_metadata().get_name().unwrap_or( "Untitled" );
    writer.write_all( "LA:".as_bytes() )?;
    if name.len() >= 16
    {
      writer.write_all( &name.as_bytes()[ ..16 ] )?;
    }
    else
    {
      let spaces = vec![ b' '; 16 - name.len() ];
      writer.write_all( name.as_bytes() )?;
      writer.write_all( spaces.as_slice() )?;
    }
    writer.write_u8( b'\r' )?;
    // unknown
    writer.write_all( b"\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\xFF\x00" )?;
    
    // division to converts width from bit-wise to byte-wise
    writer.write_u8( PEC_ICON_WIDTH / 8 )?;
    // height goes as is
    writer.write_u8( PEC_ICON_HEIGHT )?;

    let thread_palette = pec_threads();
    let color_indices = build_unique_palette( &thread_palette, &emb.threads()[ 1.. ] );
    let current_thread_count = color_indices.len();

    if current_thread_count != 0
    {
      // unknown, just spaces
      writer.write_all( b"\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20" )?;
      let add_value = current_thread_count - 1;
      
      // Too many color changes
      if add_value >= 255
      {
        let msg = format!( "Too many color changes. {current_thread_count} is unsupported value. Maximum: 255" );
        return Err( EmbroideryError::CompatibilityError( msg.into() ) );
      }

      // `color_indices` holds indices returned by `build_unique_palette` into
      // `thread_palette`, a fixed 65-entry array (see `pec_threads`), so every
      // value is < 65 and fits in `u8`.
      let bytes = color_indices.iter().map( | v | *v as u8 ).collect::< Vec< _ > >();
      // Guarded above: the `add_value >= 255` branch already returned an error,
      // so `add_value` is always < 255 here and fits in `u8`.
      let add_value_u8 = add_value as u8;
      writer.write_u8( add_value_u8 )?;
      writer.write_all( &bytes )?;
    }
    else
    {
      writer.write_all( b"\x20\x20\x20\x20\x64\x20\x00\x20\x00\x20\x20\x20\xFF" )?;
    }

    // fill with spaces
    for _ in current_thread_count..463
    {
      writer.write_all( b"\x20" )?;
    }

    Ok( color_indices )
  }

  /// Writes data block of PEC into writer
  fn write_pec_block< W >( emb : &EmbroideryFile, extends : ( i32, i32, i32, i32 ), writer : &mut W )
  -> Result< (), std::io::Error >
  where
    W : Write + Seek
  {
    // `extends` come from `emb.bounds()`; an embroidery design can in principle carry
    // coordinates wider than PEC's `u16` field, so this is a real, reportable error
    // rather than a value we can prove bounded ahead of time.
    let width = if emb.stitches().is_empty()
    {
      0
    }
    else
    {
      u16::try_from( extends.2 - extends.0 )
      .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "design width exceeds PEC's u16 coordinate range" ) )?
    };
    let height = if emb.stitches().is_empty()
    {
      0
    }
    else
    {
      u16::try_from( extends.3 - extends.1 )
      .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "design height exceeds PEC's u16 coordinate range" ) )?
    };

    let instruction_block_position = writer.stream_position()?;
    writer.write_all( b"\x00\x00" )?;
    writer.write_u24::< LE >( 0 )?; // will be overwritten few lines below
    writer.write_all( b"\x31\xff\xf0" )?; // unknown
    writer.write_u16::< LE >( width )?;
    writer.write_u16::< LE >( height )?;
    writer.write_u16::< LE >( 0x1E0 )?; // unknown
    writer.write_u16::< LE >( 0x1B0 )?;

    write_pec_instructions( emb, writer )?;

    let current_pos = writer.stream_position()?;
    let instruction_block_len = current_pos - instruction_block_position;
    // return position back and write `instruction_block_len` 
    writer.seek( SeekFrom::Start( instruction_block_position + 2 ) )?;
    // Realistic embroidery instruction blocks are tiny, but this is a length
    // computed from stream positions, not a value bounded by any type-level
    // invariant, so an out-of-range value is reported rather than truncated.
    let instruction_block_len = u32::try_from( instruction_block_len )
    .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "PEC instruction block exceeds u32 range" ) )?;
    writer.write_u24::< LE >( instruction_block_len )?;
    writer.seek( SeekFrom::Start( current_pos ) )?;

    Ok( () )
  }

  /// Writes embroidery instructions in PEC specific way
  fn write_pec_instructions< W >( emb : &EmbroideryFile, writer : &mut W ) -> Result< (), std::io::Error >
  where
    W : Write + Seek
  {
    // this is kind of tricky
    // some explanation:
    // https://github.com/frno7/libpes/wiki/writing-PEC

    let stitches = emb.stitches();
    let mut color_two = true;
    let mut jumping = true;
    let mut init = true;

    let mut xx = 0;
    let mut yy = 0;

    for stitch in stitches
    {
      let instruction = stitch.instruction;  
      
      let x = stitch.x;
      let y = stitch.y;

      let dx = x - xx;
      let dy = y - yy;

      xx += dx;
      yy += dy;

      match instruction
      {
        Instruction::Stitch => 
        {
          if jumping
          {
            if dx != 0 && dy != 0
            {
              write_stitch( writer, 0, 0 )?;
            }
            jumping = false;
          }
          write_stitch( writer, dx, dy )?;
        },
        Instruction::Jump => 
        {
          jumping = true;
          if init
          {
            write_jump( writer, dx, dy )?;
          }
          else
          {
            write_trim( writer, dx, dy )?;
          }
        },
        Instruction::ColorChange => 
        {
          if jumping
          {
            write_stitch( writer, 0, 0 )?;
            jumping = false;
          }
          
          writer.write_all( b"\xfe\xb0" )?;

          if color_two
          {
            writer.write_all( b"\x02" )?;
          }
          else
          {
            writer.write_all( b"\x01" )?;
          }
          
          color_two = !color_two;
        },
        Instruction::End =>
        {
          writer.write_all( b"\xff" )?;
          break;
        },
        _ => {}
      }

      init = false;
    }

    Ok( () )
  }

  /// Writes stitch instruction
  fn write_stitch< W >( writer : &mut W, dx : i32, dy : i32 )
  -> Result< (), std::io::Error >
  where
    W : Write
  {
    write_value( writer, dx, false, 0 )?;
    write_value( writer, dy, false, 0 )
  }

  /// Writes jump instruction
  fn write_jump< W >( writer : &mut W, dx : i32, dy : i32 )
  -> Result< (), std::io::Error >
  where
    W : Write
  {
    write_value( writer, dx, true, JUMP_CODE )?;
    write_value( writer, dy, true, JUMP_CODE )
  }

  /// Writes trim instruction
  fn write_trim< W >( writer : &mut W, dx : i32, dy : i32 )
  -> Result< (), std::io::Error >
  where
    W : Write
  {
    write_value( writer, dx, true, TRIM_CODE )?;
    write_value( writer, dy, true, TRIM_CODE )
  }

  /// Writes instruction into writer.
  /// # Parameters
  /// - `value`: Coordinate of instruction
  /// - `long`: Bit indicating that instruction is in long or short form
  /// - `flag`: Instruction value either JUMP or TRIM
  fn write_value< W >( writer : &mut W, value : i32, long : bool, flag : u8 )
  -> Result< (), std::io::Error >
  where
    W : Write
  {
    // Mask to remain only first 7 bit of a number
    const MASK_7_BIT : i8 = 0b0111_1111;

    if !long && value > -64 && value < 63
    {
      // short instruction (1 byte)
      // Guarded by the condition above: `value` is within (-64, 63), well inside `i8`'s range.
      let byte = value as i8 & MASK_7_BIT;
      writer.write_i8( byte )
    }
    else
    {
      // long instruction (2 bytes)
      // Not provably bounded (a stitch delta this large indicates malformed input
      // rather than a value we can silently truncate into the encoding), so an
      // out-of-range delta is reported as a real error instead of corrupted.
      let mut value = i16::try_from( value )
      .map_err( | _ | std::io::Error::new( std::io::ErrorKind::InvalidData, "stitch delta exceeds PEC's i16 encoding range" ) )?;
      value &= 0b0000_1111_1111_1111;
      value |= -0b1000_0000_0000_0000; // LONG flag
      value |= i16::from( flag ) << 8; // INSTRUCTION flag

      // write two parts of i16 as u8 via byte-split, avoiding sign-losing casts
      let bytes = value.to_be_bytes();
      writer.write_u8( bytes[ 0 ] )?;
      writer.write_u8( bytes[ 1 ] )
    }
  }

  /// This currently writes zeroes, not the actual thumbnails
  fn write_pec_graphics< W >( emb : &EmbroideryFile, writer : &mut W )
  -> Result< (), std::io::Error >
  where
    W : Write
  {
    // Thumbnail is bit image so 8 pixels is 1 byte
    let size = ( PEC_ICON_WIDTH / 8 * PEC_ICON_HEIGHT ) as usize;
    let zeroes = vec![ 0_u8; size ];
    for _ in 0..=emb.threads().len()
    {
      writer.write_all( &zeroes )?;
    }

    Ok( () )
  }
}

crate::mod_interface!
{
  orphan use write;
  orphan use write_content;
}
