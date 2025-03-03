//!
//! # Embroidery thread representation
//!

mod private
{
  use crate::format::pec;
  use itertools::Itertools as _;
  use rand::seq::SliceRandom as _;
  use serde::{Deserialize, Serialize};
  use std::borrow::Cow;

  #[derive(Debug, Serialize, Deserialize)]
  pub struct SerThread {
    pub hex_color: u32,
    pub description: Option<String>,
    pub catalog_number: Option<String>,
    pub details: Option<String>,
    pub brand: Option<String>,
    pub chart: Option<String>,
    pub weight: Option<String>,
  }

  impl From<Thread> for SerThread
  {
    fn from(value: Thread) -> Self
    {
      // println!("{:?}", value.color);
      let red = (value.color.r as u32) << 16;
      let green = (value.color.g as u32) << 8;
      let blue = value.color.b as u32;
      let mut color = 0xFF_FF_FF_FF;
      color &= red | green | blue;

      // println!("{}", color);
      Self
      {
        hex_color: color,
        description: Some(value.description.into()),
        catalog_number: Some(value.catalog_number.into()),
        details: Some(value.details.into()),
        brand: Some(value.brand.into()),
        chart: Some(value.chart.into()),
        weight: Some(value.weight.into()),
      }
    }
  }

  /// RGB color
  #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct Color
  {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
  }

  /// General Thread structure for storing information about threads
  /// used in embroidery file. Not all fields may be used. Depends on a format
  #[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
  pub struct Thread {
    /// Color of thread
    pub color: Color,
    /// Thread description, almost always it is shade name
    pub description: Cow<'static, str>,
    /// A number in thread catalog
    pub catalog_number: Cow<'static, str>,
    /// Some additional description
    pub details: Cow<'static, str>,
    /// Brand name
    pub brand: Cow<'static, str>,
    /// Chart name
    pub chart: Cow<'static, str>,
    /// Weight of thread
    pub weight: Cow<'static, str>,
  }

  impl From<SerThread> for Thread {
    fn from(value: SerThread) -> Self {
      let SerThread {
          hex_color,
          description,
          catalog_number,
          details,
          brand,
          chart,
          weight,
      } = value;
      let r = (hex_color >> 16 & 0xFF) as u8;
      let g = (hex_color >> 8 & 0xFF) as u8;
      let b = (hex_color & 0xFF) as u8;

      Self {
          color: Color { r, g, b },
          description: description.unwrap_or_default().into(),
          catalog_number: catalog_number.unwrap_or_default().into(),
          details: details.unwrap_or_default().into(),
          brand: brand.unwrap_or_default().into(),
          chart: chart.unwrap_or_default().into(),
          weight: weight.unwrap_or_default().into(),
      }
    }
  }

  /// Takes unique colors from `threadlist` and maps them by finding closest colors from `palette` for each unique color.
  /// # Returns
  /// Indices into `palette` for every color in `threadlist`
  pub fn build_unique_palette(palette: &[Thread], threadlist: &[Thread]) -> Vec<usize> {
    let mut chart = vec![None; palette.len()];
    let mut palette: Vec<_> = palette.iter().map(Some).collect();

    for thread in threadlist.iter().unique() {
      let index = find_nearest_color(&thread.color, &palette);
      if let Some(index) = index {
          palette[index] = None;
          chart[index] = Some(thread);
      } else {
          break;
      }
    }

    let mut palette = vec![];
    for thread in threadlist {
      palette.push(find_nearest_color(&thread.color, &chart).unwrap());
    }

    palette
  }

  /// Finds index of closest color in palette.
  /// # Returns
  /// `None` if palette consists only of `None` values,
  /// otherwise returns index of closest color
  pub fn find_nearest_color(color: &Color, palette: &[Option<&Thread>]) -> Option<usize> {
    let mut closest_index = None;
    let mut current_distance = i32::MAX;

    for (i, thread) in palette.iter().enumerate() {
      if let Some(thread) = thread {
        let dist = color_distance_red_mean(color, &thread.color);
        if dist <= current_distance {
          current_distance = dist;
          closest_index = Some(i);
        }
      }
    }

    closest_index
  }

  /// Calculates distance between colors
  pub fn color_distance_red_mean(color1: &Color, color2: &Color) -> i32 {
    // See the very good color distance paper:
    // https://www.compuphase.com/cmetric.htm

    let red_mean = (color1.r as i32 + color2.r as i32) / 2;
    let r = color1.r as i32 - color2.r as i32;
    let g = color1.g as i32 - color2.g as i32;
    let b = color1.b as i32 - color2.b as i32;

    (((512 + red_mean) * r * r) >> 8) + 4 * g * g + (((767 - red_mean) * b * b) >> 8)
  }

  /// Retrieves a random thread from PEC pallete
  pub fn get_random_thread() -> Thread {
    pec::pec_threads()[1..]
      .choose(&mut rand::thread_rng())
      .unwrap()
      .clone()
  }
}

crate::mod_interface!
{
  own use Thread;
  own use SerThread;
  own use Color;
  own use build_unique_palette;
  own use find_nearest_color;
  own use color_distance_red_mean;
  own use get_random_thread;
}
