use crate::{
    bitreader::BitReaderLsb,
    bitwriter::BitWriterLsb,
    mask,
    uastc::{self, PlaneCount, SubsetCount, BC7_BLOCK_SIZE, UASTC_BLOCK_SIZE},
    Color32, Result,
};

pub fn convert_block_from_uastc(bytes: [u8; UASTC_BLOCK_SIZE]) -> Result<[u8; BC7_BLOCK_SIZE]> {
    let mut reader = BitReaderLsb::new(&bytes);

    let mode = uastc::decode_mode(&mut reader)?;

    let mut output = [0; BC7_BLOCK_SIZE];

    let mut writer = BitWriterLsb::new(&mut output);

    if mode.id == 8 {
        let rgba = uastc::decode_mode8_rgba(&mut reader);

        let (mode, endpoint, p_bits, weights) =
            convert_mode_8_to_bc7_mode_endpoint_p_bits_weights(rgba);

        let bc7_mode = BC7_MODES[mode as usize];

        let weights = &weights[0..bc7_mode.plane_count as usize];

        writer.write_u8(mode as usize + 1, 1 << mode);

        if mode == 5 {
            writer.write_u8(2, 0);
        }

        for channel in 0..4 {
            let bit_count = if channel != ALPHA_CHANNEL {
                bc7_mode.color_bits
            } else {
                bc7_mode.alpha_bits
            } as usize;
            writer.write_u8(bit_count, endpoint[0][channel]);
            writer.write_u8(bit_count, endpoint[1][channel]);
        }

        if mode == 6 {
            writer.write_u8(2, (p_bits[1] << 1) | p_bits[0]);
        }

        {
            // Write weights
            let bit_count = bc7_mode.weight_bits as usize;
            for &weight in weights.iter() {
                writer.write_u8(bit_count - 1, weight);
                for _ in 0..15 {
                    writer.write_u8(bit_count, weight);
                }
            }
        }
        return Ok(output);
    }

    let bc7_mode_index = UASTC_TO_BC7_MODES[mode.id as usize];
    let bc7_mode = BC7_MODES[bc7_mode_index as usize];

    uastc::skip_trans_flags(&mut reader, mode);

    let compsel = uastc::decode_compsel(&mut reader, mode);
    let uastc_pat = uastc::decode_pattern_index(&mut reader, mode)?;

    let bc7_endpoints_per_channel = 2 * bc7_mode.subset_count as usize;
    let bc7_channel_count = bc7_mode.endpoint_count as usize / bc7_endpoints_per_channel;

    let mut endpoints = {
        let endpoint_count = mode.endpoint_count();
        let quant_endpoints =
            uastc::decode_endpoints(&mut reader, mode.endpoint_range_index, endpoint_count);
        let mut unquant_endpoints = [0; 18];
        for (quant, unquant) in quant_endpoints
            .iter()
            .zip(unquant_endpoints.iter_mut())
            .take(endpoint_count)
        {
            *unquant = uastc::unquant_endpoint(*quant, mode.endpoint_range_index);
        }
        uastc::assemble_endpoint_pairs(mode, &unquant_endpoints)
    };

    let mut weights = [[0; 16]; 2];
    {
        if mode.plane_count == PlaneCount::Is1 {
            uastc::decode_weights(&mut reader, mode, uastc_pat, |i, w| {
                weights[0][i] = w;
            });
            convert_weights_to_bc7(&mut weights[0], mode.weight_bits, bc7_mode.weight_bits);
        } else {
            uastc::decode_weights(&mut reader, mode, uastc_pat, |i, w| {
                let plane = i & 1;
                let wi = i >> 1;
                weights[plane][wi] = w;
            });
            convert_weights_to_bc7(&mut weights[0], mode.weight_bits, bc7_mode.weight_bits);
            convert_weights_to_bc7(&mut weights[1], mode.weight_bits, bc7_mode.weight_bits);
        }
    }

    let endpoints = &mut endpoints[0..bc7_mode.subset_count as usize];
    let weights = &mut weights[0..bc7_mode.plane_count as usize];

    // Write block mode
    writer.write_u8(bc7_mode_index as usize + 1, 1 << bc7_mode_index);

    let mut bc7_anchors: &[u8] = &[0];

    const ALPHA_CHANNEL: usize = 3;

    // Write partition bits
    if bc7_mode.subset_count != SubsetCount::Is1 {
        let (bc7_pat, pattern, anchors, perm): (_, _, &[u8], &[u8]) =
            match (mode.id, mode.subset_count) {
                (1, _) => {
                    let (index, _) = PATTERNS_2_BC7_INDEX_INV[0];
                    (
                        index,
                        &PATTERNS_2_BC7[uastc_pat as usize],
                        &PATTERNS_2_BC7_ANCHORS[index as usize],
                        &[0, 0],
                    )
                }
                (7, _) => {
                    let (index, p) = PATTERNS_2_3_BC7_INDEX_PERM[uastc_pat as usize];
                    let perm = &PATTERNS_2_3_BC7_TO_ASTC_PERMUTATIONS[p as usize];
                    (
                        index,
                        &PATTERNS_2_3_BC7[uastc_pat as usize],
                        &PATTERNS_3_BC7_ANCHORS[index as usize],
                        perm,
                    )
                }
                (_, SubsetCount::Is2) => {
                    let (index, inv) = PATTERNS_2_BC7_INDEX_INV[uastc_pat as usize];
                    (
                        index,
                        &PATTERNS_2_BC7[uastc_pat as usize],
                        &PATTERNS_2_BC7_ANCHORS[index as usize],
                        if inv { &[1, 0] } else { &[0, 1] },
                    )
                }
                (_, SubsetCount::Is3) => {
                    let (index, p) = PATTERNS_3_BC7_INDEX_PERM[uastc_pat as usize];
                    let perm = &PATTERNS_3_BC7_TO_ASTC_PERMUTATIONS[p as usize];
                    (
                        index,
                        &PATTERNS_3_BC7[uastc_pat as usize],
                        &PATTERNS_3_BC7_ANCHORS[index as usize],
                        perm,
                    )
                }
                _ => unreachable!(),
            };
        bc7_anchors = anchors;

        writer.write_u8(bc7_mode.pat_bits as usize, bc7_pat);

        {
            // Permute endpoints
            let mut permuted_endpoints = [[Color32::default(); 2]; 3];
            permute(endpoints, &mut permuted_endpoints, perm);
            endpoints[0..bc7_mode.subset_count as usize]
                .copy_from_slice(&permuted_endpoints[0..bc7_mode.subset_count as usize]);
        }

        {
            // Swap endpoints and invert weights if anchor weight MSB is not 0
            let weight_mask = mask!(bc7_mode.weight_bits);
            let weight_msb_mask = 1 << (bc7_mode.weight_bits - 1);

            // Check which subset to invert
            let mut invert_subset = [false; 3];
            for (&anchor, inv) in anchors.iter().zip(invert_subset.iter_mut()) {
                *inv = weights[0][anchor as usize] & weight_msb_mask != 0;
            }

            // Swap endpoints
            for (endpoint_pair, &inv) in endpoints.iter_mut().zip(invert_subset.iter()) {
                if inv {
                    endpoint_pair.swap(0, 1);
                }
            }

            // Invert weights
            for (weight, &subset) in weights[0].iter_mut().zip(pattern.iter()) {
                if invert_subset[subset as usize] {
                    *weight = !*weight & weight_mask;
                }
            }
        }
    } else {
        let weight_mask = mask!(bc7_mode.weight_bits);
        let weight_msb_mask = 1 << (bc7_mode.weight_bits - 1);

        match mode.plane_count {
            PlaneCount::Is1 => {
                if weights[0][0] & weight_msb_mask != 0 {
                    endpoints[0].swap(0, 1);
                    for weight in weights[0].iter_mut() {
                        *weight = !*weight & weight_mask;
                    }
                }
            }
            PlaneCount::Is2 => {
                let invert_plane = [
                    weights[0][0] & weight_msb_mask != 0,
                    weights[1][0] & weight_msb_mask != 0,
                ];

                let endpoint_pair = &mut endpoints[0];

                // Apply channel rotation
                endpoint_pair[0].0.swap(compsel as usize, ALPHA_CHANNEL);
                endpoint_pair[1].0.swap(compsel as usize, ALPHA_CHANNEL);

                // Invert planes
                if invert_plane[0] {
                    endpoint_pair.swap(0, 1);
                }
                if invert_plane[0] != invert_plane[1] {
                    let [e0, e1] = endpoint_pair;
                    core::mem::swap(&mut e0[ALPHA_CHANNEL], &mut e1[ALPHA_CHANNEL]);
                }

                for (&inv, weight_plane) in invert_plane.iter().zip(weights.iter_mut()) {
                    if inv {
                        for w in weight_plane.iter_mut() {
                            *w = !*w & weight_mask;
                        }
                    }
                }

                // Write rotation bits
                writer.write_u8(2, (compsel as u8 + 1) & 0b11);

                if bc7_mode.id == 4 {
                    // Index selection bit, not used
                    writer.write_u8(1, 0);
                }
            }
        }
    }

    let color_bits = bc7_mode.color_bits as usize;
    let alpha_bits = bc7_mode.alpha_bits as usize;

    let mut p_bits = [[0u8; 2]; 3];
    if bc7_mode.p_bits != 0 {
        for (endpoint_pair, p) in endpoints.iter_mut().zip(p_bits.iter_mut()) {
            *p = determine_unique_pbits(bc7_channel_count, bc7_mode.color_bits, endpoint_pair);
        }
    } else if bc7_mode.sp_bits != 0 {
        for (endpoint_pair, p) in endpoints.iter_mut().zip(p_bits.iter_mut()) {
            *p = determine_shared_pbits(bc7_channel_count, bc7_mode.color_bits, endpoint_pair);
        }
    } else {
        fn scale_endpoint_to_bc7(e: u8, bits: usize) -> u8 {
            ((e as u32 * mask!(bits as u32) + 127) / 255) as u8
        }
        for endpoint_pair in endpoints.iter_mut() {
            for e in endpoint_pair {
                for channel in 0..3 {
                    e[channel] = scale_endpoint_to_bc7(e[channel], color_bits);
                }
                e[ALPHA_CHANNEL] = scale_endpoint_to_bc7(e[ALPHA_CHANNEL], alpha_bits);
            }
        }
    }
    let p_bits = &mut p_bits[0..bc7_mode.subset_count as usize];

    for channel in 0..bc7_channel_count {
        let bit_count = if channel != ALPHA_CHANNEL {
            color_bits
        } else {
            alpha_bits
        };
        for e in endpoints.iter() {
            writer.write_u8(bit_count, e[0][channel]);
            writer.write_u8(bit_count, e[1][channel]);
        }
    }

    if bc7_mode.p_bits != 0 {
        for p in p_bits.iter() {
            writer.write_u8(2, (p[1] << 1) | p[0]);
        }
    } else if bc7_mode.sp_bits != 0 {
        writer.write_u8(2, (p_bits[1][0] << 1) | p_bits[0][0]);
    }

    {
        // Write weights
        let mut bit_counts = [bc7_mode.weight_bits; 16];
        for &anchor in bc7_anchors {
            bit_counts[anchor as usize] -= 1;
        }
        for plane_weights in weights.iter() {
            for (&weight, &bit_count) in plane_weights.iter().zip(bit_counts.iter()) {
                writer.write_u8(bit_count as usize, weight);
            }
        }
    }

    Ok(output)
}

