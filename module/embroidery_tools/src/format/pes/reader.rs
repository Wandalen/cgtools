//!
//! # PES format reader.
//! Original implementation refers to https://github.com/EmbroidePy/pyembroidery/blob/main/pyembroidery/PesReader.py
//!

mod private
{
  use crate::{stitch_instruction::{Instruction, Stitch}, thread::SerThread, *};
  use embroidery_file::EmbroideryFile;
  use error::EmbroideryError;
  use format::pec;
  use thread::{ Color, Thread };
  use std::{ fs::File, io, path::Path, process::Command };
  use io::{ BufReader, Read, Seek, SeekFrom, Cursor };
  use byteorder::{ ReadBytesExt as _, LE };

  pub fn read_bytes(data: &[u8], filename: &str) -> Result< EmbroideryFile, EmbroideryError > {
    let src = crate::READ_SRC;
    let mut unique_convertion_path = filename.to_string();
    unique_convertion_path.push_str("CONVERTION_PURPOSES");
    unique_convertion_path.push_str(&rand::random::<u32>().to_string());
    let mut unique_data_path = filename.to_string();
    unique_data_path.push_str("TRANSFER_PURPOSES");
    unique_data_path.push_str(&rand::random::<u32>().to_string());

    std::fs::write(&unique_data_path, data)?;
    _ = Command::new("python3")
        .arg("-c")
        .arg(src)
        .arg(&unique_data_path)
        .arg(&unique_convertion_path)
        // .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .output()?;

    let mut file = File::open(&unique_convertion_path)?;

    let mut size_buffer = [0u8; 4];
    file.read_exact(&mut size_buffer)?;
    let num_rows = u32::from_le_bytes(size_buffer) as usize;

    let mut stitches = Vec::new();
    for _ in 0..num_rows {
        let mut row_buffer = [0u8; 12]; // 3 * i32 (each i32 = 4 bytes)
        file.read_exact(&mut row_buffer)?;

        let x = i32::from_le_bytes(row_buffer[0..4].try_into().unwrap());
        let y = i32::from_le_bytes(row_buffer[4..8].try_into().unwrap());
        let instruction = Instruction::from(i32::from_le_bytes(row_buffer[8..12].try_into().unwrap()));

        stitches.push( Stitch { x, y, instruction });
    }

    file.read_exact(&mut size_buffer)?;
    let metadata_size = u32::from_le_bytes(size_buffer) as usize;

    // Read metadata JSON
    let mut metadata_bytes = vec![0u8; metadata_size];
    file.read_exact(&mut metadata_bytes)?;

    // Convert metadata bytes to JSON string
    // let metadata_str = str::from_utf8(&metadata_bytes).expect("Invalid UTF-8 metadata");
    let metadata: Vec<SerThread> = serde_json::from_slice(&metadata_bytes).expect("Invalid JSON");
    let threads: Vec<Thread> =  metadata.into_iter().map(|item| item.into()).collect();
    let mut res = EmbroideryFile::new();
    res.threads = threads;
    res.stitches = stitches;

    std::fs::remove_file(unique_convertion_path)?;
    std::fs::remove_file(unique_data_path)?;
    Ok(res)
  }

  /// Reads PES file at `path`
  pub fn read_file< P >( path : P ) -> Result< EmbroideryFile, EmbroideryError >
  where
    P : AsRef< Path >
  {
    let file = std::fs::File::open( path )?;
    let mut reader = BufReader::new( file );
    read( &mut reader )
  }

  /// Reads PES file from byte slice
  pub fn read_memory( mem : &[ u8 ] ) -> Result< EmbroideryFile, EmbroideryError >
  {
    let mut reader = Cursor::new( mem );
    read( &mut reader )
  }

