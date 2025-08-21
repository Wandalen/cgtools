//! Universal coordinate system conversion utilities.
//!
//! This module provides traits and utilities for converting between different
//! coordinate systems, enabling seamless interoperability across grid types.
//!
//! # Conversion Types
//!
//! - **Exact Conversions**: Perfect 1:1 mappings (e.g., Square ↔ Isometric)
//! - **Approximate Conversions**: Best-effort mappings with potential information loss
//! - **Batch Conversions**: Efficient bulk coordinate transformations
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::coordinates::{
//!     conversion::{Convert, ApproximateConvert, BatchConvertExact},
//!     square::{Coordinate as SquareCoord, FourConnected},
//!     isometric::{Coordinate as IsoCoord, Diamond},
//! };
//!
//! // Exact conversion: Square ↔ Isometric
//! let square = SquareCoord::<FourConnected>::new(3, 2);
//! let iso: IsoCoord<Diamond> = square.convert();
//! let back: SquareCoord<FourConnected> = iso.convert();
//! assert_eq!(square, back);
//!
//! // Batch conversion for performance
//! let square_coords = vec![
//!     SquareCoord::<FourConnected>::new(0, 0),
//!     SquareCoord::<FourConnected>::new(1, 1),
//!     SquareCoord::<FourConnected>::new(2, 2),
//! ];
//! let iso_coords: Vec<IsoCoord<Diamond>> = square_coords.convert_batch_exact();
//! ```


/// Trait for exact coordinate system conversions.
///
/// Provides perfect 1:1 mappings between coordinate systems where no
/// information is lost during conversion. All exact conversions are
/// bidirectional and satisfy the roundtrip property: `x.convert().convert() == x`.
///
/// # Examples
///
/// ```rust
/// use tiles_tools::coordinates::{
///     conversion::Convert,
///     square::{Coordinate as SquareCoord, FourConnected},
///     isometric::{Coordinate as IsoCoord, Diamond},
/// };
///
/// let square = SquareCoord::<FourConnected>::new(5, 3);
/// let iso: IsoCoord<Diamond> = square.convert();
/// let roundtrip: SquareCoord<FourConnected> = iso.convert();
/// assert_eq!(square, roundtrip);
/// ```
pub trait Convert<T> {
  /// Converts this coordinate to the target coordinate system.
  ///
  /// This is an exact conversion that preserves all coordinate information.
  /// The conversion is guaranteed to be reversible.
  fn convert(self) -> T;
}

/// Trait for approximate coordinate system conversions.
///
/// Provides best-effort mappings between coordinate systems where some
/// information may be lost due to fundamental differences in grid structure.
/// These conversions are typically used when exact mappings are impossible.
///
/// # Examples
///
/// ```rust
/// use tiles_tools::coordinates::{
///     conversion::ApproximateConvert,
///     hexagonal::{Coordinate as HexCoord, Axial, Pointy},
///     square::{Coordinate as SquareCoord, FourConnected},
/// };
///
/// let hex = HexCoord::<Axial, Pointy>::new(2, -1);
/// let square: SquareCoord<FourConnected> = hex.approximate_convert();
/// // Note: Reverse conversion may not yield exactly the original coordinate
/// ```
pub trait ApproximateConvert<T> {
  /// Converts this coordinate to the target coordinate system approximately.
  ///
  /// This conversion may lose information and is not guaranteed to be reversible.
  /// Use when exact conversion is not possible due to grid structure differences.
  fn approximate_convert(self) -> T;
}

/// Trait for batch exact coordinate conversions.
///
/// Provides efficient bulk conversion operations for collections of coordinates
/// using exact conversions. Implementations may optimize for performance when
/// converting many coordinates at once.
pub trait BatchConvertExact<T, U> {
  /// Converts a collection of coordinates using exact conversion.
  ///
  /// This method may be more efficient than calling `convert()` individually
  /// on each coordinate, especially for large collections.
  fn convert_batch_exact(self) -> U;
}

/// Trait for batch approximate coordinate conversions.
///
/// Provides efficient bulk conversion operations for collections of coordinates
/// using approximate conversions.
pub trait BatchConvertApproximate<T, U> {
  /// Converts a collection of coordinates using approximate conversion.
  ///
  /// This method may be more efficient than calling `approximate_convert()`
  /// individually on each coordinate, especially for large collections.
  fn convert_batch_approximate(self) -> U;
}

// =============================================================================
// Exact Conversions: Square ↔ Isometric
// =============================================================================

use crate::coordinates::square::{ Coordinate as SquareCoord, FourConnected as SquareFour, EightConnected as SquareEight };
use crate::coordinates::isometric::{ Coordinate as IsoCoord, Diamond };

/// Square to Isometric conversion (exact).
///
/// Square and isometric grids have identical logical structure - isometric
/// is simply a visual transformation of the square grid. This allows for
/// perfect 1:1 conversion with no information loss.
impl Convert<IsoCoord<Diamond>> for SquareCoord<SquareFour> {
  fn convert(self) -> IsoCoord<Diamond> {
    IsoCoord::<Diamond>::new(self.x, self.y)
  }
}

impl Convert<IsoCoord<Diamond>> for SquareCoord<SquareEight> {
  fn convert(self) -> IsoCoord<Diamond> {
    IsoCoord::<Diamond>::new(self.x, self.y)
  }
}

/// Isometric to Square conversion (exact).
impl Convert<SquareCoord<SquareFour>> for IsoCoord<Diamond> {
  fn convert(self) -> SquareCoord<SquareFour> {
    SquareCoord::<SquareFour>::new(self.x, self.y)
  }
}

impl Convert<SquareCoord<SquareEight>> for IsoCoord<Diamond> {
  fn convert(self) -> SquareCoord<SquareEight> {
    SquareCoord::<SquareEight>::new(self.x, self.y)
  }
}

// =============================================================================
// Approximate Conversions: Hexagonal ↔ Square/Isometric
// =============================================================================

use crate::coordinates::hexagonal::{ Coordinate as HexCoord, Axial, Pointy };

/// Hexagonal to Square conversion (approximate).
///
/// Converts hexagonal coordinates to the nearest square grid position.
/// Uses a mapping that attempts to preserve relative spatial relationships
/// while acknowledging that hexagonal and square grids have different
/// neighbor structures.
impl<Orientation> ApproximateConvert<SquareCoord<SquareFour>> for HexCoord<Axial, Orientation> {
  fn approximate_convert(self) -> SquareCoord<SquareFour> {
    // Map hexagonal axial coordinates to square coordinates
    // This mapping attempts to preserve the general spatial layout
    let x = self.q + (self.r / 2);
    let y = self.r;
    SquareCoord::<SquareFour>::new(x, y)
  }
}

impl<Orientation> ApproximateConvert<SquareCoord<SquareEight>> for HexCoord<Axial, Orientation> {
  fn approximate_convert(self) -> SquareCoord<SquareEight> {
    let x = self.q + (self.r / 2);
    let y = self.r;
    SquareCoord::<SquareEight>::new(x, y)
  }
}

/// Square to Hexagonal conversion (approximate).
impl<Connectivity> ApproximateConvert<HexCoord<Axial, Pointy>> for SquareCoord<Connectivity> {
  fn approximate_convert(self) -> HexCoord<Axial, Pointy> {
    // Map square coordinates to hexagonal axial coordinates
    let q = self.x - (self.y / 2);
    let r = self.y;
    HexCoord::<Axial, Pointy>::new(q, r)
  }
}

/// Hexagonal to Isometric conversion (approximate, via Square).
impl<Orientation> ApproximateConvert<IsoCoord<Diamond>> for HexCoord<Axial, Orientation> {
  fn approximate_convert(self) -> IsoCoord<Diamond> {
    // Convert hex → square → isometric
    let square: SquareCoord<SquareFour> = self.approximate_convert();
    square.convert()
  }
}

/// Isometric to Hexagonal conversion (approximate, via Square).
impl ApproximateConvert<HexCoord<Axial, Pointy>> for IsoCoord<Diamond> {
  fn approximate_convert(self) -> HexCoord<Axial, Pointy> {
    // Convert isometric → square → hex
    let square: SquareCoord<SquareFour> = self.convert();
    square.approximate_convert()
  }
}

// =============================================================================
// Approximate Conversions: Triangular ↔ Other Systems
// =============================================================================

// Currently no implementation


// =============================================================================
// Batch Conversion Implementations
// =============================================================================

/// Batch exact conversion for vectors of coordinates.
impl<T, U> BatchConvertExact<Vec<T>, Vec<U>> for Vec<T>
where
  T: Convert<U>,
{
  fn convert_batch_exact(self) -> Vec<U> {
    self.into_iter().map(|coord| coord.convert()).collect()
  }
}

/// Batch approximate conversion for vectors of coordinates.
impl<T, U> BatchConvertApproximate<Vec<T>, Vec<U>> for Vec<T>
where
  T: ApproximateConvert<U>,
{
  fn convert_batch_approximate(self) -> Vec<U> {
    self.into_iter().map(|coord| coord.approximate_convert()).collect()
  }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Converts a collection of coordinates using exact conversion.
///
/// This is a convenience function for batch conversion when you want
/// to be explicit about using exact conversion.
///
/// # Examples
///
/// ```rust
/// use tiles_tools::coordinates::{
///     conversion::convert_batch_exact,
///     square::{Coordinate as SquareCoord, FourConnected},
///     isometric::{Coordinate as IsoCoord, Diamond},
/// };
///
/// let squares = vec![
///     SquareCoord::<FourConnected>::new(1, 2),
///     SquareCoord::<FourConnected>::new(3, 4),
/// ];
/// let isometric_coords: Vec<IsoCoord<Diamond>> = convert_batch_exact(squares);
/// ```
pub fn convert_batch_exact<T, U>(coords: Vec<T>) -> Vec<U>
where
  T: Convert<U>,
{
  coords.into_iter().map(|coord| coord.convert()).collect()
}

/// Converts a collection of coordinates using approximate conversion.
///
/// This is a convenience function for batch conversion when you want
/// to be explicit about using approximate conversion.
///
/// # Examples
///
/// ```rust
/// use tiles_tools::coordinates::{
///     conversion::convert_batch_approximate,
///     hexagonal::{Coordinate as HexCoord, Axial, Pointy},
///     square::{Coordinate as SquareCoord, FourConnected},
/// };
///
/// let hex_coords = vec![
///     HexCoord::<Axial, Pointy>::new(1, -1),
///     HexCoord::<Axial, Pointy>::new(2, 0),
/// ];
/// let square_coords: Vec<SquareCoord<FourConnected>> =
///     convert_batch_approximate(hex_coords);
/// ```
pub fn convert_batch_approximate<T, U>(coords: Vec<T>) -> Vec<U>
where
  T: ApproximateConvert<U>,
{
  coords.into_iter().map(|coord| coord.approximate_convert()).collect()
}

/// Checks if a coordinate conversion preserves the roundtrip property.
///
/// This function is useful for testing and validation to ensure that
/// conversions behave as expected.
///
/// # Examples
///
/// ```rust
/// use tiles_tools::coordinates::{
///     conversion::test_roundtrip_conversion,
///     square::{Coordinate as SquareCoord, FourConnected},
///     isometric::{Coordinate as IsoCoord, Diamond},
/// };
///
/// let square = SquareCoord::<FourConnected>::new(5, 3);
/// assert!(test_roundtrip_conversion::<_, IsoCoord<Diamond>, SquareCoord<FourConnected>>(square));
/// ```
pub fn test_roundtrip_conversion<T, U, V>(original: T) -> bool
where
  T: Convert<U> + PartialEq + Clone,
  U: Convert<V>,
  V: PartialEq<T>,
{
  let converted: U = original.clone().convert();
  let roundtrip: V = converted.convert();
  roundtrip == original
}

/// Measures the conversion error for approximate conversions.
///
/// Returns the distance between the original coordinate and the result
/// of a roundtrip approximate conversion. Useful for evaluating conversion
/// quality.
pub fn measure_approximate_conversion_error<T, U>(original: T) -> f64
where
  T: ApproximateConvert<U> + Clone,
  U: ApproximateConvert<T>,
  T: Into<(i32, i32)>,
{
  let converted: U = original.clone().approximate_convert();
  let roundtrip: T = converted.approximate_convert();

  let (x1, y1) = original.into();
  let (x2, y2) = roundtrip.into();

  // Calculate Euclidean distance
  let dx = (x2 - x1) as f64;
  let dy = (y2 - y1) as f64;
  (dx * dx + dy * dy).sqrt()
}
