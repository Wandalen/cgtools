//! 
//! # PES format reader.
//! Original implementation refers to <https://github.com/EmbroidePy/pyembroidery/blob/main/pyembroidery/PesReader.py>
//! 

mod private
{
  use crate::{ embroidery_file, format, thread, error };
  use embroidery_file::EmbroideryFile;
  use error::EmbroideryError;
  use format::pec;
  use thread::{ Color, Thread };
  use std::{ io, path::Path };
  use io::{ BufReader, Read, Seek, SeekFrom, Cursor };
  use byteorder::{ ReadBytesExt as _, LE };

  /// Reads PES file at `path`
  /// # Errors
  /// Returns `EmbroideryError::IOError` if the file cannot be opened or read.
  /// Propagates any error returned by [`read`].
  #[ inline ]
  pub fn file_read< P >( path : P ) -> Result< EmbroideryFile, EmbroideryError >
  where
    P : AsRef< Path >
  {
    let file = std::fs::File::open( path )?;
    let mut reader = BufReader::new( file );
    read( &mut reader )
  }

  /// Reads PES file from byte slice
  /// # Errors
  /// Propagates any error returned by [`read`].
  #[ inline ]
  pub fn memory_read( mem : &[ u8 ] ) -> Result< EmbroideryFile, EmbroideryError >
  {
    let mut reader = Cursor::new( mem );
    read( &mut reader )
  }

  /// Read PES file. Currently supported versions: 1, 6
  /// # Errors
  /// Returns `EmbroideryError::IOError` if `reader` fails.
  /// Returns `EmbroideryError::UnsupportedFormatError` if the header is not a
  /// recognized PES/PEC version.
  /// Propagates any error from decoding the embedded PEC section.
  #[ inline ]
  pub fn read< R >( reader : &mut R ) -> Result< EmbroideryFile, EmbroideryError >
  where
    R : Read + Seek
  {
    let mut emb = EmbroideryFile::new();
    
    // Header string
    let mut pes_string = [ 0_u8; 8 ];
    reader.read_exact( &mut pes_string )?;
    
    if pes_string == "#PEC0001".as_bytes()
    {
      pec::content_read( &mut emb, reader, &[] )?;
      return Ok( emb );
    }
    // Position where PEC section starts
    let pec_block_position = reader.read_u32::< LE >()?;
    let mut threads = vec![];

    if pes_string == "#PES0001".as_bytes()
    {
      emb.metadata_get_mut().text_insert( "version", "1".into() );
      // pyembroidery just don't do anything for this version
      // and goes straight to reading PEC section
    }
    else if pes_string == "#PES0060".as_bytes()
    {
      emb.metadata_get_mut().text_insert( "version", "6".into() );
      header_version6_read( &mut emb, reader, &mut threads )?;
    }
    else
    {
      let msg = format!( "Unupported PES version: {}", String::from_utf8_lossy( &pes_string ) );
      return Err( EmbroideryError::UnsupportedFormatError( msg.into() ) );
    }
    // Read PEC
    reader.seek( SeekFrom::Start( u64::from( pec_block_position ) ) )?;
    pec::content_read( &mut emb, reader, &threads )?;

    Ok( emb )
  }

  /// Reads PES header version 6. If it encounters any complex thing it just returns immediately
  fn header_version6_read< R >( emb : &mut EmbroideryFile, reader : &mut R, threads : &mut Vec< Thread > )
  -> Result< (), EmbroideryError >
  where
    R : Read + Seek
  {
    reader.seek( SeekFrom::Current( 4 ) )?; // skip some offset
    pes_metadata_read( emb, reader )?;
    reader.seek( SeekFrom::Current( 36 ) )?;
    let val = pes_string_read( reader )?;
    if let Some( val ) = val
    {
      emb.metadata_get_mut().text_insert( "image_file", val );
    }

    reader.seek( SeekFrom::Current( 24 ) )?;

    let count_programmable_fills = reader.read_u16::< LE >()?;
    if count_programmable_fills != 0 { return Ok( () ); }

    let count_motifs = reader.read_u16::< LE >()?;
    if count_motifs != 0 { return Ok( () ); }

    let count_feather_patterns = reader.read_u16::< LE >()?;
    if count_feather_patterns != 0 { return Ok( () ); }

    let count_threads = reader.read_u16::< LE >()?;
    for _ in 0..count_threads
    {
      threads.push( pes_thread_read( reader )? );
    }
    Ok( () )
  }

  /// Reads few metadata fields
  fn pes_metadata_read< R >( emb : &mut EmbroideryFile, reader : &mut R ) -> Result< (), EmbroideryError >
  where
    R : Read
  {
    let val = pes_string_read( reader )?;
    if let Some( val ) = val
    {
      emb.metadata_get_mut().text_insert( "name", val );
    }
    let val = pes_string_read( reader )?;
    if let Some( val ) = val
    {
      emb.metadata_get_mut().text_insert( "category", val );
    }
    let val = pes_string_read( reader )?;
    if let Some( val ) = val
    {
      emb.metadata_get_mut().text_insert( "author", val );
    }
    let val = pes_string_read( reader )?;
    if let Some( val ) = val
    {
      emb.metadata_get_mut().text_insert( "keywords", val );
    }
    let val = pes_string_read( reader )?;
    if let Some( val ) = val
    {
      emb.metadata_get_mut().text_insert( "comments", val );
    }

    Ok( () )
  }

  /// Reads PES thread
  fn pes_thread_read< R >( reader : &mut R ) -> Result< Thread, EmbroideryError >
  where
    R : Read + Seek
  {
    let mut thread = Thread
    {
      catalog_number : pes_string_read( reader )?.map_or( "0".into(), std::convert::Into::into ),
      ..Default::default()
    };

    let r = reader.read_u8()?;
    let g = reader.read_u8()?;
    let b = reader.read_u8()?;
    thread.color = Color { r, g, b };
    reader.seek( SeekFrom::Current( 5 ) )?; // Some offset
    thread.description = pes_string_read( reader )?.map_or( "Unknown".into(), std::convert::Into::into );
    thread.brand = pes_string_read( reader )?.map_or( std::borrow::Cow::default(), std::convert::Into::into );
    thread.chart = pes_string_read( reader )?.map_or( std::borrow::Cow::default(), std::convert::Into::into );

    Ok( thread )
  }

  /// Reads PES string. First byte is lenght of a string, then its content
  fn pes_string_read< R >( reader : &mut R ) -> Result< Option< String >, EmbroideryError >
  where
    R : Read
  {
    let len = reader.read_u8()? as usize;
    if len == 0
    {
      Ok( None )
    }
    else
    {
      let mut string = vec![ 0_u8; len ];
      reader.read_exact( &mut string )?;
      Ok( Some( String::from_utf8_lossy( &string ).to_string() ) )
    }
  }
}

crate::mod_interface!
{
  orphan use file_read;
  orphan use memory_read;
  orphan use read;
}
