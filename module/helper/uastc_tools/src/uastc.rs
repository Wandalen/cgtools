use alloc::{string::String, vec, vec::Vec};
use core::convert::TryInto;

use crate::{
    bitreader::BitReaderLsb,
    target_formats::{astc, bc7, etc},
    Color32, Result,
};

const MAX_ENDPOINT_COUNT: usize = 18;
const MAX_WEIGHT_COUNT: usize = 32;

pub const UASTC_BLOCK_SIZE: usize = 16;
pub const ASTC_BLOCK_SIZE: usize = 16;
pub const ETC1_BLOCK_SIZE: usize = 8;
pub const ETC2_BLOCK_SIZE: usize = 16;
pub const BC7_BLOCK_SIZE: usize = 16;

#[derive(Clone, Copy, Debug)]
pub struct Mode8Etc1Flags {
    pub etc1d: bool, // 1 bit
    pub etc1i: u8,   // 3 bits
    pub etc1s: u8,   // 2 bits
    pub etc1r: u8,   // 5 bits
    pub etc1g: u8,   // 5 bits
    pub etc1b: u8,   // 5 bits
}

#[derive(Clone, Copy, Debug)]
pub struct TranscodingFlags {
    pub bc1h0: bool,          // 1 bit
    pub bc1h1: bool,          // 1 bit
    pub etc1f: bool,          // 1 bit
    pub etc1d: bool,          // 1 bit
    pub etc1i0: u8,           // 3 bits
    pub etc1i1: u8,           // 3 bits
    pub etc1bias: Option<u8>, // 5 bits
    pub etc2tm: u8,           // 8 bits
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetTextureFormat {
    Astc,
    Bc7,
    Etc1,
    Etc2,
}

struct Blocks<'a> {
    data: &'a [u8],
}

impl<'a> Blocks<'a> {
    fn new(data: &'a [u8]) -> Result<Self> {
        if data.len() % UASTC_BLOCK_SIZE != 0 {
            return Err("data length is not divisible by UASTC block size (16)".into());
        }
        Ok(Self { data })
    }

    fn len(&self) -> usize {
        self.data.len() / UASTC_BLOCK_SIZE
    }

    fn iter(&self) -> impl Iterator<Item = [u8; UASTC_BLOCK_SIZE]> + '_ {
        block_iter(self.data)
    }

    fn zip_with_output<'s, 'b: 's, const N: usize>(
        &'s self,
        output: &'b mut [u8],
    ) -> impl Iterator<Item = ([u8; UASTC_BLOCK_SIZE], &'b mut [u8; N])> + 's {
        core::iter::zip(self.iter(), block_iter_mut(output))
    }
}

pub struct Decoder {}

impl Decoder {
    pub(crate) fn new() -> Self {
        // TODO: LUTs
        Self {}
    }

    pub(crate) fn read_to_uastc(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    pub(crate) fn decode_to_rgba(
        &self,
        data: &[u8],
        blocks_per_row: usize,
    ) -> Result<Vec<Color32>> {
        let blocks = Blocks::new(data)?;
        let mut data = vec![Color32::default(); blocks.len() * 16];
        let stride = 4 * blocks_per_row;

        for (i, block) in blocks.iter().enumerate() {
            let rgba = decode_block_to_rgba(block)?;
            let block_x = i % blocks_per_row;
            let block_y = i / blocks_per_row;
            for y in 0..4 {
                let x_start = 4 * block_x;
                let image_start = (4 * block_y + y) * stride + x_start;
                data[image_start..image_start + 4].copy_from_slice(&rgba[4 * y..4 * y + 4]);
            }
        }

        Ok(data)
    }

    pub fn transcode(&self, format: TargetTextureFormat, data: &[u8]) -> Result<Vec<u8>> {
        let blocks = Blocks::new(data)?;

        match format {
            TargetTextureFormat::Astc => transcode(blocks, astc::convert_block_from_uastc),
            TargetTextureFormat::Bc7 => transcode(blocks, bc7::convert_block_from_uastc),
            TargetTextureFormat::Etc1 => transcode(blocks, etc::convert_etc1_block_from_uastc),
            TargetTextureFormat::Etc2 => transcode(blocks, etc::convert_etc2_block_from_uastc),
        }
    }

    pub fn _transcode_into(
        &self,
        format: TargetTextureFormat,
        data: &[u8],
        output: &mut [u8],
    ) -> Result<()> {
        let blocks = Blocks::new(data)?;

        match format {
            TargetTextureFormat::Astc => {
                transcode_into(blocks, output, astc::convert_block_from_uastc)
            }
            TargetTextureFormat::Bc7 => {
                transcode_into(blocks, output, bc7::convert_block_from_uastc)
            }
            TargetTextureFormat::Etc1 => {
                transcode_into(blocks, output, etc::convert_etc1_block_from_uastc)
            }
            TargetTextureFormat::Etc2 => {
                transcode_into(blocks, output, etc::convert_etc2_block_from_uastc)
            }
        }
    }
}

fn transcode<const N: usize, F>(blocks: Blocks, f: F) -> Result<Vec<u8>>
where
    F: Fn([u8; UASTC_BLOCK_SIZE]) -> Result<[u8; N]>,
{
    let mut output = vec![0u8; blocks.len() * N];
    transcode_into(blocks, &mut output, f)?;
    Ok(output)
}

fn transcode_into<const N: usize, F>(blocks: Blocks, output: &mut [u8], f: F) -> Result<()>
where
    F: Fn([u8; UASTC_BLOCK_SIZE]) -> Result<[u8; N]>,
{
    for (block, output) in blocks.zip_with_output(output) {
        *output = f(block)?;
    }
    Ok(())
}

fn block_iter<const N: usize>(data: &[u8]) -> impl Iterator<Item = [u8; N]> + '_ {
    data.chunks_exact(N)
        .map(|c| TryInto::<[u8; N]>::try_into(c).unwrap())
}

