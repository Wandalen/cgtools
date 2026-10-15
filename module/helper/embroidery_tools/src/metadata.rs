//!
//! # Represents different metadata from embroidery files
//! 

mod private
{
  use crate::thread::Thread;
  use std::collections::HashMap;

  /// Struct for storing different metadata that could possibly be encountered in embroidery formats
  #[ derive( Debug, Clone ) ]
  pub struct Metadata
  {
    // name of design
    name : Option< String >,
    // metadata in text form
    text : HashMap< String, String >,
    // some graphic metadata
    graphics : HashMap< String, Graphics >,
  }

  impl Metadata
  {
    /// Creates new `Metadata` instance
    #[ must_use ]
    #[ inline ]
    pub fn new() -> Self
    {
      Self { name : None, text : HashMap::new(), graphics : HashMap::new() }
    }

    /// Returns design name
    #[ must_use ]
    #[ inline ]
    pub fn name_get( &self ) -> Option< &str >
    {
      self.name.as_deref()
    }

    /// Sets design name
    #[ inline ]
    pub fn name_set( &mut self, name : Option< String > )
    {
      self.name = name;
    }

    /// Returns text data stored by `key`
    #[ must_use ]
    #[ inline ]
    pub fn text_get( &self, key : &str ) -> Option< &str >
    {
      self.text.get( key ).map( String::as_str )
    }

    /// Inserts text data by `key`
    #[ inline ]
    pub fn text_insert( &mut self, key : &str, value : String )
    {
      _ = self.text.insert( key.into(), value );
    }

    /// Inserts graphics data by `key`
    #[ inline ]
    pub fn graphics_insert( &mut self, key : &str, graphics : Graphics )
    {
      _ = self.graphics.insert( key.into(), graphics );
    }
  }

  impl Default for Metadata
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }

  /// Graphics data that could be encountered in embroidery formats.
  /// For now it is only PEC preview thumbnails images
  #[ derive( Debug, Clone ) ]
  #[ non_exhaustive ]
  pub enum Graphics
  {
    /// PEC thumbnail preview image
    PecGraphics
    {
      /// Each byte is 8 pixels
      image : Vec< u8 >,
      /// Width in bytes, actual width is `stride * 8`  
      stride : u8,
      /// Thread that corresponds to this image
      thread : Option< Thread >,
    }
  }
}

crate::mod_interface!
{
  own use Metadata;
  own use Graphics;
}
