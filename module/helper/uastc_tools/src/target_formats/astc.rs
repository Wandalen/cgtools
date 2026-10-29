use crate::{
    bitreader::BitReaderLsb,
    bitwriter::{BitWriterLsb, BitWriterMsbRevBytes},
    uastc::{self, PlaneCount, SubsetCount, ASTC_BLOCK_SIZE, UASTC_BLOCK_SIZE},
    Result,
};

pub fn convert_block_from_uastc(bytes: [u8; UASTC_BLOCK_SIZE]) -> Result<[u8; ASTC_BLOCK_SIZE]> {
    let mut reader = BitReaderLsb::new(&bytes);

    let mode = uastc::decode_mode(&mut reader)?;

    let mut output = [0; ASTC_BLOCK_SIZE];

    let mut writer = BitWriterLsb::new(&mut output);

    if mode.id == 8 {
        let rgba = uastc::decode_mode8_rgba(&mut reader);

        // 0..=8: void-extent signature
        // 9: 0 means endpoints are UNORM16, 1 means FP16
        // 10..=11: reserved, must be 1
        writer.write_u16(12, 0b1101_1111_1100);

        // 4x 13 bits of void extent coordinates, we don't calculate
        // them yet so we set them to all 1s to get them ignored
        writer.write_u32(20, 0x000F_FFFF);
        writer.write_u32(32, 0xFFFF_FFFF);

        let (r, g, b, a) = (
            rgba[0] as u16,
            rgba[1] as u16,
            rgba[2] as u16,
            rgba[3] as u16,
        );

        writer.write_u16(16, r << 8 | r);
        writer.write_u16(16, g << 8 | g);
        writer.write_u16(16, b << 8 | b);
        writer.write_u16(16, a << 8 | a);

        return Ok(output);
    }

    uastc::skip_trans_flags(&mut reader, mode);

    let compsel = uastc::decode_compsel(&mut reader, mode);
    let pat = uastc::decode_pattern_index(&mut reader, mode)?;

    let endpoint_count = mode.endpoint_count();

    let mut quant_endpoints =
        uastc::decode_endpoints(&mut reader, mode.endpoint_range_index, endpoint_count);

    let mut invert_subset_weights = [false, false, false];

    // Invert endpoints if they would trigger blue contraction
    if mode.has_blue() {
        let endpoints_per_subset = endpoint_count / mode.subset_count as usize;
        let quant_subset_endpoints = quant_endpoints
            .chunks_exact_mut(endpoints_per_subset)
            .take(mode.subset_count as usize)
            .enumerate();
        for (subset, quant_endpoints) in quant_subset_endpoints {
            let mut endpoints = [0u8; 6];
            for (unquant, quant) in endpoints.iter_mut().zip(quant_endpoints.iter()) {
                *unquant = uastc::unquant_endpoint(*quant, mode.endpoint_range_index);
            }
            let s0 = endpoints[0] as u32 + endpoints[2] as u32 + endpoints[4] as u32;
            let s1 = endpoints[1] as u32 + endpoints[3] as u32 + endpoints[5] as u32;
            if s0 > s1 {
                invert_subset_weights[subset] = true;
                for pair in quant_endpoints.chunks_exact_mut(2) {
                    pair.swap(0, 1);
                }
            }
        }
    }

    {
        // Write block mode and config bits
        writer.write_u16(13, UASTC_TO_ASTC_BLOCK_MODE_13[mode.id as usize]);

        if let Some(pattern_astc_index_10) = get_pattern_astc_index_10(mode, pat) {
            writer.write_u16(10, pattern_astc_index_10);
            writer.write_u8(2, 0b00); // To specify that all endpoints use the same CEM
        }

        let cem = match mode.format {
            uastc::Format::Rgb => 8,
            uastc::Format::Rgba => 12,
            uastc::Format::La => 4,
        };

        writer.write_u8(4, cem);
    }

    {
        // Write endpoints
        let bise_range = BISE_RANGES[mode.endpoint_range_index as usize];
        let bit_count = bise_range.bits as usize;

        if bise_range.quints > 0 {
            for chunk in quant_endpoints.chunks(3) {
                let q_lut_id = chunk
                    .iter()
                    .rev()
                    .fold(0, |acc, qe| acc * 5 + qe.trit_quint);
                let q = ASTC_QUINT_ENCODE_LUT[q_lut_id as usize];
                writer.write_u8(bit_count, chunk.get(0).map(|qe| qe.bits).unwrap_or(0));
                writer.write_u8(3, q);
                writer.write_u8(bit_count, chunk.get(1).map(|qe| qe.bits).unwrap_or(0));
                writer.write_u8(2, q >> 3);
                writer.write_u8(bit_count, chunk.get(2).map(|qe| qe.bits).unwrap_or(0));
                writer.write_u8(2, q >> 5);
            }
        } else if bise_range.trits > 0 {
            for chunk in quant_endpoints.chunks(5) {
                let t_lut_id = chunk
                    .iter()
                    .rev()
                    .fold(0, |acc, qe| acc * 3 + qe.trit_quint);
                let t = ASTC_TRIT_ENCODE_LUT[t_lut_id as usize];
                writer.write_u8(bit_count, chunk.get(0).map(|qe| qe.bits).unwrap_or(0));
                writer.write_u8(2, t);
                writer.write_u8(bit_count, chunk.get(1).map(|qe| qe.bits).unwrap_or(0));
                writer.write_u8(2, t >> 2);
                writer.write_u8(bit_count, chunk.get(2).map(|qe| qe.bits).unwrap_or(0));
                writer.write_u8(1, t >> 4);
                writer.write_u8(bit_count, chunk.get(3).map(|qe| qe.bits).unwrap_or(0));
                writer.write_u8(2, t >> 5);
                writer.write_u8(bit_count, chunk.get(4).map(|qe| qe.bits).unwrap_or(0));
                writer.write_u8(1, t >> 7);
            }
        } else {
            let bit_count = bise_range.bits as usize;
            for qe in &quant_endpoints {
                writer.write_u8(bit_count, qe.bits);
            }
        }
    }

    {
        // Write the weights and CCS which is filled from the end
        let mut writer_rev = BitWriterMsbRevBytes::new(&mut output);

        if mode.subset_count == SubsetCount::Is1 {
            if invert_subset_weights[0] {
                let weight_consumer = |_, weight: u8| {
                    writer_rev.write_u8_rev_bits(mode.weight_bits as usize, !weight);
                };
                uastc::decode_weights(&mut reader, mode, pat, weight_consumer);
            } else {
                let weight_consumer = |_, weight: u8| {
                    writer_rev.write_u8_rev_bits(mode.weight_bits as usize, weight);
                };
                uastc::decode_weights(&mut reader, mode, pat, weight_consumer);
            };
        } else {
            let pattern = uastc::get_pattern(mode, pat);

            let weight_consumer = |i, weight: u8| {
                let texel_id = i / mode.plane_count as usize;
                let subset = pattern[texel_id] as usize;
                if invert_subset_weights[subset] {
                    writer_rev.write_u8_rev_bits(mode.weight_bits as usize, !weight);
                } else {
                    writer_rev.write_u8_rev_bits(mode.weight_bits as usize, weight);
                }
            };
            uastc::decode_weights(&mut reader, mode, pat, weight_consumer);
        }

        if mode.plane_count != PlaneCount::Is1 {
            // Weights have bits reversed, but not CCS
            writer_rev.write_u8(2, compsel as u8);
        }
    }

    Ok(output)
}