fn block_iter_mut<const N: usize>(data: &mut [u8]) -> impl Iterator<Item = &mut [u8; N]> {
    data.chunks_exact_mut(N).map(|c| c.try_into().unwrap())
}

pub(crate) fn assemble_endpoint_pairs(mode: Mode, endpoint_bytes: &[u8]) -> [[Color32; 2]; 3] {
    let mut endpoint_pairs = [[Color32::default(); 2]; 3];

    match mode.format {
        Format::Rgb => {
            for (pair, bytes) in endpoint_pairs
                .iter_mut()
                .zip(endpoint_bytes.chunks_exact(6))
            {
                *pair = [
                    Color32::new(bytes[0], bytes[2], bytes[4], 0xFF),
                    Color32::new(bytes[1], bytes[3], bytes[5], 0xFF),
                ];
            }
        }
        Format::Rgba => {
            for (pair, bytes) in endpoint_pairs
                .iter_mut()
                .zip(endpoint_bytes.chunks_exact(8))
            {
                *pair = [
                    Color32::new(bytes[0], bytes[2], bytes[4], bytes[6]),
                    Color32::new(bytes[1], bytes[3], bytes[5], bytes[7]),
                ];
            }
        }
        Format::La => {
            for (pair, bytes) in endpoint_pairs
                .iter_mut()
                .zip(endpoint_bytes.chunks_exact(4))
            {
                *pair = [
                    Color32::new(bytes[0], bytes[0], bytes[0], bytes[2]),
                    Color32::new(bytes[1], bytes[1], bytes[1], bytes[3]),
                ];
            }
        }
    }

    endpoint_pairs
}

fn astc_interpolate(l: u8, h: u8, w: u8, srgb: bool) -> u8 {
    let mut l = l as u32;
    let mut h = h as u32;
    let w = w as u32;

    if srgb {
        l = (l << 8) | 0x80;
        h = (h << 8) | 0x80;
    } else {
        l = (l << 8) | l;
        h = (h << 8) | h;
    }

    // round(lerp(l, h, w/64.0))
    let k = (l * (64 - w) + h * w + 32) >> 6;

    (k >> 8) as u8
}

