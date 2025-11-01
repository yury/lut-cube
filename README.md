# Cube LUT Format parser

Based on [resolve spec](https://resolve.cafe/developers/luts/).

Supports 1D, 3D with optional 1D shaper.

## Features

- **LUT Parsing**: Parse .cube files with 1D, 3D LUTs and optional 1D shapers
- **LUT Application**: High-performance LUT application with SIMD optimizations
  - x86_64: AVX2 support (runtime detected)
  - aarch64: NEON support
  - Portable scalar fallback for all platforms

## Usage

### Parsing .cube files

```rust
use lut_cube::Lut;
use std::io::BufReader;
use std::fs::File;

let mut reader = BufReader::new(File::open("my_lut.cube")?);
let lut = Lut::parse(&mut reader)?;
```

### Applying a LUT

```rust
use lut_cube::apply_lut_auto;

// Create a simple inverting LUT
let mut lut = [0u8; 256];
for i in 0..256 {
    lut[i] = (255 - i) as u8;
}

let src = vec![0u8, 127, 255];
let mut dst = vec![0u8; 3];

apply_lut_auto(&src, &mut dst, &lut);
assert_eq!(dst, vec![255, 128, 0]);
```

## Benchmarks

Run benchmarks locally:

```bash
cargo bench
```

See [benches/README.md](benches/README.md) for more details.

## Optional Features

- `simd`: Marker feature for SIMD-optimized code (always enabled on x86_64 and aarch64)
- `rayon`: Enable parallel processing support (not yet implemented)
