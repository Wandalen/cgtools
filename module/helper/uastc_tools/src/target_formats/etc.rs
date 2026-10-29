use core::convert::TryInto;

use crate::{
    bitreader::BitReaderLsb,
    bitwriter::BitWriterLsb,
    mask,
    uastc::{self, ETC1_BLOCK_SIZE, ETC2_BLOCK_SIZE, UASTC_BLOCK_SIZE},
    Color32, Result,
};

pub fn convert_etc1_block_from_uastc(
    bytes: [u8; UASTC_BLOCK_SIZE],
) -> Result<[u8; ETC1_BLOCK_SIZE]> {
    let mut output = [0; ETC1_BLOCK_SIZE];
    convert_block_from_uastc(bytes, &mut output, None)?;
    Ok(output)
}

pub fn convert_etc2_block_from_uastc(
    bytes: [u8; UASTC_BLOCK_SIZE],
) -> Result<[u8; ETC2_BLOCK_SIZE]> {
    let mut output = [0; ETC2_BLOCK_SIZE];
    let (alpha, rgb) = output.split_at_mut(8);
    convert_block_from_uastc(
        bytes,
        rgb.try_into().unwrap(),
        Some(alpha.try_into().unwrap()),
    )?;
    Ok(output)
}

fn convert_block_from_uastc(
    bytes: [u8; UASTC_BLOCK_SIZE],
    output: &mut [u8; 8],
    alpha: Option<&mut [u8; 8]>,
) -> Result<()> {
    let mut reader = BitReaderLsb::new(&bytes);

    let mode = uastc::decode_mode(&mut reader)?;

    let mut writer = BitWriterLsb::new(output);

    if mode.id == 8 {
        if let Some(alpha) = alpha {
            let rgba = uastc::decode_mode8_rgba(&mut reader);
            write_solid_etc2_alpha_block(alpha, rgba[3]);
        } else {
            uastc::skip_mode8_rgba(&mut reader);
        }

        let trans_flags = uastc::decode_mode8_etc1_flags(&mut reader);

        if !trans_flags.etc1d {
            writer.write_u8(8, trans_flags.etc1r << 4 | trans_flags.etc1r);
            writer.write_u8(8, trans_flags.etc1g << 4 | trans_flags.etc1g);
            writer.write_u8(8, trans_flags.etc1b << 4 | trans_flags.etc1b);
        } else {
            writer.write_u8(8, trans_flags.etc1r << 3);
            writer.write_u8(8, trans_flags.etc1g << 3);
            writer.write_u8(8, trans_flags.etc1b << 3);
        }
        // codeword1 (3), codeword2 (3), diff bit, flip bit
        writer.write_u8(
            8,
            trans_flags.etc1i << 5 | trans_flags.etc1i << 2 | (trans_flags.etc1d as u8) << 1,
        );

        let selector = [0b11, 0b10, 0b00, 0b01][trans_flags.etc1s as usize];
        let s_lo = selector & 1;
        let s_hi = selector >> 1;

        writer.write_u16(16, 0u16.wrapping_sub(s_hi as u16));
        writer.write_u16(16, 0u16.wrapping_sub(s_lo as u16));

        return Ok(());
    }

    let trans_flags = uastc::decode_trans_flags(&mut reader, mode);

    let mut rgba = uastc::decode_block_to_rgba(bytes)?;

    if let Some(alpha) = alpha {
        write_etc2_alpha_block(alpha, trans_flags.etc2tm, &rgba);
    }

    if !trans_flags.etc1f {
        // Transpose to have the two subblocks in 0..8 and 8..16
        for y in 0..3 {
            for x in (y + 1)..4 {
                let a = y * 4 + x;
                let b = x * 4 + y;
                rgba.swap(a, b);
            }
        }
    }

    let color_bits = if !trans_flags.etc1d { 4 } else { 5 };
    let limit = mask!(color_bits as u32);

    let mut avg_colors = [Color32::default(); 2];
    for (subblock, avg) in rgba.chunks_exact(8).zip(avg_colors.iter_mut()) {
        let sum = subblock.iter().fold([0u16; 4], |mut acc, c| {
            for (acc, &c) in acc.iter_mut().zip(c.0.iter()) {
                *acc += c as u16;
            }
            acc
        });
        for (&sum, avg) in sum.iter().zip(avg.0.iter_mut()).take(3) {
            *avg = ((sum as u32 * limit + 1020) / (8 * 255)) as u8;
        }
    }

    let [c0, c1] = if let Some(bias) = trans_flags.etc1bias {
        [
            apply_etc1_bias(avg_colors[0], bias, limit, 0),
            apply_etc1_bias(avg_colors[1], bias, limit, 1),
        ]
    } else {
        avg_colors
    };

    let block_colors = if !trans_flags.etc1d {
        writer.write_u8(8, c0[0] << 4 | c1[0]);
        writer.write_u8(8, c0[1] << 4 | c1[1]);
        writer.write_u8(8, c0[2] << 4 | c1[2]);
        [
            apply_mod_to_base_color(color_4_to_8(c0), trans_flags.etc1i0),
            apply_mod_to_base_color(color_4_to_8(c1), trans_flags.etc1i1),
        ]
    } else {
        let d = [
            (c1[0] as i16 - c0[0] as i16).clamp(-4, 3),
            (c1[1] as i16 - c0[1] as i16).clamp(-4, 3),
            (c1[2] as i16 - c0[2] as i16).clamp(-4, 3),
        ];
        writer.write_u8(8, c0[0] << 3 | (d[0] & 0b111) as u8);
        writer.write_u8(8, c0[1] << 3 | (d[1] & 0b111) as u8);
        writer.write_u8(8, c0[2] << 3 | (d[2] & 0b111) as u8);
        let c1 = Color32::new(
            (c0[0] as i16 + d[0]) as u8,
            (c0[1] as i16 + d[1]) as u8,
            (c0[2] as i16 + d[2]) as u8,
            255,
        );
        [
            apply_mod_to_base_color(color_5_to_8(c0), trans_flags.etc1i0),
            apply_mod_to_base_color(color_5_to_8(c1), trans_flags.etc1i1),
        ]
    };

    {
        // Write codebooks, diff and flip bits
        let val = trans_flags.etc1i0 << 5
            | trans_flags.etc1i1 << 2
            | (trans_flags.etc1d as u8) << 1
            | trans_flags.etc1f as u8;
        writer.write_u8(8, val);
    }

    let mut selector = Selector::default();

    for (subblock, (rgba, block_colors)) in
        rgba.chunks_exact(8).zip(block_colors.iter()).enumerate()
    {
        const LUM_FACTORS: [i32; 3] = [108, 366, 38];
        let mut block_lums = [0; 4];
        for (block_lum, block_color) in block_lums.iter_mut().zip(block_colors.iter()) {
            *block_lum = block_color
                .0
                .iter()
                .zip(LUM_FACTORS.iter())
                .map(|(&col, &f)| col as i32 * f)
                .sum();
        }
        let block_lum_01 = (block_lums[0] + block_lums[1]) / 2;
        let block_lum_12 = (block_lums[1] + block_lums[2]) / 2;
        let block_lum_23 = (block_lums[2] + block_lums[3]) / 2;

        for (i, c) in rgba.iter().enumerate() {
            let lum: i32 =
                c.0.iter()
                    .zip(LUM_FACTORS.iter())
                    .map(|(&col, &f)| col as i32 * f)
                    .sum();
            let sel = (lum >= block_lum_01) as u8
                + (lum >= block_lum_12) as u8
                + (lum >= block_lum_23) as u8;
            let x = i & 0b11;
            let y = 2 * subblock + (i >> 2);
            if trans_flags.etc1f {
                selector.set_selector(x, y, sel);
            } else {
                selector.set_selector(y, x, sel);
            }
        }
    }

    writer.write_u32(32, u32::from_le_bytes(selector.etc1_bytes));

    Ok(())
}