pub(crate) fn decode_block_to_rgba(bytes: [u8; UASTC_BLOCK_SIZE]) -> Result<[Color32; 16]> {
    let reader = &mut BitReaderLsb::new(&bytes);

    let mode = decode_mode(reader)?;

    if mode.id == 8 {
        return Ok([decode_mode8_rgba(reader); 16]);
    }

    skip_trans_flags(reader, mode);

    // Component selector for dual-plane modes
    let compsel = decode_compsel(reader, mode);

    // Pattern id for modes with multiple subsets
    let pat = decode_pattern_index(reader, mode)?;

    let endpoint_count = mode.endpoint_count();
    let weight_count = mode.weight_count();

    let endpoints = &mut [0u8; MAX_ENDPOINT_COUNT][..endpoint_count];
    let weights = &mut [0u8; MAX_WEIGHT_COUNT][..weight_count];

    let quant_endpoints = decode_endpoints(reader, mode.endpoint_range_index, endpoints.len());
    for (quant, unquant) in quant_endpoints.iter().zip(endpoints.iter_mut()) {
        *unquant = unquant_endpoint(*quant, mode.endpoint_range_index);
    }

    let weight_consumer = |i, weight| {
        weights[i] = weight;
    };
    decode_weights(reader, mode, pat, weight_consumer);
    unquant_weights(weights, mode.weight_bits);

    let srgb = false;
    let mut output = [Color32::default(); 16];

    if mode.subset_count == SubsetCount::Is1 {
        let [e0, e1] = assemble_endpoint_pairs(mode, endpoints)[0];

        match mode.plane_count {
            PlaneCount::Is1 => {
                assert_eq!(weights.len(), 16);
                for (&w, out) in weights.iter().zip(output.iter_mut()) {
                    *out = Color32::new(
                        astc_interpolate(e0[0], e1[0], w, srgb),
                        astc_interpolate(e0[1], e1[1], w, srgb),
                        astc_interpolate(e0[2], e1[2], w, srgb),
                        astc_interpolate(e0[3], e1[3], w, false),
                    );
                }
            }
            PlaneCount::Is2 => {
                assert_eq!(weights.len(), 32);
                for (ws, out) in weights.chunks_exact(2).zip(output.iter_mut()) {
                    assert_eq!(ws.len(), 2);
                    let wr = if compsel == CompSel::R { ws[1] } else { ws[0] };
                    let wg = if compsel == CompSel::G { ws[1] } else { ws[0] };
                    let wb = if compsel == CompSel::B { ws[1] } else { ws[0] };
                    let wa = if compsel == CompSel::A { ws[1] } else { ws[0] };

                    *out = Color32::new(
                        astc_interpolate(e0[0], e1[0], wr, srgb),
                        astc_interpolate(e0[1], e1[1], wg, srgb),
                        astc_interpolate(e0[2], e1[2], wb, srgb),
                        astc_interpolate(e0[3], e1[3], wa, false),
                    );
                }
            }
        }
    } else {
        let e = assemble_endpoint_pairs(mode, endpoints);

        let pattern = get_pattern(mode, pat);

        assert_eq!(weights.len(), 16);

        for ((&subset, &w), out) in pattern.iter().zip(weights.iter()).zip(output.iter_mut()) {
            let [e0, e1] = e[subset as usize];

            *out = Color32::new(
                astc_interpolate(e0[0], e1[0], w, srgb),
                astc_interpolate(e0[1], e1[1], w, srgb),
                astc_interpolate(e0[2], e1[2], w, srgb),
                astc_interpolate(e0[3], e1[3], w, false),
            );
        }
    }

    Ok(output)
}

pub fn decode_mode(reader: &mut BitReaderLsb) -> Result<Mode> {
    let mode_code = reader.peek(7) as usize;
    let mode_index = MODE_LUT[mode_code] as usize;

    let mode = MODES
        .get(mode_index)
        .copied()
        .ok_or_else(|| String::from("invalid mode index"))?;

    reader.remove(mode.code_size as usize);

    Ok(mode)
}

pub fn decode_compsel(reader: &mut BitReaderLsb, mode: Mode) -> CompSel {
    match (mode.plane_count, mode.format) {
        // LA modes always have component selector 3 for alpha
        (PlaneCount::Is2, Format::La) => CompSel::A,
        (PlaneCount::Is2, _) => CompSel::from_u2(reader.read_u8(2)),
        _ => CompSel::R,
    }
}

pub fn decode_pattern_index(reader: &mut BitReaderLsb, mode: Mode) -> Result<u8> {
    let (pattern_index, pattern_count) = match (mode.id, mode.subset_count) {
        (7, _) => (reader.read_u8(5), TOTAL_BC7_3_ASTC2_COMMON_PARTITIONS),
        (_, SubsetCount::Is1) => return Ok(0),
        (_, SubsetCount::Is2) => (reader.read_u8(5), TOTAL_ASTC_BC7_COMMON_PARTITIONS2),
        (_, SubsetCount::Is3) => (reader.read_u8(4), TOTAL_ASTC_BC7_COMMON_PARTITIONS3),
    };

    // Check pattern bounds
    if (pattern_index as usize) < pattern_count {
        Ok(pattern_index)
    } else {
        Err("block pattern is not valid".into())
    }
}