static PATTERNS_2_ASTC_INDEX_10: [u16; uastc::TOTAL_ASTC_BC7_COMMON_PARTITIONS2] = [
    28, 20, 16, 29, 91, 9, 107, 72, 149, 204, 50, 114, 496, 17, 78, 39, 252, 828, 43, 156, 116,
    210, 476, 273, 684, 359, 246, 195, 694, 524,
];

static PATTERNS_3_ASTC_INDEX_10: [u16; uastc::TOTAL_ASTC_BC7_COMMON_PARTITIONS3] =
    [260, 74, 32, 156, 183, 15, 745, 0, 335, 902, 254];

static PATTERNS_2_3_ASTC_INDEX_10: [u16; uastc::TOTAL_BC7_3_ASTC2_COMMON_PARTITIONS] = [
    36, 48, 61, 137, 161, 183, 226, 281, 302, 307, 479, 495, 593, 594, 605, 799, 812, 988, 993,
];

fn get_pattern_astc_index_10(mode: uastc::Mode, pat: u8) -> Option<u16> {
    if mode.id == 7 {
        // Mode 7 has 2 subsets, but needs 2/3 patern table
        Some(PATTERNS_2_3_ASTC_INDEX_10[pat as usize])
    } else {
        match mode.subset_count {
            SubsetCount::Is1 => None,
            SubsetCount::Is2 => Some(PATTERNS_2_ASTC_INDEX_10[pat as usize]),
            SubsetCount::Is3 => Some(PATTERNS_3_ASTC_INDEX_10[pat as usize]),
        }
    }
}