fn convert_mode_8_to_bc7_mode_endpoint_p_bits_weights(
    solid_color: Color32,
) -> (u8, [Color32; 2], [u8; 2], [u8; 2]) {
    // Compute the error from BC7 mode 6 p-bit 0
    let best_err0: u32 = solid_color
        .0
        .iter()
        .map(|&c| mode_6_optimal_endpoint_err(c, false))
        .sum();

    // Compute the error from BC7 mode 6 p-bit 1
    let best_err1: u32 = solid_color
        .0
        .iter()
        .map(|&c| mode_6_optimal_endpoint_err(c, true))
        .sum();

    let mode;
    let mut endpoint = [Color32::default(); 2];
    let mut p_bits = [0u8; 2];
    let mut weights = [0u8; 2];

    // Is BC7 mode 6 not lossless? If so, use mode 5 instead.
    if best_err0 > 0 && best_err1 > 0 {
        // Output BC7 mode 5
        mode = 5;

        // Convert the endpoints
        for c in 0..3 {
            endpoint[0][c] = BC7_MODE_5_OPTIMAL_ENDPOINTS[solid_color[c] as usize].lo;
            endpoint[1][c] = BC7_MODE_5_OPTIMAL_ENDPOINTS[solid_color[c] as usize].hi;
        }
        // Set the output alpha
        endpoint[0][3] = solid_color[3];
        endpoint[1][3] = solid_color[3];

        // Set the output BC7 selectors/weights to all 1's
        weights[0] = BC7ENC_MODE_5_OPTIMAL_INDEX;

        // Set the output BC7 alpha selectors/weights to all 0's
        weights[1] = 0;
    } else {
        // Output BC7 mode 6
        mode = 6;

        // Choose the p-bit with minimal error
        let best_p = best_err1 < best_err0;

        // Convert the components
        for c in 0..4 {
            endpoint[0][c] = mode_6_optimal_endpoint(solid_color[c], best_p).lo;
            endpoint[1][c] = mode_6_optimal_endpoint(solid_color[c], best_p).hi;
        }

        // Set the output p-bits
        p_bits[0] = best_p as u8;
        p_bits[1] = best_p as u8;

        // Set the output BC7 selectors/weights to all 5's
        weights = [BC7ENC_MODE_6_OPTIMAL_INDEX, BC7ENC_MODE_6_OPTIMAL_INDEX];
    }

    (mode, endpoint, p_bits, weights)
}

fn convert_weights_to_bc7(weights: &mut [u8; 16], uastc_weight_bits: u8, bc7_weight_bits: u8) {
    const UASTC_1_TO_BC7_2: &[u8; 2] = &[0, 3];
    const UASTC_2_TO_BC7_4: &[u8; 4] = &[0, 5, 10, 15];
    const UASTC_3_TO_BC7_4: &[u8; 8] = &[0, 2, 4, 6, 9, 11, 13, 15];
    const UASTC_5_TO_BC7_4: &[u8; 32] = &[
        0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 6, 7, 8, 9, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
        14, 14, 15, 15,
    ];

    let lut = match (uastc_weight_bits, bc7_weight_bits) {
        (1, 2) => &UASTC_1_TO_BC7_2[..],
        (2, 4) => &UASTC_2_TO_BC7_4[..],
        (3, 4) => &UASTC_3_TO_BC7_4[..],
        (5, 4) => &UASTC_5_TO_BC7_4[..],
        (a, b) if a == b => return,
        _ => unreachable!(),
    };

    for weight in weights {
        *weight = lut[*weight as usize];
    }
}

fn permute<T: Copy>(src: &[T], dst: &mut [T], src_for_dst: &[u8]) {
    // dst[X] = src[src_for_dst[X]]
    for (&src_id, dst_pair) in src_for_dst.iter().zip(dst.iter_mut()) {
        *dst_pair = src[src_id as usize];
    }
}

// Determines the best shared pbits to use to encode xl/xh
fn determine_shared_pbits(
    total_comps: usize,
    comp_bits: u8,
    endpoint_pair: &mut [Color32; 2],
) -> [u8; 2] {
    let total_bits = comp_bits + 1;
    assert!((4..=8).contains(&total_bits));

    let iscalep = (1 << total_bits) - 1;
    let scalep = iscalep as f32;

    let [xl_col, xh_col] = endpoint_pair;

    let mut xl = [0f32; 4];
    for (f, &i) in xl.iter_mut().zip(xl_col.0.iter()) {
        *f = i as f32 / 255.;
    }
    let mut xh = [0f32; 4];
    for (f, &i) in xh.iter_mut().zip(xh_col.0.iter()) {
        *f = i as f32 / 255.;
    }

    *xl_col = Default::default();
    *xh_col = Default::default();

    let mut best_err = 1e+9f32;

    let mut s_bit = 0;

    for p in 0..2 {
        let mut x_min_col = Color32::default();
        let mut x_max_col = Color32::default();
        for c in 0..4 {
            x_min_col[c] = (((xl[c] * scalep - p as f32) / 2. + 0.5) as i32 * 2 + p)
                .clamp(p, iscalep - 1 + p) as u8;
            x_max_col[c] = (((xh[c] * scalep - p as f32) / 2. + 0.5) as i32 * 2 + p)
                .clamp(p, iscalep - 1 + p) as u8;
        }

        let mut scaled_low = Color32::default();
        let mut scaled_high = Color32::default();

        for i in 0..4 {
            scaled_low[i] = x_min_col[i] << (8 - total_bits);
            scaled_low[i] |= scaled_low[i] >> total_bits;

            scaled_high[i] = x_max_col[i] << (8 - total_bits);
            scaled_high[i] |= scaled_high[i] >> total_bits;
        }

        let mut err = 0.;
        for i in 0..total_comps {
            err += (scaled_low[i] as f32 / 255. - xl[i]).powi(2)
                + (scaled_high[i] as f32 / 255. - xh[i]).powi(2);
        }

        if err < best_err {
            best_err = err;
            s_bit = p as u8;
            for j in 0..4 {
                xl_col[j] = x_min_col[j] >> 1;
                xh_col[j] = x_max_col[j] >> 1;
            }
        }
    }

    [s_bit, s_bit]
}

