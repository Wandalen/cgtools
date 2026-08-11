//! PES format tests — writer output pinned byte-for-byte against reference fixtures
//! in `test_files/`, and a write→read roundtrip preserving metadata and threads.
//! Public API only.

use std::io::Cursor;
use embroidery_tools::embroidery_file::EmbroideryFile;
use embroidery_tools::format::{ pec, pes };
use embroidery_tools::thread::{ Color, Thread };

/// Builds the stitch program the reference files in `test_files/` were generated from.
fn fixture_program() -> EmbroideryFile
{
  let mut emb = EmbroideryFile::new();
  emb.stitch( 0, 0 );
  emb.stitch( -40, -30 );
  emb.color_change( 0, 0 );
  emb.trim();
  emb.jump( 2, 3 );
  emb.stitch( 0, 0 );
  emb.trim();
  emb.jump( 90, -100 );
  emb.stitch( 0, 0 );
  emb.stitch( 1, 1 );
  emb.end();

  let threads = pec::pec_threads();
  emb.add_thread( threads[ 1 ].clone() );
  emb.add_thread( threads[ 2 ].clone() );

  emb
}

#[ test ]
fn write_v1_matches_reference_fixture()
{
  let sample = std::fs::read( "test_files/pes_test_v1.pes" ).unwrap();
  let mut emb = fixture_program();

  let mut memory = vec![ 0_u8; 4096 ];
  {
    let mut writer = Cursor::new( &mut memory );
    pes::write( &mut emb, &mut writer, pes::PESVersion::V1 ).unwrap();
  }

  // 192 is the index where the PES section ends and the PEC section starts
  // specifically in this file.
  assert_eq!( &memory[ ..192 ], &sample[ ..192 ] );
}

#[ test ]
fn write_v6_matches_reference_fixture()
{
  let sample = std::fs::read( "test_files/pes_test_v6.pes" ).unwrap();
  let mut emb = fixture_program();

  let mut memory = vec![ 0_u8; 4096 ];
  {
    let mut writer = Cursor::new( &mut memory );
    pes::write( &mut emb, &mut writer, pes::PESVersion::V6 ).unwrap();
  }

  // 361 is the index where the PES section ends and the PEC section starts
  // specifically in this file.
  assert_eq!( &memory[ ..361 ], &sample[ ..361 ] );
}

#[ test ]
fn v6_roundtrip_preserves_metadata_and_threads()
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
  let emb = pes::read( &mut reader ).unwrap();
  let metadata = emb.get_metadata();

  assert_eq!( metadata.get_text( "category" ).unwrap(), "Fantasy" );
  assert_eq!( metadata.get_text( "author" ).unwrap(), "George R.R. Martin" );
  assert_eq!( metadata.get_text( "keywords" ).unwrap(), "Dragons, mediavel, story, adventure" );
  assert_eq!( metadata.get_text( "comments" ).unwrap(), "When \"The Winds of Winter\"?" );

  let thread = &emb.threads()[ 0 ];
  assert_eq!( thread.description, "A very good thread" );
  assert_eq!( thread.catalog_number, "197" );
  assert_eq!( thread.brand, "No brand" );
  assert_eq!( thread.chart, "No chart" );
}
