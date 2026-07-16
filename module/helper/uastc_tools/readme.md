# 🧱 uastc_tools

**Pure-Rust UASTC (Basis Universal) block transcoder — UASTC → BC7 / ASTC / ETC / RGBA**

`uastc_tools` converts **UASTC LDR 4×4** texture blocks into the GPU-compressed format a
device actually supports. It is the transcode step behind cgtools' `KHR_texture_basisu`
(KTX2) support in the `renderer` glTF loader.

It is `no_std`, `forbid(unsafe_code)`, has **zero dependencies**, and builds for
`wasm32-unknown-unknown`.

## directory layout

`Origin` matters more here than in a normal crate: **only `lib.rs` is cgtools-authored.**
Everything under it is upstream code kept byte-for-byte, so it can still be diffed against
the recorded commit — see [Provenance](#️-provenance--this-is-vendored-third-party-code).

| Path | Origin | Responsibility |
|------|--------|----------------|
| `src/lib.rs` | **cgtools** | Crate entry point — the five public entry points (`unpack_uastc_block_to_rgba`, `transcode_uastc_block_to_{bc7,astc,etc1,etc2}`), the `Error` / `Result` aliases, the `mask!` helper macro, and the crate-boundary lint relaxation for the vendored modules below. No decode logic of its own. |
| `src/uastc.rs` | vendored | The UASTC LDR 4×4 **block decoder** — mode parsing (`decode_mode`), subset patterns, endpoint unquantisation (`decode_endpoints`), weight decoding (`decode_weights`), and `decode_block_to_rgba`. Also defines the block-size constants every target format keys off. The core of the crate. |
| `src/target_formats/mod.rs` | vendored | Layer entry point for the three transcoders. |
| `src/target_formats/bc7.rs` | vendored | **UASTC → BC7** (`convert_block_from_uastc`). The desktop path (`EXT_texture_compression_bptc`). Largest file in the crate — BC7's mode space is wide. |
| `src/target_formats/astc.rs` | vendored | **UASTC → ASTC 4×4** (`convert_block_from_uastc`). The mobile path (`WEBGL_compressed_texture_astc`). Small, because UASTC was designed as a near-subset of ASTC. |
| `src/target_formats/etc.rs` | vendored | **UASTC → ETC1 / ETC2** (`convert_etc1_block_from_uastc`, `convert_etc2_block_from_uastc`). Older-mobile fallback (`WEBGL_compressed_texture_etc`). |
| `src/bitreader.rs` | vendored | `BitReaderLsb` — LSB-first bit reader used to pull fields out of a packed UASTC block. |
| `src/bitwriter.rs` | vendored | `BitWriterLsb` — LSB-first bit writer used by the transcoders to *emit* packed BC7 / ASTC / ETC blocks. |
| `src/color.rs` | vendored | `Color32` — the RGBA texel type. Note `to_rgba_u32` packs **RGBA**, not BGRA. |
| `readme.md` | cgtools | This file: provenance, scope, and the validation result. |
| `license-mit`, `license-apache` | upstream | Upstream's dual license, shipped unmodified as the license requires. |

## ⚠️ Provenance — this is vendored third-party code

**This crate is not original work.** It is a vendored subset of
**[`basisu_rs`](https://github.com/JakubValtar/basisu_rs) by Jakub Valtar**, used under its
**MIT OR Apache-2.0** dual license (both license texts ship alongside this readme, as
`license-mit` and `license-apache`).

| | |
|---|---|
| **Upstream** | https://github.com/JakubValtar/basisu_rs |
| **Author** | Jakub Valtar |
| **Commit vendored** | `60e1bcb6a914be55b7bc2c212f37d34e022cb0ec` (2023-05-10) |
| **License** | MIT OR Apache-2.0 (unchanged) |

### Why vendored rather than depended upon

`basisu_rs` **was never published to crates.io** — its readme still describes the crate
release as pending, and the repository has been dormant since May 2023. A `git` dependency
would make every crate that transitively depends on it **unpublishable**, since crates.io
rejects packages with non-registry dependencies. Vendoring is the only route that keeps
`renderer` releasable.

The practical consequence is that **cgtools now owns this code**: upstream fixes will not
flow to us automatically, and bugs found here are ours to fix. The commit SHA above is
recorded so the vendored tree can still be diffed against upstream.

### What was vendored, and what was left behind

Only the **UASTC** decode/transcode path was taken (~3,500 of upstream's ~4,900 lines):

| vendored | dropped |
|---|---|
| `uastc.rs` — UASTC block decoder | `basis.rs` — `.basis` container reader |
| `target_formats/{bc7,astc,etc}.rs` | `basis_lz/` — ETC1S / BasisLZ decoder |
| `bitreader.rs`, `bitwriter.rs`, `color.rs` | `bytereader.rs` (only used by the above) |

The dropped code reads the **`.basis` container**, which cgtools does not use — we read
UASTC out of **KTX2** containers via the [`ktx2`](https://crates.io/crates/ktx2) crate
instead. **ETC1S/BasisLZ is out of scope** for the current `KHR_texture_basisu` support.
Cutting them also removes upstream's only dependency (`byteorder`), which nothing in the
UASTC path used.

### Validation

Upstream is unmaintained and was never independently validated, so cgtools validated it
before adopting it. Decoding real `KHR_texture_basisu` assets, this code is **bit-exact
with the reference implementation** (Khronos KTX-Software 4.4.2 `ktx extract`) — maximum
per-channel difference `0`, **100 % of pixels identical**, across three 1024×1024 textures
fed identical UASTC input. Its BC7 output was separately cross-checked by decoding it back
with an independent decoder (`texture2ddecoder`).

## 🚀 Usage

Each entry point takes one 128-bit UASTC block and returns one block in the target format.
Blocks are 16 bytes in, 16 bytes out (BC7 and ASTC 4×4 are the same size as UASTC); the
RGBA fallback returns the block's 16 texels.

```rust,ignore
// One 4x4 UASTC block, e.g. sliced out of a decompressed KTX2 mip level.
let block : [ u8; 16 ] = /* ... */;

// Desktop: EXT_texture_compression_bptc
let bc7 : [ u8; 16 ] = uastc_tools::transcode_uastc_block_to_bc7( block )?;

// Mobile: WEBGL_compressed_texture_astc
let astc : [ u8; 16 ] = uastc_tools::transcode_uastc_block_to_astc( block )?;

// Fallback: no compressed format supported. 16 texels, each packed [ r, g, b, a ].
let rgba : [ u32; 16 ] = uastc_tools::unpack_uastc_block_to_rgba( block )?;
```

**Byte-order note:** `unpack_uastc_block_to_rgba` packs each texel as **RGBA**
(`u32::from_le_bytes( [ r, g, b, a ] )`). Some other block decoders — `texture2ddecoder`,
for instance — pack **BGRA**. Mixing them up silently swaps the red and blue channels.

## 📦 Scope

- **UASTC LDR 4×4 only.** No ETC1S/BasisLZ, no UASTC HDR, no `.basis` containers.
- **Decode only.** There is no encoder; produce UASTC with
  [`gltf-transform`](https://gltf-transform.dev) or KTX-Software's `ktx` CLI.
- **Block-level only.** Mip chains, supercompression (zstd) and GPU upload are the caller's
  job — in cgtools that is `renderer`'s KTX2 loader.