pub fn get_pattern(mode: Mode, pat: u8) -> &'static [u8; 16] {
    match (mode.id, mode.subset_count) {
        // Mode 7 has 2 subsets, but needs 2/3 patern table
        (7, _) => &PATTERNS_2_3[pat as usize],
        (_, SubsetCount::Is1) => &[0; 16],
        (_, SubsetCount::Is2) => &PATTERNS_2[pat as usize],
        (_, SubsetCount::Is3) => &PATTERNS_3[pat as usize],
    }
}

fn get_anchor_weight_indices(mode: Mode, pat: u8) -> &'static [u8] {
    match (mode.id, mode.subset_count) {
        (7, _) => &PATTERNS_2_3_ANCHORS[pat as usize],
        (_, SubsetCount::Is1) => &[0],
        (_, SubsetCount::Is2) => &PATTERNS_2_ANCHORS[pat as usize],
        (_, SubsetCount::Is3) => &PATTERNS_3_ANCHORS[pat as usize],
    }
}

pub(crate) fn decode_mode8_rgba(reader: &mut BitReaderLsb) -> Color32 {
    Color32::new(
        reader.read_u8(8), // R
        reader.read_u8(8), // G
        reader.read_u8(8), // B
        reader.read_u8(8), // A
    )
}

pub fn skip_mode8_rgba(reader: &mut BitReaderLsb) {
    reader.remove(32);
}

pub fn decode_mode8_etc1_flags(reader: &mut BitReaderLsb) -> Mode8Etc1Flags {
    Mode8Etc1Flags {
        etc1d: reader.read_bool(),
        etc1i: reader.read_u8(3),
        etc1s: reader.read_u8(2),
        etc1r: reader.read_u8(5),
        etc1g: reader.read_u8(5),
        etc1b: reader.read_u8(5),
    }
}

pub fn decode_trans_flags(reader: &mut BitReaderLsb, mode: Mode) -> TranscodingFlags {
    assert_ne!(mode.id, 8);

    TranscodingFlags {
        bc1h0: reader.read_bool(),
        bc1h1: if (10..=12).contains(&mode.id) {
            false
        } else {
            reader.read_bool()
        },
        etc1f: reader.read_bool(),
        etc1d: reader.read_bool(),
        etc1i0: reader.read_u8(3),
        etc1i1: reader.read_u8(3),
        etc1bias: if (10..=12).contains(&mode.id) {
            None
        } else {
            Some(reader.read_u8(5))
        },
        etc2tm: if mode.has_alpha() {
            reader.read_u8(8)
        } else {
            0
        },
    }
}

pub fn skip_trans_flags(reader: &mut BitReaderLsb, mode: Mode) {
    assert_ne!(mode.id, 8);
    reader.remove(mode.trans_flags_bits as usize);
}

#[derive(Clone, Copy, Debug)]
pub struct Mode {
    pub id: u8,
    code_size: u8,
    pub endpoint_range_index: u8,
    pub format: Format,
    pub weight_bits: u8,
    pub plane_count: PlaneCount,
    pub subset_count: SubsetCount,
    trans_flags_bits: u8,
}

impl Mode {
    pub fn has_alpha(&self) -> bool {
        match self.format {
            Format::Rgb => false,
            Format::Rgba | Format::La => true,
        }
    }

    pub fn has_blue(&self) -> bool {
        match self.format {
            Format::Rgb | Format::Rgba => true,
            Format::La => false,
        }
    }

    pub fn channel_count(&self) -> usize {
        match self.format {
            Format::Rgb => 3,
            Format::Rgba => 4,
            Format::La => 2,
        }
    }

    pub fn endpoint_count(&self) -> usize {
        self.channel_count() * self.subset_count as usize * 2
    }

