# Cube LUT Format parser

Based on [resolve spec](https://resolve.cafe/developers/luts/).

Supports 1D, 3D with optional 1D shaper.

`Lut::meta()` exposes the title, comments, video range flags, domain bounds,
and optional `LUT_3D_SIZE` through `Meta::size()`. To read only the header without allocating cubes or parsing sample
values, use `Lut::parse_meta(&mut reader)`. It returns a `Meta` and stops after
reading the first sample line (or at EOF for a header-only file). Comments after
sample data are not included.
