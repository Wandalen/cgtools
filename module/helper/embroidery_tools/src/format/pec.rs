//!
//! # PEC format reader and writer
//! 

mod private
{
  use crate::*;
  use thread::{ Thread, Color };

  /// Default PEC thread palette
  pub fn pec_threads() -> [ Thread; 65 ]
  {
    macro_rules! pec_thread
    {
      ( $r:expr, $g:expr, $b:expr, $desc:expr, $catnum:expr ) =>
      {
        Thread
        {
          color : Color { r : $r, g : $g, b : $b },
          description : $desc.into(), 
          catalog_number : $catnum.into(), 
          brand : "Brother".into(),
          chart : "Brother".into(),
          ..Default::default()
        }
      };
    }


    [
      // This one is for indicating invalid value
      Thread
      {
        color : Color { r : 0, g : 0, b : 0 },
        description : "Unknown".into(),
        catalog_number : "0".into(),
        ..Default::default()
      },
      pec_thread!( 14, 31, 124, "Prussian Blue", "1" ),
      pec_thread!( 10, 85, 163, "Blue", "2" ),
      pec_thread!( 0, 135, 119, "Teal Green", "3" ),
      pec_thread!( 75, 107, 175, "Cornflower Blue", "4" ),
      pec_thread!( 237, 23, 31, "Red", "5" ),
      pec_thread!( 209, 92, 0, "Reddish Brown", "6" ),
      pec_thread!( 145, 54, 151, "Magenta", "7" ),
      pec_thread!( 228, 154, 203, "Light Lilac", "8" ),
      pec_thread!( 145, 95, 172, "Lilac", "9" ),
      pec_thread!( 158, 214, 125, "Mint Green", "10" ),
      pec_thread!( 232, 169, 0, "Deep Gold", "11" ),
      pec_thread!( 254, 186, 53, "Orange", "12" ),
      pec_thread!( 255, 255, 0, "Yellow", "13" ),
      pec_thread!( 112, 188, 31, "Lime Green", "14" ),
      pec_thread!( 186, 152, 0, "Brass", "15" ),
      pec_thread!( 168, 168, 168, "Silver", "16" ),
      pec_thread!( 125, 111, 0, "Russet Brown", "17" ),
      pec_thread!( 255, 255, 179, "Cream Brown", "18" ),
      pec_thread!( 79, 85, 86, "Pewter", "19" ),
      pec_thread!( 0, 0, 0, "Black", "20" ),
      pec_thread!( 11, 61, 145, "Ultramarine", "21" ),
      pec_thread!( 119, 1, 118, "Royal Purple", "22" ),
      pec_thread!( 41, 49, 51, "Dark Gray", "23" ),
      pec_thread!( 42, 19, 1, "Dark Brown", "24" ),
      pec_thread!( 246, 74, 138, "Deep Rose", "25" ),
      pec_thread!( 178, 118, 36, "Light Brown", "26" ),
      pec_thread!( 252, 187, 197, "Salmon Pink", "27" ),
      pec_thread!( 254, 55, 15, "Vermilion", "28" ),
      pec_thread!( 240, 240, 240, "White", "29" ),
      pec_thread!( 106, 28, 138, "Violet", "30" ),
      pec_thread!( 168, 221, 196, "Seacrest", "31" ),
      pec_thread!( 37, 132, 187, "Sky Blue", "32" ),
      pec_thread!( 254, 179, 67, "Pumpkin", "33" ),
      pec_thread!( 255, 243, 107, "Cream Yellow", "34" ),
      pec_thread!( 208, 166, 96, "Khaki", "35" ),
      pec_thread!( 209, 84, 0, "Clay Brown", "36" ),
      pec_thread!( 102, 186, 73, "Leaf Green", "37" ),
      pec_thread!( 19, 74, 70, "Peacock Blue", "38" ),
      pec_thread!( 135, 135, 135, "Gray", "39" ),
      pec_thread!( 216, 204, 198, "Warm Gray", "40" ),
      pec_thread!( 67, 86, 7, "Dark Olive", "41" ),
      pec_thread!( 253, 217, 222, "Flesh Pink", "42" ),
      pec_thread!( 249, 147, 188, "Pink", "43" ),
      pec_thread!( 0, 56, 34, "Deep Green", "44" ),
      pec_thread!( 178, 175, 212, "Lavender", "45" ),
      pec_thread!( 104, 106, 176, "Wisteria Violet", "46" ),
      pec_thread!( 239, 227, 185, "Beige", "47" ),
      pec_thread!( 247, 56, 102, "Carmine", "48" ),
      pec_thread!( 181, 75, 100, "Amber Red", "49" ),
      pec_thread!( 19, 43, 26, "Olive Green", "50" ),
      pec_thread!( 199, 1, 86, "Dark Fuchsia", "51" ),
      pec_thread!( 254, 158, 50, "Tangerine", "52" ),
      pec_thread!( 168, 222, 235, "Light Blue", "53" ),
      pec_thread!( 0, 103, 62, "Emerald Green", "54" ),
      pec_thread!( 78, 41, 144, "Purple", "55" ),
      pec_thread!( 47, 126, 32, "Moss Green", "56" ),
      pec_thread!( 255, 204, 204, "Flesh Pink", "57" ),
      pec_thread!( 255, 217, 17, "Harvest Gold", "58" ),
      pec_thread!( 9, 91, 166, "Electric Blue", "59" ),
      pec_thread!( 240, 249, 112, "Lemon Yellow", "60" ),
      pec_thread!( 227, 243, 91, "Fresh Green", "61" ),
      pec_thread!( 255, 153, 0, "Orange", "62" ),
      pec_thread!( 255, 240, 141, "Cream Yellow", "63" ),
      pec_thread!( 255, 200, 200, "Applique", "64" ),
    ]
  }
}

crate::mod_interface!
{
  layer reader;
  layer writer;

  own use pec_threads;
}