static ASTC_QUINT_ENCODE_LUT: [u8; 125] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x10, 0x11, 0x12, 0x13, 0x14, 0x18,
    0x19, 0x1A, 0x1B, 0x1C, 0x05, 0x0D, 0x15, 0x1D, 0x06, 0x20, 0x21, 0x22, 0x23, 0x24, 0x28, 0x29,
    0x2A, 0x2B, 0x2C, 0x30, 0x31, 0x32, 0x33, 0x34, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x25, 0x2D, 0x35,
    0x3D, 0x0E, 0x40, 0x41, 0x42, 0x43, 0x44, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x50, 0x51, 0x52, 0x53,
    0x54, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x45, 0x4D, 0x55, 0x5D, 0x16, 0x60, 0x61, 0x62, 0x63, 0x64,
    0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x70, 0x71, 0x72, 0x73, 0x74, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x65,
    0x6D, 0x75, 0x7D, 0x1E, 0x66, 0x67, 0x46, 0x47, 0x26, 0x6E, 0x6F, 0x4E, 0x4F, 0x2E, 0x76, 0x77,
    0x56, 0x57, 0x36, 0x7E, 0x7F, 0x5E, 0x5F, 0x3E, 0x27, 0x2F, 0x37, 0x3F, 0x1F,
];

// fn astc_decode_quint(q: u8) -> u8 {
//     let (q0, q1, q2);
//     if (q >> 1) & 0b11 == 0b11 && (q >> 5) & 0b11 == 0 {
//         q2 = (q & 1) << 2
//             | ((q >> 4) & !q & 1) << 1
//             | ((q >> 3) & !q & 1);
//         q1 = 4;
//         q0 = 4;
//     } else {
//         let c;
//         if (q >> 1) & 0b11 == 0b11 {
//             q2 = 4;
//             c = (q & 0b11000) | (((!q >> 5) & 0b11) << 1) | (q & 1);
//         } else {
//             q2 = (q >> 5) & 0b11;
//             c = q & 0b11111;
//         }
//         if c & 0b111 == 0b101 {
//             q1 = 4;
//             q0 = (c >> 3) & 0b11;
//         } else {
//             q1 = (c >> 3) & 0b11;
//             q0 = c & 0b111;
//         }
//     }
//     (q2 * 5 + q1) * 5 + q0
// }