  /// Read PES file. Currently supported versions: 1, 6
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
      pec::read_content( &mut emb, reader, &[] )?;
      return Ok( emb );
    }
    // Position where PEC section starts
    let pec_block_position = reader.read_u32::< LE >()?;
    let mut threads = vec![];

    if pes_string == "#PES0001".as_bytes()
    {
      emb.get_mut_metadata().insert_text( "version", "1".into() );
      // pyembroidery just don't do anything for this version
      // and goes straight to reading PEC section
    }
    else if pes_string == "#PES0060".as_bytes()
    {
      emb.get_mut_metadata().insert_text( "version", "6".into() );
      read_header_version6( &mut emb, reader, &mut threads )?;
    }
    else
    {
      let msg = format!( "Unupported PES version: {}", String::from_utf8_lossy( &pes_string ) );
      return Err( EmbroideryError::UnsupportedFormatError( msg.into() ) );
    }
    // Read PEC
    reader.seek( SeekFrom::Start( pec_block_position as u64 ) )?;
    pec::read_content( &mut emb, reader, &threads )?;

    Ok( emb )
  }

  /// Reads PES header version 6. If it encounters any complex thing it just returns immediately
  fn read_header_version6< R >( emb : &mut EmbroideryFile, reader : &mut R, threads : &mut Vec< Thread > )
  -> Result< (), EmbroideryError >
  where
    R : Read + Seek
  {
    reader.seek( SeekFrom::Current( 4 ) )?; // skip some offset
    read_pes_metadata( emb, reader )?;
    reader.seek( SeekFrom::Current( 36 ) )?;
    let val = read_pes_string( reader )?;
    if let Some( val ) = val
    {
      emb.get_mut_metadata().insert_text( "image_file", val );
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
      threads.push( read_pes_thread( reader )? );
    }
    Ok( () )
  }

  /// Reads few metadata fields
  fn read_pes_metadata< R >( emb : &mut EmbroideryFile, reader : &mut R ) -> Result< (), EmbroideryError >
  where
    R : Read
  {
    let val = read_pes_string( reader )?;
    if let Some( val ) = val
    {
      emb.get_mut_metadata().insert_text( "name", val );
    }
    let val = read_pes_string( reader )?;
    if let Some( val ) = val
    {
      emb.get_mut_metadata().insert_text( "category", val );
    }
    let val = read_pes_string( reader )?;
    if let Some( val ) = val
    {
      emb.get_mut_metadata().insert_text( "author", val );
    }
    let val = read_pes_string( reader )?;
    if let Some( val ) = val
    {
      emb.get_mut_metadata().insert_text( "keywords", val );
    }
    let val = read_pes_string( reader )?;
    if let Some( val ) = val
    {
      emb.get_mut_metadata().insert_text( "comments", val );
    }

    Ok( () )
  }

  /// Reads PES thread
  fn read_pes_thread< R >( reader : &mut R ) -> Result< Thread, EmbroideryError >
  where
    R : Read + Seek
  {
    let mut thread = Thread
    {
      catalog_number : read_pes_string( reader )?.map_or( "0".into(), | v | v.into() ),
      ..Default::default()
    };

    let r = reader.read_u8()?;
    let g = reader.read_u8()?;
    let b = reader.read_u8()?;
    thread.color = Color { r, g, b };
    reader.seek( SeekFrom::Current( 5 ) )?; // Some offset
    thread.description = read_pes_string( reader )?.map_or( "Unknown".into(), | v | v.into() );
    thread.brand = read_pes_string( reader )?.map_or( Default::default(), | v | v.into() );
    thread.chart = read_pes_string( reader )?.map_or( Default::default(), | v | v.into() );

    Ok( thread )
  }

  /// Reads PES string. First byte is lenght of a string, then its content
  fn read_pes_string< R >( reader : &mut R ) -> Result< Option< String >, EmbroideryError >
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

  #[ cfg( test )]
  mod tests
  {
    use crate::*;
    use embroidery_file::EmbroideryFile;
    use thread::*;
    use std::io::Cursor;
    use format::pes;
    use super::read;

    #[ test ]
    fn test_version6()
    {
      let mut emb = EmbroideryFile::new();
      emb.stitch( 0, 0 );
      emb.end();
      let metadata = emb.get_mut_metadata();
      metadata.insert_text( "category", "Fantasy".into() );
      metadata.insert_text( "author", "George R.R. Martin".into() );
      metadata.insert_text( "keywords", "Dragons, mediavel, story, adventure".into() );
      metadata.insert_text( "comments", "When \"The Winds of Winter\"?".into() );

      let color = Color { r : 123, g : 234, b : 125 };
      let thread = Thread
      {
        color,
        description : "A very good thread".into(),
        catalog_number : "197".into(),
        brand : "No brand".into(),
        chart : "No chart".into(),
        ..Default::default()
      };
      emb.add_thread( thread );

      let mut memory = vec![ 0_u8; 2048 ];
      {
        let mut writer = Cursor::new( &mut memory );
        pes::write( &mut emb, &mut writer, pes::PESVersion::V6 ).unwrap();
      }

      let mut reader = Cursor::new( &mut memory );
      let emb = read( &mut reader ).unwrap();
      println!( "{:?}", emb.stitches() );
      let metadata = emb.get_metadata();

      let category = metadata.get_text( "category" ).unwrap();
      let author = metadata.get_text( "author" ).unwrap();
      let keywords = metadata.get_text( "keywords" ).unwrap();
      let comments = metadata.get_text( "comments" ).unwrap();

      assert_eq!( category, "Fantasy" );
      assert_eq!( author, "George R.R. Martin" );
      assert_eq!( keywords, "Dragons, mediavel, story, adventure" );
      assert_eq!( comments, "When \"The Winds of Winter\"?" );

      let thread = &emb.threads()[ 0 ];
      assert_eq!( thread.description, "A very good thread" );
      assert_eq!( thread.catalog_number, "197" );
      assert_eq!( thread.brand, "No brand" );
      assert_eq!( thread.chart, "No chart" );
    }
  }
}

crate::mod_interface!
{
  orphan use read_file;
  orphan use read_memory;
  orphan use read;
  orphan use read_bytes;
}
