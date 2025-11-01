# LUT Application Benchmarks

This directory contains Criterion benchmarks for the LUT (Look-Up Table) application functionality.

## Running Benchmarks

To run all benchmarks:

```bash
cargo bench
```

To run a specific benchmark group:

```bash
cargo bench lut_application
cargo bench lut_patterns
cargo bench lut_alignment
```

To run benchmarks with a specific baseline for comparison:

```bash
cargo bench --bench lut_cube -- --save-baseline my-baseline
```

## Benchmark Groups

### lut_application
Measures performance across different buffer sizes:
- **small_1KB**: 1 KiB buffer
- **medium_64KB**: 64 KiB buffer
- **large_1MB**: 1 MiB buffer
- **xlarge_4MB**: 4 MiB buffer

These benchmarks help identify how well the SIMD optimizations scale with data size.

### lut_patterns
Measures performance with different LUT patterns on 1 MiB buffers:
- **identity**: Pass-through LUT (output = input)
- **invert**: Inverting LUT (output = 255 - input)
- **gamma**: Gamma correction curve

### lut_alignment
Measures performance with different memory alignments:
- **aligned**: Buffers aligned to natural boundaries
- **unaligned_1**: Buffers offset by 1 byte
- **unaligned_7**: Buffers offset by 7 bytes

## SIMD Optimizations

The `apply_lut_auto` function automatically selects the best implementation:

- **x86_64**: Uses AVX2 instructions when available (detected at runtime via `is_x86_feature_detected!("avx2")`), falls back to scalar
- **aarch64**: Uses ARM NEON instructions (always available on this architecture)
- **Other platforms**: Uses optimized scalar implementation with loop unrolling

## Benchmark Results

Results are saved in `target/criterion/` and include:
- HTML reports with graphs
- Statistical analysis (mean, median, std deviation)
- Comparison with previous runs

Open `target/criterion/report/index.html` in a browser to view detailed results.

## CI Integration

Benchmarks can be run in CI but are not enabled by default due to time and cost considerations.
See `.github/workflows/bench.yml` for the CI configuration.