// Determines the best unique pbits to use to encode xl/xh
fn determine_unique_pbits(
    total_comps: usize,
    comp_bits: u8,
    endpoint_pair: &mut [Color32; 2],
) -> [u8; 2] {
    let total_bits = comp_bits + 1;
    let iscalep = (1 << total_bits) - 1;
    let scalep = iscalep as f32;

    let [xl_col, xh_col] = endpoint_pair;

    let mut xl = [0f32; 4];
    for (f, &i) in xl.iter_mut().zip(xl_col.0.iter()) {
        *f = i as f32 / 255.;
    }
    let mut xh = [0f32; 4];
    for (f, &i) in xh.iter_mut().zip(xh_col.0.iter()) {
        *f = i as f32 / 255.;
    }

    *xl_col = Default::default();
    *xh_col = Default::default();

    let mut best_err0 = 1e+9f32;
    let mut best_err1 = 1e+9f32;

    let mut p_bits: [u8; 2] = Default::default();

    for p in 0..2 {
        let mut x_min_color = Color32::default();
        let mut x_max_color = Color32::default();

        for c in 0..4 {
            x_min_color[c] = (((xl[c] * scalep - p as f32) / 2. + 0.5) as i32 * 2 + p)
                .clamp(p, iscalep - 1 + p) as u8;
            x_max_color[c] = (((xh[c] * scalep - p as f32) / 2. + 0.5) as i32 * 2 + p)
                .clamp(p, iscalep - 1 + p) as u8;
        }

        let mut scaled_low = Color32::default();
        let mut scaled_high = Color32::default();

        for i in 0..4 {
            scaled_low[i] = x_min_color[i] << (8 - total_bits);
            scaled_low[i] |= scaled_low[i].wrapping_shr(total_bits as u32);

            scaled_high[i] = x_max_color[i] << (8 - total_bits);
            scaled_high[i] |= scaled_high[i].wrapping_shr(total_bits as u32);
        }

        let mut err0 = 0.;
        let mut err1 = 0.;
        for i in 0..total_comps {
            err0 += (scaled_low[i] as f32 - xl[i] * 255.).powi(2);
            err1 += (scaled_high[i] as f32 - xh[i] * 255.).powi(2);
        }

        if err0 < best_err0 {
            best_err0 = err0;
            p_bits[0] = p as u8;
            for j in 0..4 {
                xl_col[j] = x_min_color[j] >> 1;
            }
        }

        if err1 < best_err1 {
            best_err1 = err1;
            p_bits[1] = p as u8;
            for j in 0..4 {
                xh_col[j] = x_max_color[j] >> 1;
            }
        }
    }

    p_bits
}

#[derive(Clone, Copy, Debug)]
struct Bc7Mode {
    id: u8,
    pat_bits: u8,
    endpoint_count: u8,
    color_bits: u8,
    alpha_bits: u8,
    weight_bits: u8,
    plane_count: PlaneCount,
    subset_count: SubsetCount,
    p_bits: u8,
    sp_bits: u8,
}

#[rustfmt::skip]
static BC7_MODES: [Bc7Mode; 8] = [
    Bc7Mode { id: 0, endpoint_count: 18, color_bits: 4, alpha_bits: 0, weight_bits: 3, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is3, pat_bits: 4, p_bits: 1, sp_bits: 0 },
    Bc7Mode { id: 1, endpoint_count: 12, color_bits: 6, alpha_bits: 0, weight_bits: 3, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is2, pat_bits: 6, p_bits: 0, sp_bits: 1 },
    Bc7Mode { id: 2, endpoint_count: 18, color_bits: 5, alpha_bits: 0, weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is3, pat_bits: 6, p_bits: 0, sp_bits: 0 },
    Bc7Mode { id: 3, endpoint_count: 12, color_bits: 7, alpha_bits: 0, weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is2, pat_bits: 6, p_bits: 1, sp_bits: 0 },
    Bc7Mode { id: 4, endpoint_count:  8, color_bits: 5, alpha_bits: 6, weight_bits: 2, plane_count: PlaneCount::Is2, subset_count: SubsetCount::Is1, pat_bits: 0, p_bits: 0, sp_bits: 0 },
    Bc7Mode { id: 5, endpoint_count:  8, color_bits: 7, alpha_bits: 8, weight_bits: 2, plane_count: PlaneCount::Is2, subset_count: SubsetCount::Is1, pat_bits: 0, p_bits: 0, sp_bits: 0 },
    Bc7Mode { id: 6, endpoint_count:  8, color_bits: 7, alpha_bits: 7, weight_bits: 4, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is1, pat_bits: 0, p_bits: 1, sp_bits: 0 },
    Bc7Mode { id: 7, endpoint_count: 16, color_bits: 5, alpha_bits: 5, weight_bits: 2, plane_count: PlaneCount::Is1, subset_count: SubsetCount::Is2, pat_bits: 6, p_bits: 1, sp_bits: 0 },
];

#[rustfmt::skip]
static UASTC_TO_BC7_MODES: [u8; 20] = [
    6, 3, 1, 2, 3, 6, 5, 2, // 0..=7 RGB
    0,                      // 8 Void extent
    7, 6, 5, 6, 5, 6,       // 9..=14 RGBA
    6, 7, 5,                // 15..=17 LA
    6,                      // 18 RGB
    0,                      // 19 Reserved
];

#[rustfmt::skip]
static PATTERNS_2_BC7_INDEX_INV: [(u8, bool); uastc::TOTAL_ASTC_BC7_COMMON_PARTITIONS2] = [
    ( 0, false), ( 1, false), ( 2, true ), ( 3, false),
    ( 4, true ), ( 5, false), ( 6, true ), ( 7, true ),
    ( 8, false), ( 9, true ), (10, false), (11, true ),
    (12, true ), (13, true ), (14, false), (15, true ),
    (17, true ), (18, true ), (19, false), (20, false),
    (21, false), (22, true ), (23, true ), (24, false),
    (25, true ), (26, false), (29, true ), (32, true ),
    (33, true ), (52, true ),
];

#[rustfmt::skip]
static PATTERNS_3_BC7_INDEX_PERM: [(u8, u8); uastc::TOTAL_ASTC_BC7_COMMON_PARTITIONS3] = [
    ( 4, 0), ( 8, 5), ( 9, 5), (10, 2),
    (11, 2), (12, 0), (13, 4), (20, 1),
    (35, 1), (36, 5), (57, 0),
];

#[rustfmt::skip]
static PATTERNS_3_BC7_TO_ASTC_PERMUTATIONS: [[u8; 3]; 6] = [
    [0, 1, 2], [2, 0, 1], [1, 2, 0], [2, 1, 0], [0, 2, 1], [1, 0, 2],
];

#[rustfmt::skip]
static PATTERNS_2_3_BC7_INDEX_PERM: [(u8, u8); uastc::TOTAL_BC7_3_ASTC2_COMMON_PARTITIONS] = [
    (10, 4), (11, 4), ( 0, 3), ( 2, 4),
    ( 8, 5), (13, 4), ( 1, 2), (33, 2),
    (40, 3), (20, 4), (21, 0), (58, 3),
    ( 3, 0), (32, 2), (59, 1), (34, 3),
    (20, 1), (14, 4), (31, 3),
];

#[rustfmt::skip]
static PATTERNS_2_3_BC7_TO_ASTC_PERMUTATIONS: [[u8; 3]; 6] = [
    [0, 0, 1], [1, 1, 0], [0, 1, 1], [1, 0, 0], [0, 1, 0], [1, 0, 1],
];

// fn convert_bc7_subset_indices_3_to_2(mut p: u8, k: u8) -> u8 {
//     assert!(k < 6);
//     match k >> 1 {
//         0 => if p <= 1 {
//             p = 0;
//         } else {
//             p = 1;
//         }
//         1 => if p == 0 {
//             p = 0;
//         } else {
//             p = 1;
//         }
//         2 => if (p == 0) || (p == 2) {
//             p = 0;
//         } else {
//             p = 1;
//         }
//         _ => unreachable!()
//     }
//     if k & 1 == 1 {
//         p = 1 - p;
//     }
//     p
// }

