use crate::mask;

pub struct BitWriterLsb<'a> {
    bytes: &'a mut [u8],
    bit_pos: usize,
}

impl<'a> BitWriterLsb<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes, bit_pos: 0 }
    }

    pub fn write_u8(&mut self, count: usize, v: u8) {
        assert!(count <= 8);
        self.write_u32(count, v as u32)
    }

    pub fn write_u16(&mut self, count: usize, v: u16) {
        assert!(count <= 16);
        self.write_u32(count, v as u32)
    }

    pub fn write_u32(&mut self, count: usize, v: u32) {
        assert!(count <= 32);
        let v = v & mask!(count as u32);

        let mut byte = self.bit_pos / 8;
        let mut written = 0;

        let mut trash = 0;

        {
            let bit = self.bit_pos % 8;
            let byte_val = self.bytes.get_mut(byte).unwrap_or(&mut trash);
            *byte_val |= (v << bit) as u8;
            written += 8 - bit;
            byte += 1;
        }

        self.bit_pos += count;

        loop {
            if written >= count {
                return;
            }
            let byte_val = self.bytes.get_mut(byte).unwrap_or(&mut trash);
            *byte_val |= (v >> written) as u8;
            written += 8;
            byte += 1;
        }
    }

    // TODO: check on close/drop that the buffer did not overflow
}

/// MSB bit writer which fills the output buffer from the end
/// (last byte to first byte). Optionally it can also reverse written bits.
pub struct BitWriterMsbRevBytes<'a> {
    bytes: &'a mut [u8],
    bit_pos: usize,
}

impl<'a> BitWriterMsbRevBytes<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        let bit_pos = bytes.len() * 8;
        Self { bytes, bit_pos }
    }

    pub fn write_u8_rev_bits(&mut self, count: usize, v: u8) {
        assert!(count <= 8);
        self.write_u32_rev_bits(count, v as u32)
    }

    pub fn write_u32_rev_bits(&mut self, count: usize, v: u32) {
        let v = v.reverse_bits().wrapping_shr(32 - count as u32);
        self.write_u32(count, v);
    }

    pub fn write_u8(&mut self, count: usize, v: u8) {
        assert!(count <= 8);
        self.write_u32(count, v as u32)
    }

    pub fn write_u32(&mut self, count: usize, v: u32) {
        assert!(count <= 32);

        let v = v & mask!(count as u32);

        self.bit_pos = self.bit_pos.wrapping_sub(count);

        let mut byte = self.bit_pos / 8;
        let mut written = 0;

        let mut trash = 0;

        {
            let bit = self.bit_pos % 8;
            let byte_val = self.bytes.get_mut(byte).unwrap_or(&mut trash);
            *byte_val |= (v << bit) as u8;
            written += 8 - bit;
            byte += 1;
        }

        loop {
            if written >= count {
                return;
            }
            let byte_val = self.bytes.get_mut(byte).unwrap_or(&mut trash);
            *byte_val |= (v >> written) as u8;
            written += 8;
            byte += 1;
        }
    }

    // TODO: check on close/drop that the buffer did not overflow
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns one of 16 test patterns.
    /// These are runs of 64 alternating 0 and 1 bits, with the four 16-bit
    /// segments corresponding to four least significant input bits inverted
    fn generate_test_pattern(i: u64) -> u64 {
        let pattern = 0x5555_5555_5555_5555u64;
        let segment = mask!(16u64);
        let xor_mask = (segment * ((i >> 3) & 0x1)) << 48
            | (segment * ((i >> 2) & 0x1)) << 32
            | (segment * ((i >> 1) & 0x1)) << 16
            | (segment * (i & 0x1));

        pattern ^ xor_mask
    }

    #[test]
    fn test_bitwriter_lsb() {
        // For each of these 16 pattens
        for i in 0..16 {
            let data = generate_test_pattern(i);
            let mut bytes;

            // Check that writing two numbers with all combinations of bits
            // writes the right bits into the output byte buffer
            for len in 0..32 {
                for offset in 0..32 {
                    bytes = [0; 8];
                    let mut writer = BitWriterLsb::new(&mut bytes);

                    writer.write_u32(offset, data as u32);
                    writer.write_u32(len, (data >> offset) as u32);

                    let expected = data & mask!((offset + len) as u64);
                    let actual = u64::from_le_bytes(bytes);

                    assert_eq!(
                        actual, expected,
                        "value mismatch, left: {:b}, right: {:b}, off: {}, len: {}, data: {:b}",
                        actual, expected, offset, len, data
                    );
                }
            }
        }
    }

    #[test]
    fn test_bitwriter_msb_rev_bytes_rev_bits() {
        // For each of these 16 pattens
        for i in 0..16 {
            let data = generate_test_pattern(i);
            let mut bytes;

            // Check that writing two numbers with all combinations of bits
            // writes the right bits into the output byte buffer
            for len in 0..32 {
                for offset in 0..32 {
                    bytes = [0; 8];
                    let mut writer = BitWriterMsbRevBytes::new(&mut bytes);

                    writer.write_u32_rev_bits(offset, data as u32);
                    writer.write_u32_rev_bits(len, (data >> offset) as u32);

                    let expected = data & mask!((offset + len) as u64);
                    let actual = u64::from_le_bytes(bytes).reverse_bits();

                    assert_eq!(
                        actual, expected,
                        "value mismatch, left: {:b}, right: {:b}, off: {}, len: {}, data: {:b}",
                        actual, expected, offset, len, data
                    );
                }
            }
        }
    }

    #[test]
    fn test_bitwriter_msb_rev_bytes() {
        // For each of these 16 pattens
        for i in 0..16 {
            let data = generate_test_pattern(i);
            let mut bytes;

            // Check that writing two numbers with all combinations of bits
            // writes the right bits into the output byte buffer
            for len in 0..32 {
                for offset in 0..32 {
                    bytes = [0; 8];
                    let mut writer = BitWriterMsbRevBytes::new(&mut bytes);

                    writer.write_u32(offset, data.wrapping_shr((64 - offset) as u32) as u32);
                    writer.write_u32(len, data.wrapping_shr((64 - offset - len) as u32) as u32);

                    let expected = data & !mask!((64 - offset - len) as u64);
                    let actual = u64::from_le_bytes(bytes);

                    assert_eq!(
                        actual, expected,
                        "value mismatch, left: {:b}, right: {:b}, off: {}, len: {}, data: {:b}",
                        actual, expected, offset, len, data
                    );
                }
            }
        }
    }
}
