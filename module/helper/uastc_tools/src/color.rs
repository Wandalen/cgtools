use alloc::{fmt, vec, vec::Vec};
use core::ops::{Index, IndexMut};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Color32(pub [u8; 4]);

impl Color32 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    pub fn into_rgba_bytes(data: Vec<Self>) -> Vec<u8> {
        let mut result = vec![0u8; data.len() * 4];

        for (chunk, color) in result.chunks_exact_mut(4).zip(data.into_iter()) {
            chunk.copy_from_slice(&color.0);
        }

        result
    }

    pub fn to_rgba_u32(self) -> u32 {
        u32::from_le_bytes(self.0)
    }

    #[allow(dead_code)]
    pub fn from_rgba_u32(rgba: u32) -> Self {
        Color32(rgba.to_le_bytes())
    }
}

impl fmt::Debug for Color32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:08X}", self.to_rgba_u32())
    }
}

impl Index<usize> for Color32 {
    type Output = u8;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for Color32 {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}
