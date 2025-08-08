//! This module defines a `RectangularGrid` layout, which represents a rectangularly-bounded
//! area within a hexagonal grid using offset coordinates. It provides methods for iterating
//! over the coordinates within these bounds and for calculating the layout's center point in pixel space.

use crate::coordinates::{hexagonal, pixel};
use hexagonal::{Axial, Coordinate, Flat, Offset, Pointy};
use ndarray_cg::{F32x2, I32x2};
use pixel::Pixel;
use std::marker::PhantomData;

/// Represents a rectangularly-bounded area of a hexagonal grid using offset coordinates.
#[derive(Debug)]
pub struct RectangularGrid<Parity, Orientation> {
    /// The inclusive minimum and maximum offset coordinates that define the grid's boundaries.
    pub bounds: [Coordinate<Offset<Parity>, Orientation>; 2],
}

impl<Parity, Orientation> Clone for RectangularGrid<Parity, Orientation> {
    /// Clones the rectangular grid layout.
    fn clone(&self) -> Self {
        Self {
            bounds: self.bounds,
        }
    }
}

impl<Parity, Orientation> Copy for RectangularGrid<Parity, Orientation> {}

impl<Parity, Orientation> RectangularGrid<Parity, Orientation> {
    /// Creates a new `RectangularGrid` with the specified offset coordinate bounds.
    ///
    /// # Arguments
    /// * `bounds` - An array containing the minimum and maximum inclusive offset coordinates.
    ///
    /// # Returns
    /// A new `RectangularGrid` instance.
    ///
    /// # Panics
    /// Panics if the minimum coordinate is greater than the maximum coordinate on either axis.
    pub const fn new(bounds: [Coordinate<Offset<Parity>, Orientation>; 2]) -> Self {
        assert!(bounds[0].q <= bounds[1].q, "Incorrect bounds");
        assert!(bounds[0].r <= bounds[1].r, "Incorrect bounds");

        Self { bounds }
    }

    /// Returns an iterator over all coordinates contained within the rectangular grid.
    pub fn coordinates(&self) -> impl Iterator<Item = Coordinate<Offset<Parity>, Orientation>> {
        let min = self.bounds[0];
        let max = self.bounds[1];
        let current = min;

        RectangularGridIterator::<Parity, Orientation> {
            current: current.into(),
            max: max.into(),
            min: min.into(),
            _marker: PhantomData,
        }
    }
}

impl<Parity> RectangularGrid<Parity, Pointy>
where
    Coordinate<Offset<Parity>, Pointy>: Into<Coordinate<Axial, Pointy>>,
{
    /// Calculates the geometric center of the grid in pixel coordinates for a pointy-topped layout.
    pub fn center(&self) -> Pixel {
        let [min, max] = self.bounds;

        let min1: Pixel = Into::<Coordinate<Axial, Pointy>>::into(min).into();
        let min_x = if min.r + 1 <= max.r {
            let min2 = Coordinate::<Offset<Parity>, Pointy>::new(min.q, min.r + 1);
            let min2: Pixel = Into::<Coordinate<Axial, Pointy>>::into(min2).into();
            min1[0].min(min2[0])
        } else {
            min1[0]
        };
        let min_y = min1[1];

        let max1: Pixel = Into::<Coordinate<Axial, Pointy>>::into(max).into();
        let max_x = if max.r - 1 >= min.r {
            let max2 = Coordinate::<Offset<Parity>, Pointy>::new(max.q, max.r - 1);
            let max2: Pixel = Into::<Coordinate<Axial, Pointy>>::into(max2).into();
            max1[0].max(max2[0])
        } else {
            max1[0]
        };
        let max_y = max1[1];

        Pixel::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
    }
}

impl<Parity> RectangularGrid<Parity, Flat>
where
    Coordinate<Offset<Parity>, Flat>: Into<Coordinate<Axial, Flat>>,
{
    /// Calculates the geometric center of the grid in pixel coordinates for a flat-topped layout.
    pub fn center(&self) -> F32x2 {
        let [min, max] = self.bounds;

        let min1: Pixel = Into::<Coordinate<Axial, Flat>>::into(min).into();
        let min_y = if min.r + 1 <= max.r {
            let min2 = Coordinate::<Offset<Parity>, Flat>::new(min.q + 1, min.r);
            let min2: Pixel = Into::<Coordinate<Axial, Flat>>::into(min2).into();
            min1[1].min(min2[1])
        } else {
            min1[1]
        };
        let min_x = min1[0];

        let max1: Pixel = Into::<Coordinate<Axial, Flat>>::into(max).into();
        let max_y = if max.r - 1 >= min.r {
            let max2 = Coordinate::<Offset<Parity>, Flat>::new(max.q - 1, max.r);
            let max2: Pixel = Into::<Coordinate<Axial, Flat>>::into(max2).into();
            max1[1].max(max2[1])
        } else {
            max1[1]
        };
        let max_x = max1[0];

        F32x2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
    }
}

/// An iterator that traverses the coordinates within a `RectangularGrid`.
struct RectangularGridIterator<Parity, Orientation> {
    current: I32x2,
    max: I32x2,
    min: I32x2,
    _marker: PhantomData<Coordinate<Offset<Parity>, Orientation>>,
}

impl<Parity, Orientation> Iterator for RectangularGridIterator<Parity, Orientation> {
    type Item = Coordinate<Offset<Parity>, Orientation>;

    /// Advances the iterator to the next coordinate in the grid, row by row.
    fn next(&mut self) -> Option<Self::Item> {
        if self.current[1] <= self.max[1] {
            let ret = Coordinate::<Offset<_>, _>::new(self.current[0], self.current[1]);
            self.current[0] += 1;
            if self.current[0] > self.max[0] {
                self.current[0] = self.min[0];
                self.current[1] += 1;
            }
            return Some(ret);
        } else {
            None
        }
    }
}
