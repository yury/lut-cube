# Cube LUT Format parser

Based on [resolve spec](https://resolve.cafe/developers/luts/).

Supports 1D, 3D with optional 1D shaper.

`Lut::meta()` exposes the title, comments, video range flags, domain bounds,
and optional `LUT_3D_SIZE` through `Meta::size()`. To read only the header without allocating cubes or parsing sample
values, use `Lut::parse_meta(&mut reader)`. It returns a `Meta` and stops after
reading the first sample line (or at EOF for a header-only file). Comments after
sample data are not included.

To cap table allocations while parsing samples and metadata in a single pass:

```rust,no_run
use lut_cube::{Lut, ParseLimits};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut reader = std::io::BufReader::new(std::fs::File::open("look.cube")?);
let lut = Lut::parse_with_limits(&mut reader, ParseLimits {
    max_1d_size: 0,   // Disallow 1D tables, including shapers.
    max_3d_size: 256, // Maximum 3D edge length (256 cubed RGB entries).
})?;
# Ok(())
# }
```

Every `LUT_1D_SIZE` and `LUT_3D_SIZE` declaration is checked before allocating its
table, including repeated declarations and shapers. Valid sizes start at 2; a zero
limit disables that dimension. `Lut::parse` uses `ParseLimits::default()`, whose
limits are `u16::MAX`. `Lut::parse_meta` still reports header metadata without
applying allocation limits, since it does not allocate sample tables.