    pub fn weight_count(&self) -> usize {
        self.plane_count as usize * 16
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Rgb,
    Rgba,
    La,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneCount {
    Is1 = 1,
    Is2 = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubsetCount {
    Is1 = 1,
    Is2 = 2,
    Is3 = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompSel {
    R = 0,
    G = 1,
    B = 2,
    A = 3,
}

impl CompSel {
    fn from_u2(val: u8) -> CompSel {
        match val {
            0 => Self::R,
            1 => Self::G,
            2 => Self::B,
            3 => Self::A,
            _ => unreachable!(),
        }
    }
}

#[rustfmt::skip]
static MODES: [Mode; 19] = [
    // RGB
    Mode { id:  0, code_size: 4, endpoint_range_index: 19, format: Format::Rgb,  weight_bits: 4, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits: 15 },
    Mode { id:  1, code_size: 6, endpoint_range_index: 20, format: Format::Rgb,  weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits: 15 },
    Mode { id:  2, code_size: 5, endpoint_range_index:  8, format: Format::Rgb,  weight_bits: 3, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is2, trans_flags_bits: 15 },
    Mode { id:  3, code_size: 5, endpoint_range_index:  7, format: Format::Rgb,  weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is3, trans_flags_bits: 15 },
    Mode { id:  4, code_size: 5, endpoint_range_index: 12, format: Format::Rgb,  weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is2, trans_flags_bits: 15 },
    Mode { id:  5, code_size: 5, endpoint_range_index: 20, format: Format::Rgb,  weight_bits: 3, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits: 15 },
    Mode { id:  6, code_size: 5, endpoint_range_index: 18, format: Format::Rgb,  weight_bits: 2, plane_count: PlaneCount::Is2, subset_count: SubsetCount::Is1, trans_flags_bits: 15 },
    Mode { id:  7, code_size: 5, endpoint_range_index: 12, format: Format::Rgb,  weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is2, trans_flags_bits: 15 },

    // Void-Extent (RGBA)
    Mode { id:  8, code_size: 5, endpoint_range_index:  0, format: Format::Rgba, weight_bits: 0, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits:  0 },

    // RGBA
    Mode { id:  9, code_size: 5, endpoint_range_index:  8, format: Format::Rgba, weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is2, trans_flags_bits: 23 },
    Mode { id: 10, code_size: 3, endpoint_range_index: 13, format: Format::Rgba, weight_bits: 4, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits: 17 },
    Mode { id: 11, code_size: 2, endpoint_range_index: 13, format: Format::Rgba, weight_bits: 2, plane_count: PlaneCount::Is2, subset_count: SubsetCount::Is1, trans_flags_bits: 17 },
    Mode { id: 12, code_size: 3, endpoint_range_index: 19, format: Format::Rgba, weight_bits: 3, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits: 17 },
    Mode { id: 13, code_size: 5, endpoint_range_index: 20, format: Format::Rgba, weight_bits: 1, plane_count: PlaneCount::Is2, subset_count: SubsetCount::Is1, trans_flags_bits: 23 },
    Mode { id: 14, code_size: 5, endpoint_range_index: 20, format: Format::Rgba, weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits: 23 },

    // LA
    Mode { id: 15, code_size: 7, endpoint_range_index: 20, format: Format::La,   weight_bits: 4, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits: 23 },
    Mode { id: 16, code_size: 6, endpoint_range_index: 20, format: Format::La,   weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is2, trans_flags_bits: 23 },
    Mode { id: 17, code_size: 6, endpoint_range_index: 20, format: Format::La,   weight_bits: 2, plane_count: PlaneCount::Is2, subset_count: SubsetCount::Is1, trans_flags_bits: 23 },

    // RGB
    Mode { id: 18, code_size: 4, endpoint_range_index: 11, format: Format::Rgb,  weight_bits: 5, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, trans_flags_bits: 15 },
];

#[rustfmt::skip]
static MODE_LUT: [u8; 128] = [
    11,  0, 10, 3, 11, 15, 12,  7,
    11, 18, 10, 5, 11, 14, 12,  9,
    11,  0, 10, 4, 11, 16, 12,  8,
    11, 18, 10, 6, 11,  2, 12, 13,
    11,  0, 10, 3, 11, 17, 12,  7,
    11, 18, 10, 5, 11, 14, 12,  9,
    11,  0, 10, 4, 11,  1, 12,  8,
    11, 18, 10, 6, 11,  2, 12, 13,
    11,  0, 10, 3, 11, 19, 12,  7,
    11, 18, 10, 5, 11, 14, 12,  9,
    11,  0, 10, 4, 11, 16, 12,  8,
    11, 18, 10, 6, 11,  2, 12, 13,
    11,  0, 10, 3, 11, 17, 12,  7,
    11, 18, 10, 5, 11, 14, 12,  9,
    11,  0, 10, 4, 11,  1, 12,  8,
    11, 18, 10, 6, 11,  2, 12, 13,
];

#[derive(Clone, Copy, Debug, Default)]
pub struct QuantEndpoint {
    pub trit_quint: u8,
    pub bits: u8,
}

pub fn unquant_endpoint(quant: QuantEndpoint, range_index: u8) -> u8 {
    let range = astc::BISE_RANGES[range_index as usize];
    let quant_bits = quant.bits as u16;
    if range.trits == 0 && range.quints == 0 && range.bits > 0 {
        // Left align bits
        let mut bits_la = quant_bits << (8 - range.bits);
        let mut val: u16 = 0;
        // Repeat bits into val
        while bits_la > 0 {
            val |= bits_la;
            bits_la >>= range.bits;
        }
        val as u8
    } else {
        let a = if quant_bits & 1 != 0 { 511 } else { 0 };
        let mut b: u16 = 0;
        for j in 0..9 {
            b <<= 1;
            let shift = range.deq_b[j];
            if shift != b'0' {
                b |= (quant_bits >> (shift - b'a')) & 0x1;
            }
        }
        let c = range.deq_c as u16;
        let d = quant.trit_quint as u16;
        let mut val = d * c + b;
        val ^= a;
        (a & 0x80 | val >> 2) as u8
    }
}

pub fn decode_endpoints(
    reader: &mut BitReaderLsb,
    range_index: u8,
    value_count: usize,
) -> [QuantEndpoint; MAX_ENDPOINT_COUNT] {
    assert!(value_count <= MAX_ENDPOINT_COUNT);

    let mut output = [QuantEndpoint::default(); MAX_ENDPOINT_COUNT];

    let range = astc::BISE_RANGES[range_index as usize];

    let bit_count = range.bits;

    if range.quints > 0 {
        const QUINTS_PER_GROUP: usize = 3;
        const BITS_PER_GROUP: usize = 7;
        let mut out_pos = 0;
        for _ in 0..(value_count / QUINTS_PER_GROUP) {
            let mut quints = reader.read_u8(BITS_PER_GROUP);
            for _ in 0..QUINTS_PER_GROUP {
                output[out_pos].trit_quint = quints % 5;
                quints /= 5;
                out_pos += 1;
            }
        }
        let remaining = value_count - out_pos;
        if remaining > 0 {
            let bits_used = match remaining {
                1 => 3,
                2 => 5,
                _ => unreachable!(),
            };
            let mut quints = reader.read_u8(bits_used);
            for _ in 0..remaining {
                output[out_pos].trit_quint = quints % 5;
                quints /= 5;
                out_pos += 1;
            }
        }
    }

    if range.trits > 0 {
        const TRITS_PER_GROUP: usize = 5;
        const BITS_PER_GROUP: usize = 8;
        let mut out_pos = 0;
        for _ in 0..(value_count / TRITS_PER_GROUP) {
            let mut trits = reader.read_u8(BITS_PER_GROUP);
            for _ in 0..TRITS_PER_GROUP {
                output[out_pos].trit_quint = trits % 3;
                trits /= 3;
                out_pos += 1;
            }
        }
        let remaining = value_count - out_pos;
        if remaining > 0 {
            let bits_used = match remaining {
                1 => 2,
                2 => 4,
                3 => 5,
                4 => 7,
                _ => unreachable!(),
            };
            let mut trits = reader.read_u8(bits_used);
            for _ in 0..remaining {
                output[out_pos].trit_quint = trits % 3;
                trits /= 3;
                out_pos += 1;
            }
        }
    }

    if bit_count > 0 {
        for endpoint in output.iter_mut().take(value_count) {
            let bits = reader.read_u8(bit_count as usize);
            endpoint.bits = bits;
        }
    }

    output
}

fn unquant_weights(weights: &mut [u8], weight_bits: u8) {
    const LUT1: [u8; 2] = [0, 64];
    const LUT2: [u8; 4] = [0, 21, 43, 64];
    const LUT3: [u8; 8] = [0, 9, 18, 27, 37, 46, 55, 64];
    const LUT4: [u8; 16] = [0, 4, 8, 12, 17, 21, 25, 29, 35, 39, 43, 47, 52, 56, 60, 64];
    const LUT5: [u8; 32] = [
        0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 34, 36, 38, 40, 42, 44, 46, 48,
        50, 52, 54, 56, 58, 60, 62, 64,
    ];

    let lut = match weight_bits {
        1 => &LUT1[..],
        2 => &LUT2[..],
        3 => &LUT3[..],
        4 => &LUT4[..],
        5 => &LUT5[..],
        _ => unreachable!(),
    };

    for weight in weights {
        *weight = lut[*weight as usize];
    }
}

pub fn decode_weights<F>(reader: &mut BitReaderLsb, mode: Mode, pat: u8, mut f: F)
where
    F: FnMut(usize, u8),
{
    let plane_count = mode.plane_count as usize;
    let anchors = get_anchor_weight_indices(mode, pat);

    // One anchor weight in each subset is encoded with one less bit (MSB = 0)
    let mut bits = [mode.weight_bits; 16];
    for &anchor in anchors {
        bits[anchor as usize] = mode.weight_bits - 1;
    }

    for i in 0..16 {
        let bits = bits[i] as usize;
        for plane in 0..plane_count {
            f(plane_count * i + plane, reader.read_u8(bits));
        }
    }
}

pub const TOTAL_ASTC_BC7_COMMON_PARTITIONS2: usize = 30;
pub const TOTAL_ASTC_BC7_COMMON_PARTITIONS3: usize = 11;
pub const TOTAL_BC7_3_ASTC2_COMMON_PARTITIONS: usize = 19;

// UASTC pattern table for the 2-subset modes
#[rustfmt::skip]
static PATTERNS_2: [[u8; 16]; TOTAL_ASTC_BC7_COMMON_PARTITIONS2] = [
    [ 0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1 ], [ 0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1 ],
    [ 1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0 ], [ 0,0,0,1,0,0,1,1,0,0,1,1,0,1,1,1 ],
    [ 1,1,1,1,1,1,1,0,1,1,1,0,1,1,0,0 ], [ 0,0,1,1,0,1,1,1,0,1,1,1,1,1,1,1 ],
    [ 1,1,1,0,1,1,0,0,1,0,0,0,0,0,0,0 ], [ 1,1,1,1,1,1,1,0,1,1,0,0,1,0,0,0 ],
    [ 0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,1 ], [ 1,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0 ],
    [ 0,0,0,0,0,0,0,1,0,1,1,1,1,1,1,1 ], [ 1,1,1,1,1,1,1,1,1,1,1,0,1,0,0,0 ],
    [ 1,1,1,0,1,0,0,0,0,0,0,0,0,0,0,0 ], [ 1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0 ],
    [ 0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,1 ], [ 1,1,1,1,1,1,1,1,1,1,1,1,0,0,0,0 ],
    [ 1,0,0,0,1,1,1,0,1,1,1,1,1,1,1,1 ], [ 1,1,1,1,1,1,1,1,0,1,1,1,0,0,0,1 ],
    [ 0,1,1,1,0,0,1,1,0,0,0,1,0,0,0,0 ], [ 0,0,1,1,0,0,0,1,0,0,0,0,0,0,0,0 ],
    [ 0,0,0,0,1,0,0,0,1,1,0,0,1,1,1,0 ], [ 1,1,1,1,1,1,1,1,0,1,1,1,0,0,1,1 ],
    [ 1,0,0,0,1,1,0,0,1,1,0,0,1,1,1,0 ], [ 0,0,1,1,0,0,0,1,0,0,0,1,0,0,0,0 ],
    [ 1,1,1,1,0,1,1,1,0,1,1,1,0,0,1,1 ], [ 0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,0 ],
    [ 1,1,1,1,0,0,0,0,0,0,0,0,1,1,1,1 ], [ 1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0 ],
    [ 1,1,1,1,0,0,0,0,1,1,1,1,0,0,0,0 ], [ 1,0,0,1,0,0,1,1,0,1,1,0,1,1,0,0 ],
];

// UASTC pattern table for the 3-subset modes
#[rustfmt::skip]
static PATTERNS_3: [[u8; 16]; TOTAL_ASTC_BC7_COMMON_PARTITIONS3] = [
    [ 0,0,0,0,0,0,0,0,1,1,2,2,1,1,2,2 ], [ 1,1,1,1,1,1,1,1,0,0,0,0,2,2,2,2 ],
    [ 1,1,1,1,0,0,0,0,0,0,0,0,2,2,2,2 ], [ 1,1,1,1,2,2,2,2,0,0,0,0,0,0,0,0 ],
    [ 1,1,2,0,1,1,2,0,1,1,2,0,1,1,2,0 ], [ 0,1,1,2,0,1,1,2,0,1,1,2,0,1,1,2 ],
    [ 0,2,1,1,0,2,1,1,0,2,1,1,0,2,1,1 ], [ 2,0,0,0,2,0,0,0,2,1,1,1,2,1,1,1 ],
    [ 2,0,1,2,2,0,1,2,2,0,1,2,2,0,1,2 ], [ 1,1,1,1,0,0,0,0,2,2,2,2,1,1,1,1 ],
    [ 0,0,2,2,0,0,1,1,0,0,1,1,0,0,2,2 ]
];

#[rustfmt::skip]
static PATTERNS_2_3: [[u8; 16]; TOTAL_BC7_3_ASTC2_COMMON_PARTITIONS] = [
    [ 0,0,0,0,1,1,1,1,0,0,0,0,0,0,0,0 ], [ 0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0 ],
    [ 1,1,0,0,1,1,0,0,1,0,0,0,0,0,0,0 ], [ 0,0,0,0,0,0,0,1,0,0,1,1,0,0,1,1 ],
    [ 1,1,1,1,1,1,1,1,0,0,0,0,1,1,1,1 ], [ 0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0 ],
    [ 0,0,0,1,0,0,1,1,1,1,1,1,1,1,1,1 ], [ 0,1,1,1,0,0,1,1,0,0,1,1,0,0,1,1 ],
    [ 1,1,0,0,0,0,0,0,0,0,1,1,1,1,0,0 ], [ 0,1,1,1,0,1,1,1,0,0,0,0,0,0,0,0 ],
    [ 0,0,0,0,0,0,0,0,1,1,1,0,1,1,1,0 ], [ 1,1,0,0,0,0,0,0,0,0,0,0,1,1,0,0 ],
    [ 0,1,1,1,0,0,1,1,0,0,0,0,0,0,0,0 ], [ 0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,1 ],
    [ 1,1,1,1,1,1,1,1,1,1,1,1,0,1,1,0 ], [ 1,1,0,0,1,1,0,0,1,1,0,0,1,0,0,0 ],
    [ 1,1,1,1,1,1,1,1,1,0,0,0,1,0,0,0 ], [ 0,0,1,1,0,1,1,0,1,1,0,0,1,0,0,0 ],
    [ 1,1,1,1,0,1,1,1,0,0,0,0,0,0,0,0 ]
];

#[rustfmt::skip]
static PATTERNS_2_ANCHORS: [[u8; 2]; TOTAL_ASTC_BC7_COMMON_PARTITIONS2] = [
    [ 0, 2 ], [ 0, 3 ], [ 1, 0 ], [ 0, 3 ], [ 7, 0 ], [ 0, 2 ], [ 3, 0 ],
    [ 7, 0 ], [ 0, 11 ], [ 2, 0 ], [ 0, 7 ], [ 11, 0 ], [ 3, 0 ], [ 8, 0 ],
    [ 0, 4 ], [ 12, 0 ], [ 1, 0 ], [ 8, 0 ], [ 0, 1 ], [ 0, 2 ], [ 0, 4 ],
    [ 8, 0 ], [ 1, 0 ], [ 0, 2 ], [ 4, 0 ], [ 0, 1 ], [ 4, 0 ], [ 1, 0 ],
    [ 4, 0 ], [ 1, 0 ]
];

#[rustfmt::skip]
static PATTERNS_3_ANCHORS: [[u8; 3]; TOTAL_ASTC_BC7_COMMON_PARTITIONS3] = [
    [ 0, 8, 10 ],  [ 8, 0, 12 ], [ 4, 0, 12 ], [ 8, 0, 4 ], [ 3, 0, 2 ],
    [ 0, 1, 3 ], [ 0, 2, 1 ], [ 1, 9, 0 ], [ 1, 2, 0 ], [ 4, 0, 8 ], [ 0, 6, 2 ]
];

#[rustfmt::skip]
static PATTERNS_2_3_ANCHORS: [[u8; 2]; TOTAL_BC7_3_ASTC2_COMMON_PARTITIONS] = [
    [ 0, 4 ], [ 0, 2 ], [ 2, 0 ], [ 0, 7 ], [ 8, 0 ], [ 0, 1 ], [ 0, 3 ],
    [ 0, 1 ], [ 2, 0 ], [ 0, 1 ], [ 0, 8 ], [ 2, 0 ], [ 0, 1 ], [ 0, 7 ],
    [ 12, 0 ], [ 2, 0 ], [ 9, 0 ], [ 0, 2 ], [ 4, 0 ]
];