fn apply_etc1_bias(mut block_color: Color32, bias: u8, limit: u32, subblock: u32) -> Color32 {
    const S_DIVS: [u8; 3] = [1, 3, 9];

    for c in 0..3 {
        #[rustfmt::skip]
        let delta: i32 = match bias {
            2 => if subblock == 1 { 0 } else if c == 0 { -1 } else { 0 },
            5 => if subblock == 1 { 0 } else if c == 1 { -1 } else { 0 },
            6 => if subblock == 1 { 0 } else if c == 2 { -1 } else { 0 },

             7 => if subblock == 1 { 0 } else if c == 0 { 1 } else { 0 },
            11 => if subblock == 1 { 0 } else if c == 1 { 1 } else { 0 },
            15 => if subblock == 1 { 0 } else if c == 2 { 1 } else { 0 },

            18 => if subblock == 1 { if c == 0 { -1 } else { 0 } } else { 0 },
            19 => if subblock == 1 { if c == 1 { -1 } else { 0 } } else { 0 },
            20 => if subblock == 1 { if c == 2 { -1 } else { 0 } } else { 0 },

            21 => if subblock == 1 { if c == 0 { 1 } else { 0 } } else { 0 },
            24 => if subblock == 1 { if c == 1 { 1 } else { 0 } } else { 0 },
             8 => if subblock == 1 { if c == 2 { 1 } else { 0 } } else { 0 },

            10 => -2,

            27 => if subblock == 1 {  0 } else { -1 },
            28 => if subblock == 1 { -1 } else {  1 },
            29 => if subblock == 1 {  1 } else {  0 },
            30 => if subblock == 1 { -1 } else {  0 },
            31 => if subblock == 1 {  0 } else {  1 },

            _ => ((bias / S_DIVS[c]) % 3) as i32 - 1,
        };

        let mut v = block_color[c] as i32;
        if v == 0 {
            if delta == -2 {
                v += 3;
            } else {
                v += delta + 1;
            }
        } else if v == limit as i32 {
            v += delta - 1;
        } else {
            v += delta;
            if (v < 0) || (v > limit as i32) {
                v = (v - delta) - delta;
            }
        }

        assert!(v >= 0);
        assert!(v <= limit as i32);

        block_color[c] = v as u8;
    }

    block_color
}