#[rustfmt::skip]
static PATTERNS_2_BC7: [[u8; 16]; uastc::TOTAL_ASTC_BC7_COMMON_PARTITIONS2] = [
    [ 0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1 ], [ 0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1 ],
    [ 0,1,1,1,0,1,1,1,0,1,1,1,0,1,1,1 ], [ 0,0,0,1,0,0,1,1,0,0,1,1,0,1,1,1 ],
    [ 0,0,0,0,0,0,0,1,0,0,0,1,0,0,1,1 ], [ 0,0,1,1,0,1,1,1,0,1,1,1,1,1,1,1 ],
    [ 0,0,0,1,0,0,1,1,0,1,1,1,1,1,1,1 ], [ 0,0,0,0,0,0,0,1,0,0,1,1,0,1,1,1 ],
    [ 0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,1 ], [ 0,0,1,1,0,1,1,1,1,1,1,1,1,1,1,1 ],
    [ 0,0,0,0,0,0,0,1,0,1,1,1,1,1,1,1 ], [ 0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,1 ],
    [ 0,0,0,1,0,1,1,1,1,1,1,1,1,1,1,1 ], [ 0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1 ],
    [ 0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,1 ], [ 0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1 ],
    [ 0,1,1,1,0,0,0,1,0,0,0,0,0,0,0,0 ], [ 0,0,0,0,0,0,0,0,1,0,0,0,1,1,1,0 ],
    [ 0,1,1,1,0,0,1,1,0,0,0,1,0,0,0,0 ], [ 0,0,1,1,0,0,0,1,0,0,0,0,0,0,0,0 ],
    [ 0,0,0,0,1,0,0,0,1,1,0,0,1,1,1,0 ], [ 0,0,0,0,0,0,0,0,1,0,0,0,1,1,0,0 ],
    [ 0,1,1,1,0,0,1,1,0,0,1,1,0,0,0,1 ], [ 0,0,1,1,0,0,0,1,0,0,0,1,0,0,0,0 ],
    [ 0,0,0,0,1,0,0,0,1,0,0,0,1,1,0,0 ], [ 0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,0 ],
    [ 0,0,0,0,1,1,1,1,1,1,1,1,0,0,0,0 ], [ 0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1 ],
    [ 0,0,0,0,1,1,1,1,0,0,0,0,1,1,1,1 ], [ 0,1,1,0,1,1,0,0,1,0,0,1,0,0,1,1 ],
];

#[rustfmt::skip]
static PATTERNS_3_BC7: [[u8; 16]; uastc::TOTAL_ASTC_BC7_COMMON_PARTITIONS3] = [
    [ 0,0,0,0,0,0,0,0,1,1,2,2,1,1,2,2 ], [ 0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2 ],
    [ 0,0,0,0,1,1,1,1,1,1,1,1,2,2,2,2 ], [ 0,0,0,0,1,1,1,1,2,2,2,2,2,2,2,2 ],
    [ 0,0,1,2,0,0,1,2,0,0,1,2,0,0,1,2 ], [ 0,1,1,2,0,1,1,2,0,1,1,2,0,1,1,2 ],
    [ 0,1,2,2,0,1,2,2,0,1,2,2,0,1,2,2 ], [ 0,1,1,1,0,1,1,1,0,2,2,2,0,2,2,2 ],
    [ 0,1,2,0,0,1,2,0,0,1,2,0,0,1,2,0 ], [ 0,0,0,0,1,1,1,1,2,2,2,2,0,0,0,0 ],
    [ 0,0,2,2,0,0,1,1,0,0,1,1,0,0,2,2 ]
];

#[rustfmt::skip]
static PATTERNS_2_3_BC7: [[u8; 16]; uastc::TOTAL_BC7_3_ASTC2_COMMON_PARTITIONS] = [
    [ 0,0,0,0,1,1,1,1,2,2,2,2,2,2,2,2 ], [ 0,0,1,2,0,0,1,2,0,0,1,2,0,0,1,2 ],
    [ 0,0,1,1,0,0,1,1,0,2,2,1,2,2,2,2 ], [ 0,0,0,0,2,0,0,1,2,2,1,1,2,2,1,1 ],
    [ 0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2 ], [ 0,1,2,2,0,1,2,2,0,1,2,2,0,1,2,2 ],
    [ 0,0,0,1,0,0,1,1,2,2,1,1,2,2,2,1 ], [ 0,2,2,2,0,0,2,2,0,0,1,2,0,0,1,1 ],
    [ 0,0,1,1,1,1,2,2,2,2,0,0,0,0,1,1 ], [ 0,1,1,1,0,1,1,1,0,2,2,2,0,2,2,2 ],
    [ 0,0,0,1,0,0,0,1,2,2,2,1,2,2,2,1 ], [ 0,0,2,2,1,1,2,2,1,1,2,2,0,0,2,2 ],
    [ 0,2,2,2,0,0,2,2,0,0,1,1,0,1,1,1 ], [ 0,0,0,0,0,0,0,2,1,1,2,2,1,2,2,2 ],
    [ 0,0,0,0,0,0,0,0,0,0,0,0,2,1,1,2 ], [ 0,0,1,1,0,0,1,2,0,0,2,2,0,2,2,2 ],
    [ 0,1,1,1,0,1,1,1,0,2,2,2,0,2,2,2 ], [ 0,0,1,1,0,1,1,2,1,1,2,2,1,2,2,2 ],
    [ 0,0,0,0,2,0,0,0,2,2,1,1,2,2,2,1 ]
];

const TOTAL_BC7_PATTERNS: usize = 64;

#[rustfmt::skip]
static PATTERNS_2_BC7_ANCHORS: [[u8; 2]; TOTAL_BC7_PATTERNS] = [
    [0, 15], [0, 15], [0, 15], [0, 15], [0, 15], [0, 15], [0, 15], [0, 15],
    [0, 15], [0, 15], [0, 15], [0, 15], [0, 15], [0, 15], [0, 15], [0, 15],
    [0, 15], [0,  2], [0,  8], [0,  2], [0,  2], [0,  8], [0,  8], [0, 15],
    [0,  2], [0,  8], [0,  2], [0,  2], [0,  8], [0,  8], [0,  2], [0,  2],
    [0, 15], [0, 15], [0,  6], [0,  8], [0,  2], [0,  8], [0, 15], [0, 15],
    [0,  2], [0,  8], [0,  2], [0,  2], [0,  2], [0, 15], [0, 15], [0,  6],
    [0,  6], [0,  2], [0,  6], [0,  8], [0, 15], [0, 15], [0,  2], [0,  2],
    [0, 15], [0, 15], [0, 15], [0, 15], [0, 15], [0,  2], [0,  2], [0, 15],
];

#[rustfmt::skip]
static PATTERNS_3_BC7_ANCHORS: [[u8; 3]; TOTAL_BC7_PATTERNS] = [
    [0,  3, 15], [0,  3,  8], [0, 15,  8], [0, 15,  3], [0,  8, 15], [0,  3, 15], [0, 15,  3], [0, 15,  8],
    [0,  8, 15], [0,  8, 15], [0,  6, 15], [0,  6, 15], [0,  6, 15], [0,  5, 15], [0,  3, 15], [0,  3,  8],
    [0,  3, 15], [0,  3,  8], [0,  8, 15], [0, 15,  3], [0,  3, 15], [0,  3,  8], [0,  6, 15], [0, 10,  8],
    [0,  5,  3], [0,  8, 15], [0,  8,  6], [0,  6, 10], [0,  8, 15], [0,  5, 15], [0, 15, 10], [0, 15,  8],
    [0,  8, 15], [0, 15,  3], [0,  3, 15], [0,  5, 10], [0,  6, 10], [0, 10,  8], [0,  8,  9], [0, 15, 10],
    [0, 15,  6], [0,  3, 15], [0, 15,  8], [0,  5, 15], [0, 15,  3], [0, 15,  6], [0, 15,  6], [0, 15,  8],
    [0,  3, 15], [0, 15,  3], [0,  5, 15], [0,  5, 15], [0,  5, 15], [0,  8, 15], [0,  5, 15], [0, 10, 15],
    [0,  5, 15], [0, 10, 15], [0,  8, 15], [0, 13, 15], [0, 15,  3], [0, 12, 15], [0,  3, 15], [0,  3,  8],
];

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct OptimalEndpoint {
    lo: u8,
    hi: u8,
}

const BC7ENC_MODE_5_OPTIMAL_INDEX: u8 = 1;
const BC7ENC_MODE_6_OPTIMAL_INDEX: u8 = 5;

