use std::collections::HashMap;
use minwebgl as gl;
use crate::json::CharInfo;


#[ repr( C ) ]
#[derive( Debug, Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
pub struct CharData
{
  pub offset : [ f32; 4 ],
  pub uv_info : [ f32; 4 ],
  pub size : [ f32; 2 ],
}

pub struct FormattedText
{
  pub bounding_box : gl::F32x4,
  pub chars : Vec< CharData >
}

pub struct MSDFFont
{
  pub scale : [ f32; 2 ],
  pub chars : HashMap< u8, CharInfo >,
  pub kernings : HashMap< u8, HashMap< u8, f32 > >
}

impl MSDFFont
{
  pub fn format( &self, text :&str ) -> FormattedText
  {
    let text = text.as_bytes();

    let mut buffer = Vec::new();

    let mut cursor_pos = gl::F32x2::ZERO;
    let tex_size = gl::F32x2::from( self.scale );

    let mut llc = gl::F32x2::MAX;
    let mut ruc = gl::F32x2::MIN;

    for i in 0..text.len() 
    {
      let c = text[ i ];
      let info = self.chars.get( &c ).unwrap();

      let mut xadnvace = info.xadvance;

      match c
      {
        // Space
        32 =>
        {
          cursor_pos += gl::F32x2::new( xadnvace, 0.0 );
          continue;
        },
        _ =>
        {
          let size = gl::F32x2::new( info.width, info.height );

          let uv = gl::F32x2::new( info.x, info.y ) / tex_size;
          let uv_extent = size / tex_size;
          let l_offset = gl::F32x2::new( info.xoffset, -info.yoffset );

          if i + 1 < text.len()
          {
            if let Some( dst_map ) = self.kernings.get( &c )
            { 
              let c_next = text[ i + 1 ]; 
              if let Some( amount ) = dst_map.get( &c_next )
              {
                xadnvace += amount;
              }
            }
          }

          llc = llc.min( cursor_pos - gl::F32x2::new( 0.0, size.y() ) + l_offset );
          ruc = ruc.max( cursor_pos + gl::F32x2::new( size.x(), 0.0 ) + l_offset );

          buffer.push
          (
            CharData
            {
              offset : gl::F32x4::from( ( l_offset, cursor_pos ) ).to_array(),
              uv_info : gl::F32x4::from( ( uv, uv_extent ) ).to_array(),
              size : size.to_array()
            }
          );
        }
      }

      cursor_pos += gl::F32x2::new( xadnvace, 0.0 );
    }

    FormattedText
    {
      chars : buffer,
      bounding_box : gl::F32x4::from( ( llc, ruc ) ) 
    }
  }
}