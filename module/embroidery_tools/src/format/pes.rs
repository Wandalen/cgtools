//!
//! # PES format reader and writer
//! 

mod private
{
  /// PES versions
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  #[ non_exhaustive ]
  pub enum PESVersion
  {
    /// Versoin #PES0001
    V1,
    /// Versoin #PES0060
    V6,    
  }
}

crate::mod_interface!
{
  layer reader;
  layer writer;

  own use PESVersion;
}