#[rustfmt::skip]
const BC7_MODE_5_OPTIMAL_ENDPOINTS: [OptimalEndpoint; 256] = [
    OptimalEndpoint { lo: 0, hi: 0 }, OptimalEndpoint { lo: 0, hi: 1 },
    OptimalEndpoint { lo: 0, hi: 3 }, OptimalEndpoint { lo: 0, hi: 4 },
    OptimalEndpoint { lo: 0, hi: 6 }, OptimalEndpoint { lo: 0, hi: 7 },
    OptimalEndpoint { lo: 0, hi: 9 }, OptimalEndpoint { lo: 0, hi: 10 },
    OptimalEndpoint { lo: 0, hi: 12 }, OptimalEndpoint { lo: 0, hi: 13 },
    OptimalEndpoint { lo: 0, hi: 15 }, OptimalEndpoint { lo: 0, hi: 16 },
    OptimalEndpoint { lo: 0, hi: 18 }, OptimalEndpoint { lo: 0, hi: 20 },
    OptimalEndpoint { lo: 0, hi: 21 }, OptimalEndpoint { lo: 0, hi: 23 },
    OptimalEndpoint { lo: 0, hi: 24 }, OptimalEndpoint { lo: 0, hi: 26 },
    OptimalEndpoint { lo: 0, hi: 27 }, OptimalEndpoint { lo: 0, hi: 29 },
    OptimalEndpoint { lo: 0, hi: 30 }, OptimalEndpoint { lo: 0, hi: 32 },
    OptimalEndpoint { lo: 0, hi: 33 }, OptimalEndpoint { lo: 0, hi: 35 },
    OptimalEndpoint { lo: 0, hi: 36 }, OptimalEndpoint { lo: 0, hi: 38 },
    OptimalEndpoint { lo: 0, hi: 39 }, OptimalEndpoint { lo: 0, hi: 41 },
    OptimalEndpoint { lo: 0, hi: 42 }, OptimalEndpoint { lo: 0, hi: 44 },
    OptimalEndpoint { lo: 0, hi: 45 }, OptimalEndpoint { lo: 0, hi: 47 },
    OptimalEndpoint { lo: 0, hi: 48 }, OptimalEndpoint { lo: 0, hi: 50 },
    OptimalEndpoint { lo: 0, hi: 52 }, OptimalEndpoint { lo: 0, hi: 53 },
    OptimalEndpoint { lo: 0, hi: 55 }, OptimalEndpoint { lo: 0, hi: 56 },
    OptimalEndpoint { lo: 0, hi: 58 }, OptimalEndpoint { lo: 0, hi: 59 },
    OptimalEndpoint { lo: 0, hi: 61 }, OptimalEndpoint { lo: 0, hi: 62 },
    OptimalEndpoint { lo: 0, hi: 64 }, OptimalEndpoint { lo: 0, hi: 65 },
    OptimalEndpoint { lo: 0, hi: 66 }, OptimalEndpoint { lo: 0, hi: 68 },
    OptimalEndpoint { lo: 0, hi: 69 }, OptimalEndpoint { lo: 0, hi: 71 },
    OptimalEndpoint { lo: 0, hi: 72 }, OptimalEndpoint { lo: 0, hi: 74 },
    OptimalEndpoint { lo: 0, hi: 75 }, OptimalEndpoint { lo: 0, hi: 77 },
    OptimalEndpoint { lo: 0, hi: 78 }, OptimalEndpoint { lo: 0, hi: 80 },
    OptimalEndpoint { lo: 0, hi: 82 }, OptimalEndpoint { lo: 0, hi: 83 },
    OptimalEndpoint { lo: 0, hi: 85 }, OptimalEndpoint { lo: 0, hi: 86 },
    OptimalEndpoint { lo: 0, hi: 88 }, OptimalEndpoint { lo: 0, hi: 89 },
    OptimalEndpoint { lo: 0, hi: 91 }, OptimalEndpoint { lo: 0, hi: 92 },
    OptimalEndpoint { lo: 0, hi: 94 }, OptimalEndpoint { lo: 0, hi: 95 },
    OptimalEndpoint { lo: 0, hi: 97 }, OptimalEndpoint { lo: 0, hi: 98 },
    OptimalEndpoint { lo: 0, hi: 100 }, OptimalEndpoint { lo: 0, hi: 101 },
    OptimalEndpoint { lo: 0, hi: 103 }, OptimalEndpoint { lo: 0, hi: 104 },
    OptimalEndpoint { lo: 0, hi: 106 }, OptimalEndpoint { lo: 0, hi: 107 },
    OptimalEndpoint { lo: 0, hi: 109 }, OptimalEndpoint { lo: 0, hi: 110 },
    OptimalEndpoint { lo: 0, hi: 112 }, OptimalEndpoint { lo: 0, hi: 114 },
    OptimalEndpoint { lo: 0, hi: 115 }, OptimalEndpoint { lo: 0, hi: 117 },
    OptimalEndpoint { lo: 0, hi: 118 }, OptimalEndpoint { lo: 0, hi: 120 },
    OptimalEndpoint { lo: 0, hi: 121 }, OptimalEndpoint { lo: 0, hi: 123 },
    OptimalEndpoint { lo: 0, hi: 124 }, OptimalEndpoint { lo: 0, hi: 126 },
    OptimalEndpoint { lo: 0, hi: 127 }, OptimalEndpoint { lo: 1, hi: 127 },
    OptimalEndpoint { lo: 2, hi: 126 }, OptimalEndpoint { lo: 3, hi: 126 },
    OptimalEndpoint { lo: 3, hi: 127 }, OptimalEndpoint { lo: 4, hi: 127 },
    OptimalEndpoint { lo: 5, hi: 126 }, OptimalEndpoint { lo: 6, hi: 126 },
    OptimalEndpoint { lo: 6, hi: 127 }, OptimalEndpoint { lo: 7, hi: 127 },
    OptimalEndpoint { lo: 8, hi: 126 }, OptimalEndpoint { lo: 9, hi: 126 },
    OptimalEndpoint { lo: 9, hi: 127 }, OptimalEndpoint { lo: 10, hi: 127 },
    OptimalEndpoint { lo: 11, hi: 126 }, OptimalEndpoint { lo: 12, hi: 126 },
    OptimalEndpoint { lo: 12, hi: 127 }, OptimalEndpoint { lo: 13, hi: 127 },
    OptimalEndpoint { lo: 14, hi: 126 }, OptimalEndpoint { lo: 15, hi: 125 },
    OptimalEndpoint { lo: 15, hi: 127 }, OptimalEndpoint { lo: 16, hi: 126 },
    OptimalEndpoint { lo: 17, hi: 126 }, OptimalEndpoint { lo: 17, hi: 127 },
    OptimalEndpoint { lo: 18, hi: 127 }, OptimalEndpoint { lo: 19, hi: 126 },
    OptimalEndpoint { lo: 20, hi: 126 }, OptimalEndpoint { lo: 20, hi: 127 },
    OptimalEndpoint { lo: 21, hi: 127 }, OptimalEndpoint { lo: 22, hi: 126 },
    OptimalEndpoint { lo: 23, hi: 126 }, OptimalEndpoint { lo: 23, hi: 127 },
    OptimalEndpoint { lo: 24, hi: 127 }, OptimalEndpoint { lo: 25, hi: 126 },
    OptimalEndpoint { lo: 26, hi: 126 }, OptimalEndpoint { lo: 26, hi: 127 },
    OptimalEndpoint { lo: 27, hi: 127 }, OptimalEndpoint { lo: 28, hi: 126 },
    OptimalEndpoint { lo: 29, hi: 126 }, OptimalEndpoint { lo: 29, hi: 127 },
    OptimalEndpoint { lo: 30, hi: 127 }, OptimalEndpoint { lo: 31, hi: 126 },
    OptimalEndpoint { lo: 32, hi: 126 }, OptimalEndpoint { lo: 32, hi: 127 },
    OptimalEndpoint { lo: 33, hi: 127 }, OptimalEndpoint { lo: 34, hi: 126 },
    OptimalEndpoint { lo: 35, hi: 126 }, OptimalEndpoint { lo: 35, hi: 127 },
    OptimalEndpoint { lo: 36, hi: 127 }, OptimalEndpoint { lo: 37, hi: 126 },
    OptimalEndpoint { lo: 38, hi: 126 }, OptimalEndpoint { lo: 38, hi: 127 },
    OptimalEndpoint { lo: 39, hi: 127 }, OptimalEndpoint { lo: 40, hi: 126 },
    OptimalEndpoint { lo: 41, hi: 126 }, OptimalEndpoint { lo: 41, hi: 127 },
    OptimalEndpoint { lo: 42, hi: 127 }, OptimalEndpoint { lo: 43, hi: 126 },
    OptimalEndpoint { lo: 44, hi: 126 }, OptimalEndpoint { lo: 44, hi: 127 },
    OptimalEndpoint { lo: 45, hi: 127 }, OptimalEndpoint { lo: 46, hi: 126 },
    OptimalEndpoint { lo: 47, hi: 125 }, OptimalEndpoint { lo: 47, hi: 127 },
    OptimalEndpoint { lo: 48, hi: 126 }, OptimalEndpoint { lo: 49, hi: 126 },
    OptimalEndpoint { lo: 49, hi: 127 }, OptimalEndpoint { lo: 50, hi: 127 },
    OptimalEndpoint { lo: 51, hi: 126 }, OptimalEndpoint { lo: 52, hi: 126 },
    OptimalEndpoint { lo: 52, hi: 127 }, OptimalEndpoint { lo: 53, hi: 127 },
    OptimalEndpoint { lo: 54, hi: 126 }, OptimalEndpoint { lo: 55, hi: 126 },
    OptimalEndpoint { lo: 55, hi: 127 }, OptimalEndpoint { lo: 56, hi: 127 },
    OptimalEndpoint { lo: 57, hi: 126 }, OptimalEndpoint { lo: 58, hi: 126 },
    OptimalEndpoint { lo: 58, hi: 127 }, OptimalEndpoint { lo: 59, hi: 127 },
    OptimalEndpoint { lo: 60, hi: 126 }, OptimalEndpoint { lo: 61, hi: 126 },
    OptimalEndpoint { lo: 61, hi: 127 }, OptimalEndpoint { lo: 62, hi: 127 },
    OptimalEndpoint { lo: 63, hi: 126 }, OptimalEndpoint { lo: 64, hi: 125 },
    OptimalEndpoint { lo: 64, hi: 126 }, OptimalEndpoint { lo: 65, hi: 126 },
    OptimalEndpoint { lo: 65, hi: 127 }, OptimalEndpoint { lo: 66, hi: 127 },
    OptimalEndpoint { lo: 67, hi: 126 }, OptimalEndpoint { lo: 68, hi: 126 },
    OptimalEndpoint { lo: 68, hi: 127 }, OptimalEndpoint { lo: 69, hi: 127 },
    OptimalEndpoint { lo: 70, hi: 126 }, OptimalEndpoint { lo: 71, hi: 126 },
    OptimalEndpoint { lo: 71, hi: 127 }, OptimalEndpoint { lo: 72, hi: 127 },
    OptimalEndpoint { lo: 73, hi: 126 }, OptimalEndpoint { lo: 74, hi: 126 },
    OptimalEndpoint { lo: 74, hi: 127 }, OptimalEndpoint { lo: 75, hi: 127 },
    OptimalEndpoint { lo: 76, hi: 126 }, OptimalEndpoint { lo: 77, hi: 125 },
    OptimalEndpoint { lo: 77, hi: 127 }, OptimalEndpoint { lo: 78, hi: 126 },
    OptimalEndpoint { lo: 79, hi: 126 }, OptimalEndpoint { lo: 79, hi: 127 },
    OptimalEndpoint { lo: 80, hi: 127 }, OptimalEndpoint { lo: 81, hi: 126 },
    OptimalEndpoint { lo: 82, hi: 126 }, OptimalEndpoint { lo: 82, hi: 127 },
    OptimalEndpoint { lo: 83, hi: 127 }, OptimalEndpoint { lo: 84, hi: 126 },
    OptimalEndpoint { lo: 85, hi: 126 }, OptimalEndpoint { lo: 85, hi: 127 },
    OptimalEndpoint { lo: 86, hi: 127 }, OptimalEndpoint { lo: 87, hi: 126 },
    OptimalEndpoint { lo: 88, hi: 126 }, OptimalEndpoint { lo: 88, hi: 127 },
    OptimalEndpoint { lo: 89, hi: 127 }, OptimalEndpoint { lo: 90, hi: 126 },
    OptimalEndpoint { lo: 91, hi: 126 }, OptimalEndpoint { lo: 91, hi: 127 },
    OptimalEndpoint { lo: 92, hi: 127 }, OptimalEndpoint { lo: 93, hi: 126 },
    OptimalEndpoint { lo: 94, hi: 126 }, OptimalEndpoint { lo: 94, hi: 127 },
    OptimalEndpoint { lo: 95, hi: 127 }, OptimalEndpoint { lo: 96, hi: 126 },
    OptimalEndpoint { lo: 97, hi: 126 }, OptimalEndpoint { lo: 97, hi: 127 },
    OptimalEndpoint { lo: 98, hi: 127 }, OptimalEndpoint { lo: 99, hi: 126 },
    OptimalEndpoint { lo: 100, hi: 126 }, OptimalEndpoint { lo: 100, hi: 127 },
    OptimalEndpoint { lo: 101, hi: 127 }, OptimalEndpoint { lo: 102, hi: 126 },
    OptimalEndpoint { lo: 103, hi: 126 }, OptimalEndpoint { lo: 103, hi: 127 },
    OptimalEndpoint { lo: 104, hi: 127 }, OptimalEndpoint { lo: 105, hi: 126 },
    OptimalEndpoint { lo: 106, hi: 126 }, OptimalEndpoint { lo: 106, hi: 127 },
    OptimalEndpoint { lo: 107, hi: 127 }, OptimalEndpoint { lo: 108, hi: 126 },
    OptimalEndpoint { lo: 109, hi: 125 }, OptimalEndpoint { lo: 109, hi: 127 },
    OptimalEndpoint { lo: 110, hi: 126 }, OptimalEndpoint { lo: 111, hi: 126 },
    OptimalEndpoint { lo: 111, hi: 127 }, OptimalEndpoint { lo: 112, hi: 127 },
    OptimalEndpoint { lo: 113, hi: 126 }, OptimalEndpoint { lo: 114, hi: 126 },
    OptimalEndpoint { lo: 114, hi: 127 }, OptimalEndpoint { lo: 115, hi: 127 },
    OptimalEndpoint { lo: 116, hi: 126 }, OptimalEndpoint { lo: 117, hi: 126 },
    OptimalEndpoint { lo: 117, hi: 127 }, OptimalEndpoint { lo: 118, hi: 127 },
    OptimalEndpoint { lo: 119, hi: 126 }, OptimalEndpoint { lo: 120, hi: 126 },
    OptimalEndpoint { lo: 120, hi: 127 }, OptimalEndpoint { lo: 121, hi: 127 },
    OptimalEndpoint { lo: 122, hi: 126 }, OptimalEndpoint { lo: 123, hi: 126 },
    OptimalEndpoint { lo: 123, hi: 127 }, OptimalEndpoint { lo: 124, hi: 127 },
    OptimalEndpoint { lo: 125, hi: 126 }, OptimalEndpoint { lo: 126, hi: 126 },
    OptimalEndpoint { lo: 126, hi: 127 }, OptimalEndpoint { lo: 127, hi: 127 },
];

