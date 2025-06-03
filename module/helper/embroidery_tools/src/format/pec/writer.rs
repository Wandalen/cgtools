//! 
//! # PEC format writer.
//! Original implementation refers to https://github.com/EmbroidePy/pyembroidery/blob/main/pyembroidery/PecWriter.py
//! 

mod private
{
  use crate::*;
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
  const JUMP_CODE : u8 = 0b00010000;
  const TRIM_CODE : u8 = 0b00100000;

  /// Writes embroidery file into writer
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
      writer.write_all( name[ ..16 ].as_bytes() )?; 
    }
    else
    {
      let spaces = vec![ b' '; 16 - name.len() ];
      writer.write_all( name.as_bytes() )?;
      writer.write_all( spaces.as_slice() )?;
    };
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

      let bytes = color_indices.iter().map( | v | *v as u8 ).collect::< Vec< _ > >();
      writer.write_u8( add_value as u8 )?;
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
    let width = if !emb.stitches().is_empty() { ( extends.2 - extends.0 ) as u16 } else { 0 };
    let height = if !emb.stitches().is_empty() { ( extends.3 - extends.1 ) as u16 } else { 0 };
    
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
    writer.write_u24::< LE >( instruction_block_len as u32 )?;
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
    const MASK_7_BIT : i8 = 0b01111111;
    
    if !long && value > -64 && value < 63
    {
      // short instruction (1 byte)
      writer.write_i8( value as i8 & MASK_7_BIT )
    }
    else
    {
      // long instruction (2 bytes)
      let mut value = value as i16;
      value &=  0b0000111111111111;
      value |= -0b1000000000000000; // LONG flag
      value |= ( flag as i16 ) << 8; // INSTRUCTION flag
      
      // write two parts of i16 as u8
      writer.write_u8( ( value >> 8 ) as u8 )?;
      writer.write_u8( ( value & 0xFF ) as u8 )
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
    for _ in 0..( emb.threads().len() + 1 )
    {
      writer.write_all( &zeroes )?;
    }

    Ok( () )
  }

  #[ cfg( test ) ]
  mod tests
  {
    use crate::*;
    use format::pec::{ read_memory, pec_threads, write };
    use embroidery_file::EmbroideryFile;
    use stitch_instruction::{ Stitch, Instruction };
    use std::io::Cursor;

    #[ test ]
    fn test_pec_encoding()
    {
      let mut emb = EmbroideryFile::new();
      emb.stitch( 0, 0 );
      emb.stitch( -2, -3 );
      emb.color_change( 0, 0 );
      emb.stitch( 2, 3 );
      emb.trim();
      emb.jump( 40, 30 );
      emb.stitch( 0, 0 );
      emb.stitch( 1, 1 );
      emb.end();

      let threads = pec_threads();
      emb.add_thread( threads[ 0 ].clone() );
      emb.add_thread( threads[ 2 ].clone() );

      let mut memory = vec![ 0_u8; 2048 ];
      
      {
        let mut writer = Cursor::new( &mut memory );
        write( &mut emb, &mut writer ).unwrap();
      }
      
      let emb = read_memory( &memory ).unwrap();
      
      let stitches = emb.stitches();
      assert_eq!( stitches[ 0 ], Stitch { x : 0, y : 0, instruction : Instruction::Stitch } );
      assert_eq!( stitches[ 1 ], Stitch { x : -2, y : -3, instruction : Instruction::Stitch } );
      assert_eq!( stitches[ 2 ], Stitch { x : -2, y : -3, instruction : Instruction::ColorChange } );
      assert_eq!( stitches[ 3 ], Stitch { x : 0, y : 0, instruction : Instruction::Stitch } );
      assert_eq!( stitches[ 4 ], Stitch { x : 0, y : 0, instruction : Instruction::Trim } );
      assert_eq!( stitches[ 5 ], Stitch { x : 40, y : 30, instruction : Instruction::Jump } );
      assert_eq!( stitches[ 6 ], Stitch { x : 40, y : 30, instruction : Instruction::Stitch } );
      assert_eq!( stitches[ 7 ], Stitch { x : 41, y : 31, instruction : Instruction::Stitch } );
      assert_eq!( stitches[ 8 ], Stitch { x : 41, y : 31, instruction : Instruction::End } );
      
      assert_eq!( emb.threads()[ 0 ], threads[ 2 ] );
      // assert_eq!( emb.threads()[ 1 ], threads[ 2 ] );
    }
  }
}

crate::mod_interface!
{
  orphan use write;
  orphan use write_content;
}
