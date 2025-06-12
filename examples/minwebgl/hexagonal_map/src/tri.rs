const SQRT_3 : f64 = 1.73205080;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexAxial
{
  pub q : i32,
  pub r : i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriAxial
{
  pub q : i32,
  pub r : i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point
{
  pub x : f64,
  pub y : f64,
}

#[derive(Debug, Clone, Copy)]
pub enum HexOrientation
{
  Pointy,
  Flat,
}

pub struct TriangleGrid
{
  pub hex_size: f64,
  pub orientation: HexOrientation,
}

impl TriangleGrid
{
  pub fn new(hex_size: f64, orientation: HexOrientation) -> Self {
      Self { hex_size, orientation }
  }

  /// Generate dual triangle grid from hexagonal coordinates
  pub fn hex_to_triangle_dual< I >(&self, hex_coords: I ) -> Vec<TriAxial>
  where I : Iterator< Item = HexAxial >

  {

      let mut triangles = Vec::new();

      for hex in hex_coords {
          // Each hex generates 6 triangles around its vertices
          // The triangle coordinates are based on the hex vertices
          let hex_vertices = self.get_hex_vertex_triangles(hex);
          triangles.extend(hex_vertices);
      }

      // Remove duplicates and sort
      triangles.sort_by_key(|t| (t.q, t.r));
      triangles.dedup();
      triangles
  }

  /// Get the 6 triangle coordinates that correspond to a hex's vertices
  fn get_hex_vertex_triangles(&self, hex: HexAxial) -> Vec<TriAxial> {
      // In the dual relationship, each hex vertex becomes a triangle center
      // For axial coordinates, the 6 triangles around a hex (q,r) are:
      vec![
          TriAxial { q: hex.q * 2, r: hex.r * 2 },
          TriAxial { q: hex.q * 2 + 1, r: hex.r * 2 },
          TriAxial { q: hex.q * 2 + 1, r: hex.r * 2 + 1 },
          TriAxial { q: hex.q * 2, r: hex.r * 2 + 1 },
          TriAxial { q: hex.q * 2 - 1, r: hex.r * 2 + 1 },
          TriAxial { q: hex.q * 2 - 1, r: hex.r * 2 },
      ]
  }

  /// Convert triangle axial coordinates to pixel coordinates
  pub fn triangle_to_pixel(&self, tri: TriAxial) -> Point {
      let triangle_size = self.hex_size / SQRT_3;

      match self.orientation {
          HexOrientation::Pointy => {
              let x = triangle_size * (3.0 / 2.0 * tri.q as f64);
              let y = triangle_size * (SQRT_3 / 2.0 * tri.q as f64 + SQRT_3 * tri.r as f64);
              Point { x, y }
          }
          HexOrientation::Flat => {
              let x = triangle_size * (SQRT_3 * tri.q as f64 + SQRT_3 / 2.0 * tri.r as f64);
              let y = triangle_size * (3.0 / 2.0 * tri.r as f64);
              Point { x, y }
          }
      }
  }

  /// Convert pixel coordinates to triangle axial coordinates
  pub fn pixel_to_triangle(&self, point: Point) -> TriAxial
  {
    let triangle_size = self.hex_size / SQRT_3;

    let (q, r) = match self.orientation
    {
      HexOrientation::Pointy =>
      {
        let q = (2.0 / 3.0 * point.x) / triangle_size;
        let r = (-1.0 / 3.0 * point.x + SQRT_3 / 3.0 * point.y) / triangle_size;
        (q, r)
      }
      HexOrientation::Flat =>
      {
        let q = (SQRT_3 / 3.0 * point.x - 1.0 / 3.0 * point.y) / triangle_size;
        let r = (2.0 / 3.0 * point.y) / triangle_size;
        (q, r)
      }
    };

    // Round to nearest triangle coordinates
    let rq = q.round();
    let rr = r.round();

    TriAxial
    {
      q : rq as i32,
      r : rr as i32,
    }
  }

  /// Get the pixel coordinates of a triangle's vertices
  pub fn triangle_vertices(&self, tri: TriAxial) -> [Point; 3] {
      let center = self.triangle_to_pixel(tri);
      let triangle_size = self.hex_size / SQRT_3;
      let height = triangle_size * SQRT_3 / 2.0;

      // Determine if triangle points up or down based on coordinate sum
      let points_up = (tri.q + tri.r) % 2 == 0;

      match self.orientation {
          HexOrientation::Pointy => {
              if points_up {
                  [
                      Point { x: center.x, y: center.y - 2.0 * height / 3.0 },
                      Point { x: center.x - triangle_size / 2.0, y: center.y + height / 3.0 },
                      Point { x: center.x + triangle_size / 2.0, y: center.y + height / 3.0 },
                  ]
              } else {
                  [
                      Point { x: center.x, y: center.y + 2.0 * height / 3.0 },
                      Point { x: center.x - triangle_size / 2.0, y: center.y - height / 3.0 },
                      Point { x: center.x + triangle_size / 2.0, y: center.y - height / 3.0 },
                  ]
              }
          }
          HexOrientation::Flat => {
              if points_up {
                  [
                      Point { x: center.x - 2.0 * height / 3.0, y: center.y },
                      Point { x: center.x + height / 3.0, y: center.y - triangle_size / 2.0 },
                      Point { x: center.x + height / 3.0, y: center.y + triangle_size / 2.0 },
                  ]
              } else {
                  [
                      Point { x: center.x + 2.0 * height / 3.0, y: center.y },
                      Point { x: center.x - height / 3.0, y: center.y - triangle_size / 2.0 },
                      Point { x: center.x - height / 3.0, y: center.y + triangle_size / 2.0 },
                  ]
              }
          }
      }
    }

    /// Check if a triangle points up or down
    pub fn triangle_points_up(&self, tri: TriAxial) -> bool {
        (tri.q + tri.r) % 2 == 0
    }

    /// Get neighboring triangles (triangles that share an edge)
    pub fn triangle_neighbors(&self, tri: TriAxial) -> Vec<TriAxial> {
        vec![
            TriAxial { q: tri.q + 1, r: tri.r },
            TriAxial { q: tri.q - 1, r: tri.r },
            TriAxial { q: tri.q, r: tri.r + 1 },
            TriAxial { q: tri.q, r: tri.r - 1 },
            TriAxial { q: tri.q + 1, r: tri.r - 1 },
            TriAxial { q: tri.q - 1, r: tri.r + 1 },
        ]
    }
}

impl HexAxial {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
}

impl TriAxial {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Calculate distance between two triangles
    pub fn distance(&self, other: &TriAxial) -> i32 {
        ((self.q - other.q).abs() + (self.r - other.r).abs() + (self.q + self.r - other.q - other.r).abs()) / 2
    }
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate Euclidean distance between two points
    pub fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_triangle_dual() {
        let grid = TriangleGrid::new(50.0, HexOrientation::Pointy);
        let hexes = vec![
            HexAxial::new(0, 0),
            HexAxial::new(1, 0),
        ];

        let triangles = grid.hex_to_triangle_dual(hexes.into_iter());
        assert!(!triangles.is_empty());
    }

    #[test]
    fn test_triangle_pixel_conversion() {
        let grid = TriangleGrid::new(50.0, HexOrientation::Pointy);
        let tri = TriAxial::new(2, 1);

        let pixel = grid.triangle_to_pixel(tri);
        let back_to_tri = grid.pixel_to_triangle(pixel);

        assert_eq!(tri, back_to_tri);
    }

    #[test]
    fn test_triangle_direction() {
        let tri1 = TriAxial::new(0, 0); // sum = 0 (even) -> points up
        let tri2 = TriAxial::new(1, 0); // sum = 1 (odd) -> points down

        let grid = TriangleGrid::new(50.0, HexOrientation::Pointy);
        assert!(grid.triangle_points_up(tri1));
        assert!(!grid.triangle_points_up(tri2));
    }
}