static ASTC_TRIT_ENCODE_LUT: [u8; 243] = [
    0x00, 0x01, 0x02, 0x04, 0x05, 0x06, 0x08, 0x09, 0x0A, 0x10, 0x11, 0x12, 0x14, 0x15, 0x16, 0x18,
    0x19, 0x1A, 0x03, 0x07, 0x0B, 0x13, 0x17, 0x1B, 0x0C, 0x0D, 0x0E, 0x20, 0x21, 0x22, 0x24, 0x25,
    0x26, 0x28, 0x29, 0x2A, 0x30, 0x31, 0x32, 0x34, 0x35, 0x36, 0x38, 0x39, 0x3A, 0x23, 0x27, 0x2B,
    0x33, 0x37, 0x3B, 0x2C, 0x2D, 0x2E, 0x40, 0x41, 0x42, 0x44, 0x45, 0x46, 0x48, 0x49, 0x4A, 0x50,
    0x51, 0x52, 0x54, 0x55, 0x56, 0x58, 0x59, 0x5A, 0x43, 0x47, 0x4B, 0x53, 0x57, 0x5B, 0x4C, 0x4D,
    0x4E, 0x80, 0x81, 0x82, 0x84, 0x85, 0x86, 0x88, 0x89, 0x8A, 0x90, 0x91, 0x92, 0x94, 0x95, 0x96,
    0x98, 0x99, 0x9A, 0x83, 0x87, 0x8B, 0x93, 0x97, 0x9B, 0x8C, 0x8D, 0x8E, 0xA0, 0xA1, 0xA2, 0xA4,
    0xA5, 0xA6, 0xA8, 0xA9, 0xAA, 0xB0, 0xB1, 0xB2, 0xB4, 0xB5, 0xB6, 0xB8, 0xB9, 0xBA, 0xA3, 0xA7,
    0xAB, 0xB3, 0xB7, 0xBB, 0xAC, 0xAD, 0xAE, 0xC0, 0xC1, 0xC2, 0xC4, 0xC5, 0xC6, 0xC8, 0xC9, 0xCA,
    0xD0, 0xD1, 0xD2, 0xD4, 0xD5, 0xD6, 0xD8, 0xD9, 0xDA, 0xC3, 0xC7, 0xCB, 0xD3, 0xD7, 0xDB, 0xCC,
    0xCD, 0xCE, 0x60, 0x61, 0x62, 0x64, 0x65, 0x66, 0x68, 0x69, 0x6A, 0x70, 0x71, 0x72, 0x74, 0x75,
    0x76, 0x78, 0x79, 0x7A, 0x63, 0x67, 0x6B, 0x73, 0x77, 0x7B, 0x6C, 0x6D, 0x6E, 0xE0, 0xE1, 0xE2,
    0xE4, 0xE5, 0xE6, 0xE8, 0xE9, 0xEA, 0xF0, 0xF1, 0xF2, 0xF4, 0xF5, 0xF6, 0xF8, 0xF9, 0xFA, 0xE3,
    0xE7, 0xEB, 0xF3, 0xF7, 0xFB, 0xEC, 0xED, 0xEE, 0x1C, 0x1D, 0x1E, 0x3C, 0x3D, 0x3E, 0x5C, 0x5D,
    0x5E, 0x9C, 0x9D, 0x9E, 0xBC, 0xBD, 0xBE, 0xDC, 0xDD, 0xDE, 0x1F, 0x3F, 0x5F, 0x9F, 0xBF, 0xDF,
    0x7C, 0x7D, 0x7E,
];

// fn astc_decode_trit(t: u8) -> u8 {
//     let (t0, t1, t2, t3, t4);
//     let c;
//     if (t >> 2) & 0b111 == 0b111 {
//         c = ((t >> 5) & 0b111) << 2 | (t & 0b11);
//         t4 = 2;
//         t3 = 2;
//     } else {
//         c = t & 0b11111;
//         if (t >> 5) & 0b11 == 0b11 {
//             t4 = 2;
//             t3 = (t >> 7) & 1;
//         } else {
//             t4 = (t >> 7) & 1;
//             t3 = (t >> 5) & 0b11;
//         }
//     }
//     if c & 0b11 == 0b11 {
//         t2 = 2;
//         t1 = (c >> 4) & 1;
//         t0 = ((c >> 2) & 0b10) | ((c >> 2) & (!c >> 3) & 1);
//     } else if (c >> 2) & 0b11 == 0b11 {
//         t2 = 2;
//         t1 = 2;
//         t0 = c & 0b11;
//     } else {
//         t2 = (c >> 4) & 1;
//         t1 = (c >> 2) & 0b11;
//         t0 = (c & 0b10) | (c & (!c >> 1) & 1);
//     }
//     (((t4*3 + t3)*3 + t2)*3 + t1)*3 + t0
// }