fn write_solid_etc2_alpha_block(output: &mut [u8; 8], value: u8) {
    *output = [
        value,
        // Multiplier, doesn't matter, but has to be non-zero, so choosing 1
        // Modifier table index: choosing 13 (only one with 0 in it)
        (1 << 4) | 13,
        // Weight indices, 16x 3 bits, value 4 (0b100)
        0b10010010,
        0b01001001,
        0b00100100,
        0b10010010,
        0b01001001,
        0b00100100,
    ];
}

fn write_etc2_alpha_block(output: &mut [u8; 8], etc2tm: u8, rgba: &[Color32; 16]) {
    if etc2tm == 0 {
        write_solid_etc2_alpha_block(output, 255);
    } else {
        let mut min_alpha = 255;
        let mut max_alpha = 0;

        for c in rgba.iter() {
            min_alpha = min_alpha.min(c[3]);
            max_alpha = max_alpha.max(c[3]);
        }

        if min_alpha == max_alpha {
            write_solid_etc2_alpha_block(output, min_alpha);
        } else {
            let table_index = (etc2tm & mask!(4u8)) as usize;
            let multiplier = (etc2tm >> 4) as u32;

            let mod_table = ETC2_ALPHA_MODIFIERS[table_index];

            let mod_min = mod_table[ETC2_ALPHA_MODIFIERS_MIN_INDEX];
            let mod_max = mod_table[ETC2_ALPHA_MODIFIERS_MAX_INDEX];
            let range = mod_max as i32 - mod_min as i32;

            fn lerp(a: f32, b: f32, amt: f32) -> f32 {
                a * (1.0 - amt) + b * amt
            }

            let mod_0_at_range_fraction = -(mod_min as f32) / range as f32;
            let center =
                (lerp(min_alpha as f32, max_alpha as f32, mod_0_at_range_fraction)).round() as i32;

            let mut values = [0u8; 8];
            for (val, &modifier) in values.iter_mut().zip(mod_table.iter()) {
                *val = (center + (modifier as i32 * multiplier as i32)).clamp(0, 255) as u8;
            }

            let mut selectors = 0u64;
            for (i, c) in rgba.iter().enumerate() {
                let a = c[3];
                let best_selector = values
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, &val)| (val as i32 - a as i32).abs())
                    .map(|(i, _)| i)
                    .unwrap() as u64;

                // Transpose to match ETC2 pixel order
                let x = i / 4;
                let y = i % 4;
                let id = y * 4 + x;

                selectors |= best_selector << (45 - id * 3);
            }

            output[0] = center as u8;
            // Multiplier, doesn't matter, but has to be non-zero, so choosing 1
            // Modifier table index: choosing 13 (only one with 0 in it)
            output[1] = etc2tm;

            // Weight indices, 16x 3 bits, value 4 (0b100)
            output[2..8].copy_from_slice(&selectors.to_be_bytes()[2..8]);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Selector {
    // Plain selectors (2-bits per value), one byte for each row
    selectors: [u8; 4],

    // Selectors in ETC1 format, ready to be written to an ETC1 texture
    pub etc1_bytes: [u8; 4],
}

impl Selector {
    // Returned selector value ranges from 0-3 and is a direct index into g_etc1_inten_tables.
    pub fn get_selector(&self, x: usize, y: usize) -> usize {
        assert!(x < 4);
        assert!(y < 4);

        let shift = 2 * x;
        let val = (self.selectors[y] >> shift) & 0b11;
        val as usize
    }

    pub fn set_selector(&mut self, x: usize, y: usize, val: u8) {
        assert!(x < 4);
        assert!(y < 4);
        assert!(val < 4);

        // Pack the two-bit value into the byte for the appropriate row
        let shift = 2 * x;
        self.selectors[y] &= !(0b11 << shift);
        self.selectors[y] |= val << shift;

        // Convert to ETC1 format according to the spec
        let mod_id: u8 = SELECTOR_ID_TO_ETC1[val as usize];

        // ETC1 indexes pixels from top to bottom within each column
        let pixel_id = x * 4 + y;

        // MS bit of pixel 0..8 goes to byte 1
        // MS bit of pixel 8..16 goes to byte 0
        let ms_byte_id = 1 - (pixel_id / 8);

        // LS bit of pixel 0..8 goes to byte 3
        // LS bit of pixel 8..16 goes to byte 2
        let ls_byte_id = ms_byte_id + 2;

        let bit_id = pixel_id % 8;

        self.etc1_bytes[ls_byte_id] &= !(1 << bit_id);
        self.etc1_bytes[ls_byte_id] |= (mod_id % 2) << bit_id;
        self.etc1_bytes[ms_byte_id] &= !(1 << bit_id);
        self.etc1_bytes[ms_byte_id] |= (mod_id / 2) << bit_id;
    }
}

pub(crate) fn color_5_to_8(color5: Color32) -> Color32 {
    fn extend_5_to_8(x: u8) -> u8 {
        (x << 3) | (x >> 2)
    }
    Color32::new(
        extend_5_to_8(color5[0]),
        extend_5_to_8(color5[1]),
        extend_5_to_8(color5[2]),
        255,
    )
}

pub(crate) fn color_4_to_8(color5: Color32) -> Color32 {
    fn extend_4_to_8(x: u8) -> u8 {
        (x << 4) | x
    }
    Color32::new(
        extend_4_to_8(color5[0]),
        extend_4_to_8(color5[1]),
        extend_4_to_8(color5[2]),
        255,
    )
}

pub(crate) fn apply_mod_to_base_color(base: Color32, inten: u8) -> [Color32; 4] {
    let mut colors = [Color32::default(); 4];
    for (color, &modifier) in colors.iter_mut().zip(ETC1_MODIFIERS[inten as usize].iter()) {
        *color = Color32::new(
            (base[0] as i16 + modifier).clamp(0, 255) as u8,
            (base[1] as i16 + modifier).clamp(0, 255) as u8,
            (base[2] as i16 + modifier).clamp(0, 255) as u8,
            255,
        );
    }
    colors
}

static SELECTOR_ID_TO_ETC1: [u8; 4] = [0b11, 0b10, 0b00, 0b01];

#[rustfmt::skip]
static ETC1_MODIFIERS: [[i16; 4]; 8] = [
    [   -8,  -2,  2,   8 ],
    [  -17,  -5,  5,  17 ],
    [  -29,  -9,  9,  29 ],
    [  -42, -13, 13,  42 ],
    [  -60, -18, 18,  60 ],
    [  -80, -24, 24,  80 ],
    [ -106, -33, 33, 106 ],
    [ -183, -47, 47, 183 ],
];

const ETC2_ALPHA_MODIFIERS_MIN_INDEX: usize = 3;
const ETC2_ALPHA_MODIFIERS_MAX_INDEX: usize = 7;

#[rustfmt::skip]
static ETC2_ALPHA_MODIFIERS: [[i8; 8]; 16] = [
    [ -3, -6,  -9, -15, 2, 5, 8, 14 ],
    [ -3, -7, -10, -13, 2, 6, 9, 12 ],
    [ -2, -5,  -8, -13, 1, 4, 7, 12 ],
    [ -2, -4,  -6, -13, 1, 3, 5, 12 ],
    [ -3, -6,  -8, -12, 2, 5, 7, 11 ],
    [ -3, -7,  -9, -11, 2, 6, 8, 10 ],
    [ -4, -7,  -8, -11, 3, 6, 7, 10 ],
    [ -3, -5,  -8, -11, 2, 4, 7, 10 ],
    [ -2, -6,  -8, -10, 1, 5, 7,  9 ],
    [ -2, -5,  -8, -10, 1, 4, 7,  9 ],
    [ -2, -4,  -8, -10, 1, 3, 7,  9 ],
    [ -2, -5,  -7, -10, 1, 4, 6,  9 ],
    [ -3, -4,  -7, -10, 2, 3, 6,  9 ],
    [ -1, -2,  -3, -10, 0, 1, 2,  9 ], // entry 13
    [ -4, -6,  -8,  -9, 3, 5, 7,  8 ],
    [ -3, -5,  -7,  -9, 2, 4, 6,  8 ],
];