#[rustfmt::skip]
const BC7_MODE_6_OPTIMAL_ENDPOINTS: [OptimalEndpoint; 257] = [
    OptimalEndpoint { lo: 0, hi: 0 },
    OptimalEndpoint { lo: 0, hi: 0 },
    OptimalEndpoint { lo: 0, hi: 1 },
    OptimalEndpoint { lo: 0, hi: 3 },
    OptimalEndpoint { lo: 0, hi: 4 },
    OptimalEndpoint { lo: 0, hi: 6 },
    OptimalEndpoint { lo: 0, hi: 7 },
    OptimalEndpoint { lo: 0, hi: 9 },
    OptimalEndpoint { lo: 0, hi: 10 },
    OptimalEndpoint { lo: 0, hi: 12 },
    OptimalEndpoint { lo: 0, hi: 13 },
    OptimalEndpoint { lo: 0, hi: 15 },
    OptimalEndpoint { lo: 0, hi: 16 },
    OptimalEndpoint { lo: 0, hi: 18 },
    OptimalEndpoint { lo: 0, hi: 20 },
    OptimalEndpoint { lo: 0, hi: 21 },
    OptimalEndpoint { lo: 0, hi: 23 },
    OptimalEndpoint { lo: 0, hi: 24 },
    OptimalEndpoint { lo: 0, hi: 26 },
    OptimalEndpoint { lo: 0, hi: 27 },
    OptimalEndpoint { lo: 0, hi: 29 },
    OptimalEndpoint { lo: 0, hi: 30 },
    OptimalEndpoint { lo: 0, hi: 32 },
    OptimalEndpoint { lo: 0, hi: 33 },
    OptimalEndpoint { lo: 0, hi: 35 },
    OptimalEndpoint { lo: 0, hi: 36 },
    OptimalEndpoint { lo: 0, hi: 38 },
    OptimalEndpoint { lo: 0, hi: 39 },
    OptimalEndpoint { lo: 0, hi: 41 },
    OptimalEndpoint { lo: 0, hi: 42 },
    OptimalEndpoint { lo: 0, hi: 44 },
    OptimalEndpoint { lo: 0, hi: 45 },
    OptimalEndpoint { lo: 0, hi: 47 },
    OptimalEndpoint { lo: 0, hi: 48 },
    OptimalEndpoint { lo: 0, hi: 50 },
    OptimalEndpoint { lo: 0, hi: 52 },
    OptimalEndpoint { lo: 0, hi: 53 },
    OptimalEndpoint { lo: 0, hi: 55 },
    OptimalEndpoint { lo: 0, hi: 56 },
    OptimalEndpoint { lo: 0, hi: 58 },
    OptimalEndpoint { lo: 0, hi: 59 },
    OptimalEndpoint { lo: 0, hi: 61 },
    OptimalEndpoint { lo: 0, hi: 62 },
    OptimalEndpoint { lo: 0, hi: 64 },
    OptimalEndpoint { lo: 0, hi: 65 },
    OptimalEndpoint { lo: 0, hi: 67 },
    OptimalEndpoint { lo: 0, hi: 68 },
    OptimalEndpoint { lo: 0, hi: 70 },
    OptimalEndpoint { lo: 0, hi: 71 },
    OptimalEndpoint { lo: 0, hi: 73 },
    OptimalEndpoint { lo: 0, hi: 74 },
    OptimalEndpoint { lo: 0, hi: 76 },
    OptimalEndpoint { lo: 0, hi: 77 },
    OptimalEndpoint { lo: 0, hi: 79 },
    OptimalEndpoint { lo: 0, hi: 80 },
    OptimalEndpoint { lo: 0, hi: 82 },
    OptimalEndpoint { lo: 0, hi: 84 },
    OptimalEndpoint { lo: 0, hi: 85 },
    OptimalEndpoint { lo: 0, hi: 87 },
    OptimalEndpoint { lo: 0, hi: 88 },
    OptimalEndpoint { lo: 0, hi: 90 },
    OptimalEndpoint { lo: 0, hi: 91 },
    OptimalEndpoint { lo: 0, hi: 93 },
    OptimalEndpoint { lo: 0, hi: 94 },
    OptimalEndpoint { lo: 0, hi: 96 },
    OptimalEndpoint { lo: 0, hi: 97 },
    OptimalEndpoint { lo: 0, hi: 99 },
    OptimalEndpoint { lo: 0, hi: 100 },
    OptimalEndpoint { lo: 0, hi: 102 },
    OptimalEndpoint { lo: 0, hi: 103 },
    OptimalEndpoint { lo: 0, hi: 105 },
    OptimalEndpoint { lo: 0, hi: 106 },
    OptimalEndpoint { lo: 0, hi: 108 },
    OptimalEndpoint { lo: 0, hi: 109 },
    OptimalEndpoint { lo: 0, hi: 111 },
    OptimalEndpoint { lo: 0, hi: 112 },
    OptimalEndpoint { lo: 0, hi: 114 },
    OptimalEndpoint { lo: 0, hi: 116 },
    OptimalEndpoint { lo: 0, hi: 117 },
    OptimalEndpoint { lo: 0, hi: 119 },
    OptimalEndpoint { lo: 0, hi: 120 },
    OptimalEndpoint { lo: 0, hi: 122 },
    OptimalEndpoint { lo: 0, hi: 123 },
    OptimalEndpoint { lo: 0, hi: 125 },
    OptimalEndpoint { lo: 0, hi: 126 },
    OptimalEndpoint { lo: 1, hi: 126 },
    OptimalEndpoint { lo: 1, hi: 127 },
    OptimalEndpoint { lo: 2, hi: 127 },
    OptimalEndpoint { lo: 3, hi: 126 },
    OptimalEndpoint { lo: 4, hi: 126 },
    OptimalEndpoint { lo: 4, hi: 127 },
    OptimalEndpoint { lo: 5, hi: 127 },
    OptimalEndpoint { lo: 6, hi: 126 },
    OptimalEndpoint { lo: 7, hi: 126 },
    OptimalEndpoint { lo: 7, hi: 127 },
    OptimalEndpoint { lo: 8, hi: 127 },
    OptimalEndpoint { lo: 9, hi: 126 },
    OptimalEndpoint { lo: 10, hi: 126 },
    OptimalEndpoint { lo: 10, hi: 127 },
    OptimalEndpoint { lo: 11, hi: 127 },
    OptimalEndpoint { lo: 12, hi: 126 },
    OptimalEndpoint { lo: 13, hi: 125 },
    OptimalEndpoint { lo: 13, hi: 127 },
    OptimalEndpoint { lo: 14, hi: 126 },
    OptimalEndpoint { lo: 15, hi: 126 },
    OptimalEndpoint { lo: 15, hi: 127 },
    OptimalEndpoint { lo: 16, hi: 127 },
    OptimalEndpoint { lo: 17, hi: 126 },
    OptimalEndpoint { lo: 18, hi: 126 },
    OptimalEndpoint { lo: 18, hi: 127 },
    OptimalEndpoint { lo: 19, hi: 127 },
    OptimalEndpoint { lo: 20, hi: 126 },
    OptimalEndpoint { lo: 21, hi: 126 },
    OptimalEndpoint { lo: 21, hi: 127 },
    OptimalEndpoint { lo: 22, hi: 127 },
    OptimalEndpoint { lo: 23, hi: 126 },
    OptimalEndpoint { lo: 24, hi: 126 },
    OptimalEndpoint { lo: 24, hi: 127 },
    OptimalEndpoint { lo: 25, hi: 127 },
    OptimalEndpoint { lo: 26, hi: 126 },
    OptimalEndpoint { lo: 27, hi: 126 },
    OptimalEndpoint { lo: 27, hi: 127 },
    OptimalEndpoint { lo: 28, hi: 127 },
    OptimalEndpoint { lo: 29, hi: 126 },
    OptimalEndpoint { lo: 30, hi: 126 },
    OptimalEndpoint { lo: 30, hi: 127 },
    OptimalEndpoint { lo: 31, hi: 127 },
    OptimalEndpoint { lo: 32, hi: 126 },
    OptimalEndpoint { lo: 33, hi: 126 },
    OptimalEndpoint { lo: 33, hi: 127 },
    OptimalEndpoint { lo: 34, hi: 127 },
    OptimalEndpoint { lo: 35, hi: 126 },
    OptimalEndpoint { lo: 36, hi: 126 },
    OptimalEndpoint { lo: 36, hi: 127 },
    OptimalEndpoint { lo: 37, hi: 127 },
    OptimalEndpoint { lo: 38, hi: 126 },
    OptimalEndpoint { lo: 39, hi: 126 },
    OptimalEndpoint { lo: 39, hi: 127 },
    OptimalEndpoint { lo: 40, hi: 127 },
    OptimalEndpoint { lo: 41, hi: 126 },
    OptimalEndpoint { lo: 42, hi: 126 },
    OptimalEndpoint { lo: 42, hi: 127 },
    OptimalEndpoint { lo: 43, hi: 127 },
    OptimalEndpoint { lo: 44, hi: 126 },
    OptimalEndpoint { lo: 45, hi: 125 },
    OptimalEndpoint { lo: 45, hi: 127 },
    OptimalEndpoint { lo: 46, hi: 126 },
    OptimalEndpoint { lo: 47, hi: 126 },
    OptimalEndpoint { lo: 47, hi: 127 },
    OptimalEndpoint { lo: 48, hi: 127 },
    OptimalEndpoint { lo: 49, hi: 126 },
    OptimalEndpoint { lo: 50, hi: 126 },
    OptimalEndpoint { lo: 50, hi: 127 },
    OptimalEndpoint { lo: 51, hi: 127 },
    OptimalEndpoint { lo: 52, hi: 126 },
    OptimalEndpoint { lo: 53, hi: 126 },
    OptimalEndpoint { lo: 53, hi: 127 },
    OptimalEndpoint { lo: 54, hi: 127 },
    OptimalEndpoint { lo: 55, hi: 126 },
    OptimalEndpoint { lo: 56, hi: 126 },
    OptimalEndpoint { lo: 56, hi: 127 },
    OptimalEndpoint { lo: 57, hi: 127 },
    OptimalEndpoint { lo: 58, hi: 126 },
    OptimalEndpoint { lo: 59, hi: 126 },
    OptimalEndpoint { lo: 59, hi: 127 },
    OptimalEndpoint { lo: 60, hi: 127 },
    OptimalEndpoint { lo: 61, hi: 126 },
    OptimalEndpoint { lo: 62, hi: 126 },
    OptimalEndpoint { lo: 62, hi: 127 },
    OptimalEndpoint { lo: 63, hi: 127 },
    OptimalEndpoint { lo: 64, hi: 126 },
    OptimalEndpoint { lo: 65, hi: 126 },
    OptimalEndpoint { lo: 65, hi: 127 },
    OptimalEndpoint { lo: 66, hi: 127 },
    OptimalEndpoint { lo: 67, hi: 126 },
    OptimalEndpoint { lo: 68, hi: 126 },
    OptimalEndpoint { lo: 68, hi: 127 },
    OptimalEndpoint { lo: 69, hi: 127 },
    OptimalEndpoint { lo: 70, hi: 126 },
    OptimalEndpoint { lo: 71, hi: 126 },
    OptimalEndpoint { lo: 71, hi: 127 },
    OptimalEndpoint { lo: 72, hi: 127 },
    OptimalEndpoint { lo: 73, hi: 126 },
    OptimalEndpoint { lo: 74, hi: 126 },
    OptimalEndpoint { lo: 74, hi: 127 },
    OptimalEndpoint { lo: 75, hi: 127 },
    OptimalEndpoint { lo: 76, hi: 126 },
    OptimalEndpoint { lo: 77, hi: 125 },
    OptimalEndpoint { lo: 77, hi: 127 },
    OptimalEndpoint { lo: 78, hi: 126 },
    OptimalEndpoint { lo: 79, hi: 126 },
    OptimalEndpoint { lo: 79, hi: 127 },
    OptimalEndpoint { lo: 80, hi: 127 },
    OptimalEndpoint { lo: 81, hi: 126 },
    OptimalEndpoint { lo: 82, hi: 126 },
    OptimalEndpoint { lo: 82, hi: 127 },
    OptimalEndpoint { lo: 83, hi: 127 },
    OptimalEndpoint { lo: 84, hi: 126 },
    OptimalEndpoint { lo: 85, hi: 126 },
    OptimalEndpoint { lo: 85, hi: 127 },
    OptimalEndpoint { lo: 86, hi: 127 },
    OptimalEndpoint { lo: 87, hi: 126 },
    OptimalEndpoint { lo: 88, hi: 126 },
    OptimalEndpoint { lo: 88, hi: 127 },
    OptimalEndpoint { lo: 89, hi: 127 },
    OptimalEndpoint { lo: 90, hi: 126 },
    OptimalEndpoint { lo: 91, hi: 126 },
    OptimalEndpoint { lo: 91, hi: 127 },
    OptimalEndpoint { lo: 92, hi: 127 },
    OptimalEndpoint { lo: 93, hi: 126 },
    OptimalEndpoint { lo: 94, hi: 126 },
    OptimalEndpoint { lo: 94, hi: 127 },
    OptimalEndpoint { lo: 95, hi: 127 },
    OptimalEndpoint { lo: 96, hi: 126 },
    OptimalEndpoint { lo: 97, hi: 126 },
    OptimalEndpoint { lo: 97, hi: 127 },
    OptimalEndpoint { lo: 98, hi: 127 },
    OptimalEndpoint { lo: 99, hi: 126 },
    OptimalEndpoint { lo: 100, hi: 126 },
    OptimalEndpoint { lo: 100, hi: 127 },
    OptimalEndpoint { lo: 101, hi: 127 },
    OptimalEndpoint { lo: 102, hi: 126 },
    OptimalEndpoint { lo: 103, hi: 126 },
    OptimalEndpoint { lo: 103, hi: 127 },
    OptimalEndpoint { lo: 104, hi: 127 },
    OptimalEndpoint { lo: 105, hi: 126 },
    OptimalEndpoint { lo: 106, hi: 126 },
    OptimalEndpoint { lo: 106, hi: 127 },
    OptimalEndpoint { lo: 107, hi: 127 },
    OptimalEndpoint { lo: 108, hi: 126 },
    OptimalEndpoint { lo: 109, hi: 125 },
    OptimalEndpoint { lo: 109, hi: 127 },
    OptimalEndpoint { lo: 110, hi: 126 },
    OptimalEndpoint { lo: 111, hi: 126 },
    OptimalEndpoint { lo: 111, hi: 127 },
    OptimalEndpoint { lo: 112, hi: 127 },
    OptimalEndpoint { lo: 113, hi: 126 },
    OptimalEndpoint { lo: 114, hi: 126 },
    OptimalEndpoint { lo: 114, hi: 127 },
    OptimalEndpoint { lo: 115, hi: 127 },
    OptimalEndpoint { lo: 116, hi: 126 },
    OptimalEndpoint { lo: 117, hi: 126 },
    OptimalEndpoint { lo: 117, hi: 127 },
    OptimalEndpoint { lo: 118, hi: 127 },
    OptimalEndpoint { lo: 119, hi: 126 },
    OptimalEndpoint { lo: 120, hi: 126 },
    OptimalEndpoint { lo: 120, hi: 127 },
    OptimalEndpoint { lo: 121, hi: 127 },
    OptimalEndpoint { lo: 122, hi: 126 },
    OptimalEndpoint { lo: 123, hi: 126 },
    OptimalEndpoint { lo: 123, hi: 127 },
    OptimalEndpoint { lo: 124, hi: 127 },
    OptimalEndpoint { lo: 125, hi: 126 },
    OptimalEndpoint { lo: 126, hi: 126 },
    OptimalEndpoint { lo: 126, hi: 127 },
    OptimalEndpoint { lo: 127, hi: 127 },
    OptimalEndpoint { lo: 127, hi: 127 },
];