#[derive(Clone, Copy, Debug)]
pub struct BiseCounts {
    pub bits: u8,
    pub trits: u8,
    pub quints: u8,
    pub deq_b: &'static [u8; 9],
    pub deq_c: u8,
}

#[rustfmt::skip]
pub static BISE_RANGES: [BiseCounts; 21] = [
    BiseCounts { bits: 1, trits: 0, quints: 0, deq_b: b"         ", deq_c:   0 }, //  0
    BiseCounts { bits: 0, trits: 1, quints: 0, deq_b: b"         ", deq_c:   0 }, //  1
    BiseCounts { bits: 2, trits: 0, quints: 0, deq_b: b"         ", deq_c:   0 }, //  2
    BiseCounts { bits: 0, trits: 0, quints: 1, deq_b: b"         ", deq_c:   0 }, //  3
    BiseCounts { bits: 1, trits: 1, quints: 0, deq_b: b"000000000", deq_c: 204 }, //  4
    BiseCounts { bits: 3, trits: 0, quints: 0, deq_b: b"         ", deq_c:   0 }, //  5
    BiseCounts { bits: 1, trits: 0, quints: 1, deq_b: b"000000000", deq_c: 113 }, //  6
    BiseCounts { bits: 2, trits: 1, quints: 0, deq_b: b"b000b0bb0", deq_c:  93 }, //  7
    BiseCounts { bits: 4, trits: 0, quints: 0, deq_b: b"         ", deq_c:   0 }, //  8
    BiseCounts { bits: 2, trits: 0, quints: 1, deq_b: b"b0000bb00", deq_c:  54 }, //  9
    BiseCounts { bits: 3, trits: 1, quints: 0, deq_b: b"cb000cbcb", deq_c:  44 }, // 10
    BiseCounts { bits: 5, trits: 0, quints: 0, deq_b: b"         ", deq_c:   0 }, // 11
    BiseCounts { bits: 3, trits: 0, quints: 1, deq_b: b"cb0000cbc", deq_c:  26 }, // 12
    BiseCounts { bits: 4, trits: 1, quints: 0, deq_b: b"dcb000dcb", deq_c:  22 }, // 13
    BiseCounts { bits: 6, trits: 0, quints: 0, deq_b: b"         ", deq_c:   0 }, // 14
    BiseCounts { bits: 4, trits: 0, quints: 1, deq_b: b"dcb0000dc", deq_c:  13 }, // 15
    BiseCounts { bits: 5, trits: 1, quints: 0, deq_b: b"edcb000ed", deq_c:  11 }, // 16
    BiseCounts { bits: 7, trits: 0, quints: 0, deq_b: b"         ", deq_c:   0 }, // 17
    BiseCounts { bits: 5, trits: 0, quints: 1, deq_b: b"edcb0000e", deq_c:   6 }, // 18
    BiseCounts { bits: 6, trits: 1, quints: 0, deq_b: b"fedcb000f", deq_c:   5 }, // 19
    BiseCounts { bits: 8, trits: 0, quints: 0, deq_b: b"         ", deq_c:   0 }, // 20
];

static UASTC_TO_ASTC_BLOCK_MODE_13: [u16; 20] = [
    0x0242, //  0
    0x0042, //  1
    0x0853, //  2
    0x1042, //  3
    0x0842, //  4
    0x0053, //  5
    0x0442, //  6
    0x0842, //  7
    0,      //  8
    0x0842, //  9
    0x0242, // 10
    0x0442, // 11
    0x0053, // 12
    0x0441, // 13
    0x0042, // 14
    0x0242, // 15
    0x0842, // 16
    0x0442, // 17
    0x0253, // 18
    0,      // 19
];
