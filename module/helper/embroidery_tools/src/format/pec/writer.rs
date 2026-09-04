//! 
//! # PEC format writer.
//! Original implementation refers to <https://github.com/EmbroidePy/pyembroidery/blob/main/pyembroidery/PecWriter.py>
//! 

mod private
{
  use crate::{ embroidery_file, stitch_instruction, format, thread, error };
  use embroidery_file::EmbroideryFile;
  use stitch_instruction::Instruction;
  use format::pec::pec_threads;
  use thread::unique_palette_build;
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
  /// Propagates any error returned by [`content_write`].
  #[ inline ]
  pub fn write< W >( emb : &mut EmbroideryFile, writer : &mut W )
  -> Result< (), EmbroideryError >
  where
    W : Write + Seek
  {
    // header
    writer.write_all( "#PEC0001".as_bytes() )?;
    _ = content_write( emb, writer )?;
    Ok( () )
  }

  /// Writes content of embroidery file into writer.
  /// Used standalone when embedding PEC file into something else
  /// # Errors
  /// Returns `EmbroideryError::IOError` if `writer` fails.
  /// Returns `EmbroideryError::CompatibilityError` if the design uses more colors
  /// than PEC's format can encode (255).
  #[ inline ]
  pub fn content_write< W >( emb : &mut EmbroideryFile, writer : &mut W )
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

    emb.color_count_fix();
    emb.stop_interpolate_as_duplicate_color();

    // Fix(BUG-497): `bounds()` now returns `None` for a stitch-free file.
    // `emb.end()` was just called above whenever stitches were empty, so
    // `bounds()` is guaranteed `Some` here -- but `pec_block_write` also
    // independently re-checks `emb.stitches().is_empty()` before ever reading
    // `extends`, so this fallback value is never actually observed either way.
    let extends = emb.bounds().unwrap_or( ( 0, 0, 0, 0 ) );
    let color_indices = pec_header_write( emb, writer )?;
    pec_block_write( emb, extends, writer )?;
    pec_graphics_write( emb, writer )?;

    Ok( color_indices )
  }

  /// Writes PEC header into writer
  fn pec_header_write< W >( emb : &EmbroideryFile, writer : &mut W )
  ->
  Result< Vec< usize >, EmbroideryError >
  where
    W : Write + Seek
  {
    // Header layout:
    // https://github.com/frno7/libpes/wiki/PEC-section#:~:text=The%20first%20part%20of%20the%20PEC%20section%20is%20512%20bytes.
    // Write name
    let name = emb.metadata_get().name_get().unwrap_or( "Untitled" );
    writer.write_all( "LA:".as_bytes() )?;
    // Fix(BUG-498): truncate at a UTF-8 character boundary at or before byte
    // 16, rather than a raw `[ ..16 ]` byte slice that can split a multi-byte
    // character in half. `str_truncate_char_boundary` may return fewer than
    // 16 bytes when the boundary falls short of the limit, so the space
    // padding below is computed from its actual length, not a fixed count --
    // this field is a fixed-width 16-byte slot either way.
    let truncated = format::str_truncate_char_boundary( name, 16 );
    writer.write_all( truncated.as_bytes() )?;
    let spaces = vec![ b' '; 16 - truncated.len() ];
    writer.write_all( spaces.as_slice() )?;
    writer.write_u8( b'\r' )?;
    // unknown
    writer.write_all( b"\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\xFF\x00" )?;
    
    // division to converts width from bit-wise to byte-wise
    writer.write_u8( PEC_ICON_WIDTH / 8 )?;
    // height goes as is
    writer.write_u8( PEC_ICON_HEIGHT )?;

    // Fix(BUG-152)
    // Root cause: sliced `emb.threads()[ 1.. ]`, unconditionally excluding the caller's own
    // first added thread from the written color table -- confused with `pec_threads()[ 0 ]`,
    // the *default palette's* dedicated "invalid value" sentinel entry (see `pec.rs`'s own
    // "This one is for indicating invalid value" comment), which is an unrelated concept.
    // `emb.threads()[ 0 ]` carries no such status; `read_sample_threads_resolve_from_default_palette`
    // (reading a real reference fixture, not this crate's own writer) confirms the reader
    // side already treats index 0 as an ordinary, meaningful thread.
    // Pitfall: a documented sentinel *value* inside a fixed default palette must not be
    // confused with a structural *position* in a caller-supplied, arbitrary-content list --
    // the two only appeared related because a prior test happened to use the sentinel value
    // as its own first thread.
    let thread_palette = pec_threads();
    let color_indices = unique_palette_build( &thread_palette, emb.threads() );
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

      // `color_indices` holds indices returned by `unique_palette_build` into
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
  fn pec_block_write< W >( emb : &EmbroideryFile, extends : ( i32, i32, i32, i32 ), writer : &mut W )
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

    pec_instructions_write( emb, writer )?;

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
  fn pec_instructions_write< W >( emb : &EmbroideryFile, writer : &mut W ) -> Result< (), std::io::Error >
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
              stitch_write( writer, 0, 0 )?;
            }
            jumping = false;
          }
          stitch_write( writer, dx, dy )?;
        },
        Instruction::Jump =>
        {
          jumping = true;
          if init
          {
            jump_write( writer, dx, dy )?;
          }
          else
          {
            trim_write( writer, dx, dy )?;
          }
        },
        Instruction::ColorChange =>
        {
          if jumping
          {
            stitch_write( writer, 0, 0 )?;
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
  fn stitch_write< W >( writer : &mut W, dx : i32, dy : i32 )
  -> Result< (), std::io::Error >
  where
    W : Write
  {
    value_write( writer, dx, false, 0 )?;
    value_write( writer, dy, false, 0 )
  }

  /// Writes jump instruction
  fn jump_write< W >( writer : &mut W, dx : i32, dy : i32 )
  -> Result< (), std::io::Error >
  where
    W : Write
  {
    value_write( writer, dx, true, JUMP_CODE )?;
    value_write( writer, dy, true, JUMP_CODE )
  }

  /// Writes trim instruction
  fn trim_write< W >( writer : &mut W, dx : i32, dy : i32 )
  -> Result< (), std::io::Error >
  where
    W : Write
  {
    value_write( writer, dx, true, TRIM_CODE )?;
    value_write( writer, dy, true, TRIM_CODE )
  }

  /// Writes instruction into writer.
  /// # Parameters
  /// - `value`: Coordinate of instruction
  /// - `long`: Bit indicating that instruction is in long or short form
  /// - `flag`: Instruction value either JUMP or TRIM
  fn value_write< W >( writer : &mut W, value : i32, long : bool, flag : u8 )
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
  fn pec_graphics_write< W >( emb : &EmbroideryFile, writer : &mut W )
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
  orphan use content_write;
}