const fn mode_6_optimal_endpoint(c: u8, p_bit: bool) -> OptimalEndpoint {
    // Exploiting the fact that `ENDPOINTS[c+1] (when p_bit==0) == ENDPOINTS[c] (when p_bit==1)`,
    // we can use the same table for both values of `p_bits`.
    let i = c as usize + (!p_bit) as usize;
    BC7_MODE_6_OPTIMAL_ENDPOINTS[i]
}

const fn mode_6_optimal_endpoint_err(c: u8, p_bit: bool) -> u32 {
    // Only the two extreme endpoints have error == 1, others have 0.
    ((c == 0 && p_bit) || (c == 255 && !p_bit)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_8_bc7_mode_5_optimal_endpoints_lut_is_valid() {
        assert_eq!(
            BC7_MODE_5_OPTIMAL_ENDPOINTS,
            build_mode_8_bc7_mode_5_table()
        );
    }

    #[test]
    fn mode_8_bc7_mode_6_optimal_endpoints_lut_is_valid() {
        assert_eq!(
            BC7_MODE_6_OPTIMAL_ENDPOINTS[1..],
            build_mode_8_bc7_mode_6_table()
        );
    }

    fn build_mode_8_bc7_mode_6_table() -> [OptimalEndpoint; 256] {
        const DEFAULT: OptimalEndpoint = OptimalEndpoint { lo: 0, hi: 0 };
        let mut bc7_mode_6_optimal_endpoints = [DEFAULT; 256];

        const WEIGHTS4: [u8; 16] = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];

        // BC7 777.1
        for c in 0..256i32 {
            #[cfg(test)]
            let range = 0..2;
            #[cfg(not(test))]
            let range = 0..1;
            for lp in range {
                let mut best = OptimalEndpoint { lo: 0, hi: 0 };
                let mut best_err = i32::MAX;

                for l in 0..128i32 {
                    let low = (l << 1) | lp;

                    for h in l..128i32 {
                        let high = (h << 1) | lp;

                        let w = WEIGHTS4[BC7ENC_MODE_6_OPTIMAL_INDEX as usize] as i32;
                        let k = (low * (64 - w) + high * w + 32) >> 6;

                        let err = (k - c) * (k - c);
                        if err < best_err {
                            best_err = err;
                            best.lo = l as u8;
                            best.hi = h as u8;
                        }
                    } // h
                } // l

                #[cfg(test)]
                if best_err != mode_6_optimal_endpoint_err(c as u8, lp != 0) as i32 {
                    panic!();
                }

                if lp == 0 {
                    bc7_mode_6_optimal_endpoints[c as usize] = best;
                }

                #[cfg(test)]
                if lp == 1 {
                    let prev = bc7_mode_6_optimal_endpoints[(c as usize).saturating_sub(1)];
                    if c > 0 && best.lo != prev.lo && best.hi != prev.hi {
                        panic!();
                    }
                }
            } // lp
        } // c

        bc7_mode_6_optimal_endpoints
    }

    fn build_mode_8_bc7_mode_5_table() -> [OptimalEndpoint; 256] {
        const DEFAULT: OptimalEndpoint = OptimalEndpoint { lo: 0, hi: 0 };
        let mut bc7_mode_5_optimal_endpoints = [DEFAULT; 256];

        const WEIGHTS2: [u8; 4] = [0, 21, 43, 64];

        // BC7 777
        for c in 0..256 {
            let mut best = OptimalEndpoint { lo: 0, hi: 0 };
            let mut best_err = i32::MAX;

            for l in 0..128 {
                let low = (l << 1) | (l >> 6);

                for h in l..128 {
                    let high = (h << 1) | (h >> 6);

                    let w = WEIGHTS2[BC7ENC_MODE_5_OPTIMAL_INDEX as usize] as i32;
                    let k = (low * (64 - w) + high * w + 32) >> 6;

                    let err = (k - c) * (k - c);
                    if err < best_err {
                        best_err = err;
                        best.lo = l as u8;
                        best.hi = h as u8;
                    }
                } // h
            } // l

            #[cfg(test)]
            assert_eq!(best_err, 0);

            bc7_mode_5_optimal_endpoints[c as usize] = best;
        } // c

        bc7_mode_5_optimal_endpoints
    }
}
