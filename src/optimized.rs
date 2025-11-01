//! Optimized LUT application with SIMD support
//!
//! This module provides optimized implementations for applying a 1D LUT to image data,
//! with platform-specific SIMD optimizations and a safe scalar fallback.

/// Apply a 1D LUT to source data using a safe scalar implementation.
///
/// This is the baseline implementation that uses basic loop unrolling
/// for better performance while remaining completely safe.
/// Exported for testing and benchmarking purposes.
///
/// # Arguments
/// * `src` - Source byte array (must have same length as dst)
/// * `dst` - Destination byte array (must have same length as src)
/// * `lut` - 256-entry lookup table
///
/// # Panics
/// Panics if src and dst have different lengths.
#[inline]
#[allow(dead_code)]
fn apply_lut_basic(src: &[u8], dst: &mut [u8], lut: &[u8; 256]) {
    assert_eq!(src.len(), dst.len(), "src and dst must have same length");
    
    let len = src.len();
    let chunks = len / 4;
    let remainder = len % 4;
    
    // Process 4 bytes at a time for better instruction-level parallelism
    for i in 0..chunks {
        let idx = i * 4;
        dst[idx] = lut[src[idx] as usize];
        dst[idx + 1] = lut[src[idx + 1] as usize];
        dst[idx + 2] = lut[src[idx + 2] as usize];
        dst[idx + 3] = lut[src[idx + 3] as usize];
    }
    
    // Handle remaining bytes
    let start = chunks * 4;
    for i in 0..remainder {
        dst[start + i] = lut[src[start + i] as usize];
    }
}

#[cfg(target_arch = "x86_64")]
/// Apply a 1D LUT using an optimized scalar implementation.
///
/// Note: True SIMD optimization for arbitrary 256-entry LUTs is complex because
/// x86 shuffle instructions (pshufb) work with 16-entry tables. While it's possible
/// to implement using multiple passes with nibble extraction, the overhead often
/// exceeds the benefit for random LUT values. This implementation uses aggressive
/// loop unrolling which the compiler can auto-vectorize.
///
/// # Arguments
/// * `src` - Source byte array (must have same length as dst)
/// * `dst` - Destination byte array (must have same length as src)
/// * `lut` - 256-entry lookup table
#[inline]
fn apply_lut_x86_impl(src: &[u8], dst: &mut [u8], lut: &[u8; 256]) {
    let len = src.len();
    let chunks = len / 16;
    let processed = chunks * 16;
    
    // Process 16 bytes at a time with aggressive unrolling
    // The compiler can auto-vectorize this with AVX2 if beneficial
    for i in 0..chunks {
        let idx = i * 16;
        unsafe {
            // Use unsafe for bounds check elimination
            *dst.get_unchecked_mut(idx + 0) = *lut.get_unchecked(*src.get_unchecked(idx + 0) as usize);
            *dst.get_unchecked_mut(idx + 1) = *lut.get_unchecked(*src.get_unchecked(idx + 1) as usize);
            *dst.get_unchecked_mut(idx + 2) = *lut.get_unchecked(*src.get_unchecked(idx + 2) as usize);
            *dst.get_unchecked_mut(idx + 3) = *lut.get_unchecked(*src.get_unchecked(idx + 3) as usize);
            *dst.get_unchecked_mut(idx + 4) = *lut.get_unchecked(*src.get_unchecked(idx + 4) as usize);
            *dst.get_unchecked_mut(idx + 5) = *lut.get_unchecked(*src.get_unchecked(idx + 5) as usize);
            *dst.get_unchecked_mut(idx + 6) = *lut.get_unchecked(*src.get_unchecked(idx + 6) as usize);
            *dst.get_unchecked_mut(idx + 7) = *lut.get_unchecked(*src.get_unchecked(idx + 7) as usize);
            *dst.get_unchecked_mut(idx + 8) = *lut.get_unchecked(*src.get_unchecked(idx + 8) as usize);
            *dst.get_unchecked_mut(idx + 9) = *lut.get_unchecked(*src.get_unchecked(idx + 9) as usize);
            *dst.get_unchecked_mut(idx + 10) = *lut.get_unchecked(*src.get_unchecked(idx + 10) as usize);
            *dst.get_unchecked_mut(idx + 11) = *lut.get_unchecked(*src.get_unchecked(idx + 11) as usize);
            *dst.get_unchecked_mut(idx + 12) = *lut.get_unchecked(*src.get_unchecked(idx + 12) as usize);
            *dst.get_unchecked_mut(idx + 13) = *lut.get_unchecked(*src.get_unchecked(idx + 13) as usize);
            *dst.get_unchecked_mut(idx + 14) = *lut.get_unchecked(*src.get_unchecked(idx + 14) as usize);
            *dst.get_unchecked_mut(idx + 15) = *lut.get_unchecked(*src.get_unchecked(idx + 15) as usize);
        }
    }
    
    // Handle remaining bytes
    for i in processed..len {
        dst[i] = lut[src[i] as usize];
    }
}

#[cfg(target_arch = "x86_64")]
/// Apply a 1D LUT using x86_64 optimized implementation.
///
/// This uses an aggressively unrolled loop that the compiler can auto-vectorize
/// when AVX2 is available at compile time.
pub fn apply_lut_x86_avx2(src: &[u8], dst: &mut [u8], lut: &[u8; 256]) {
    assert_eq!(src.len(), dst.len(), "src and dst must have same length");
    apply_lut_x86_impl(src, dst, lut);
}

#[cfg(target_arch = "aarch64")]
/// Apply a 1D LUT using ARM NEON optimized implementation.
///
/// Note: True SIMD optimization for arbitrary 256-entry LUTs is complex because
/// NEON's vtbl instruction works with 32-64 entry tables. While it's possible
/// to implement using multiple vtbl passes with byte range checks, the overhead
/// often exceeds the benefit. This implementation uses aggressive loop unrolling
/// which the compiler can auto-vectorize with NEON.
///
/// # Arguments
/// * `src` - Source byte array (must have same length as dst)
/// * `dst` - Destination byte array (must have same length as src)
/// * `lut` - 256-entry lookup table
pub fn apply_lut_aarch64_neon(src: &[u8], dst: &mut [u8], lut: &[u8; 256]) {
    assert_eq!(src.len(), dst.len(), "src and dst must have same length");
    
    let len = src.len();
    let chunks = len / 16;
    let processed = chunks * 16;
    
    // Process 16 bytes at a time with aggressive unrolling
    // The compiler can auto-vectorize this with NEON if beneficial
    for i in 0..chunks {
        let idx = i * 16;
        unsafe {
            // Use unsafe for bounds check elimination
            *dst.get_unchecked_mut(idx + 0) = *lut.get_unchecked(*src.get_unchecked(idx + 0) as usize);
            *dst.get_unchecked_mut(idx + 1) = *lut.get_unchecked(*src.get_unchecked(idx + 1) as usize);
            *dst.get_unchecked_mut(idx + 2) = *lut.get_unchecked(*src.get_unchecked(idx + 2) as usize);
            *dst.get_unchecked_mut(idx + 3) = *lut.get_unchecked(*src.get_unchecked(idx + 3) as usize);
            *dst.get_unchecked_mut(idx + 4) = *lut.get_unchecked(*src.get_unchecked(idx + 4) as usize);
            *dst.get_unchecked_mut(idx + 5) = *lut.get_unchecked(*src.get_unchecked(idx + 5) as usize);
            *dst.get_unchecked_mut(idx + 6) = *lut.get_unchecked(*src.get_unchecked(idx + 6) as usize);
            *dst.get_unchecked_mut(idx + 7) = *lut.get_unchecked(*src.get_unchecked(idx + 7) as usize);
            *dst.get_unchecked_mut(idx + 8) = *lut.get_unchecked(*src.get_unchecked(idx + 8) as usize);
            *dst.get_unchecked_mut(idx + 9) = *lut.get_unchecked(*src.get_unchecked(idx + 9) as usize);
            *dst.get_unchecked_mut(idx + 10) = *lut.get_unchecked(*src.get_unchecked(idx + 10) as usize);
            *dst.get_unchecked_mut(idx + 11) = *lut.get_unchecked(*src.get_unchecked(idx + 11) as usize);
            *dst.get_unchecked_mut(idx + 12) = *lut.get_unchecked(*src.get_unchecked(idx + 12) as usize);
            *dst.get_unchecked_mut(idx + 13) = *lut.get_unchecked(*src.get_unchecked(idx + 13) as usize);
            *dst.get_unchecked_mut(idx + 14) = *lut.get_unchecked(*src.get_unchecked(idx + 14) as usize);
            *dst.get_unchecked_mut(idx + 15) = *lut.get_unchecked(*src.get_unchecked(idx + 15) as usize);
        }
    }
    
    // Handle remaining bytes
    for i in processed..len {
        dst[i] = lut[src[i] as usize];
    }
}

/// Apply a 1D LUT with automatic selection of the best available implementation.
///
/// This is the main public API that automatically selects the best implementation
/// based on the current platform:
/// - x86_64: Optimized implementation with aggressive loop unrolling (compiler can auto-vectorize with AVX2)
/// - aarch64: Optimized implementation with aggressive loop unrolling (compiler can auto-vectorize with NEON)
/// - Other platforms: Safe scalar implementation with loop unrolling
///
/// Note: Direct SIMD intrinsics for arbitrary 256-entry LUTs are complex and often
/// provide minimal benefit over compiler auto-vectorization for this use case.
///
/// # Arguments
/// * `src` - Source byte array
/// * `dst` - Destination byte array (must have same length as src)
/// * `lut` - 256-entry lookup table mapping input bytes to output bytes
///
/// # Panics
/// Panics if src and dst have different lengths.
///
/// # Example
/// ```rust
/// use lut_cube::apply_lut_auto;
///
/// let src = vec![0u8, 127, 255];
/// let mut dst = vec![0u8; 3];
/// let mut lut = [0u8; 256];
/// 
/// // Create an inverting LUT
/// for i in 0..256 {
///     lut[i] = (255 - i) as u8;
/// }
///
/// apply_lut_auto(&src, &mut dst, &lut);
/// assert_eq!(dst, vec![255, 128, 0]);
/// ```
pub fn apply_lut_auto(src: &[u8], dst: &mut [u8], lut: &[u8; 256]) {
    assert_eq!(src.len(), dst.len(), "src and dst must have same length");
    
    #[cfg(target_arch = "x86_64")]
    {
        apply_lut_x86_avx2(src, dst, lut);
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        apply_lut_aarch64_neon(src, dst, lut);
    }
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        apply_lut_basic(src, dst, lut);
    }
}

/// Convenience function that maintains backward compatibility.
/// Delegates to apply_lut_auto.
pub fn apply_lut(src: &[u8], dst: &mut [u8], lut: &[u8; 256]) {
    apply_lut_auto(src, dst, lut);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_identity_lut() -> [u8; 256] {
        let mut lut = [0u8; 256];
        for i in 0..256 {
            lut[i] = i as u8;
        }
        lut
    }

    fn create_invert_lut() -> [u8; 256] {
        let mut lut = [0u8; 256];
        for i in 0..256 {
            lut[i] = (255 - i) as u8;
        }
        lut
    }

    fn create_threshold_lut(threshold: u8) -> [u8; 256] {
        let mut lut = [0u8; 256];
        for i in 0..256 {
            lut[i] = if i < threshold as usize { 0 } else { 255 };
        }
        lut
    }

    #[test]
    fn test_basic_identity() {
        let lut = create_identity_lut();
        let src = vec![0u8, 50, 100, 150, 200, 255];
        let mut dst = vec![0u8; src.len()];
        
        apply_lut_basic(&src, &mut dst, &lut);
        assert_eq!(src, dst);
    }

    #[test]
    fn test_basic_invert() {
        let lut = create_invert_lut();
        let src = vec![0u8, 127, 255];
        let mut dst = vec![0u8; src.len()];
        
        apply_lut_basic(&src, &mut dst, &lut);
        assert_eq!(dst, vec![255, 128, 0]);
    }

    #[test]
    fn test_basic_threshold() {
        let lut = create_threshold_lut(128);
        let src = vec![0u8, 50, 100, 127, 128, 200, 255];
        let mut dst = vec![0u8; src.len()];
        
        apply_lut_basic(&src, &mut dst, &lut);
        assert_eq!(dst, vec![0, 0, 0, 0, 255, 255, 255]);
    }

    #[test]
    fn test_basic_various_sizes() {
        let lut = create_identity_lut();
        
        // Test various buffer sizes to ensure chunking works
        for size in [0, 1, 2, 3, 4, 5, 7, 15, 16, 31, 32, 63, 64, 100, 1000] {
            let src: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
            let mut dst = vec![0u8; src.len()];
            
            apply_lut_basic(&src, &mut dst, &lut);
            assert_eq!(src, dst, "Failed for size {}", size);
        }
    }

    #[test]
    fn test_auto_identity() {
        let lut = create_identity_lut();
        let src = vec![0u8, 50, 100, 150, 200, 255];
        let mut dst = vec![0u8; src.len()];
        
        apply_lut_auto(&src, &mut dst, &lut);
        assert_eq!(src, dst);
    }

    #[test]
    fn test_auto_invert() {
        let lut = create_invert_lut();
        let src = vec![0u8, 127, 255];
        let mut dst = vec![0u8; src.len()];
        
        apply_lut_auto(&src, &mut dst, &lut);
        assert_eq!(dst, vec![255, 128, 0]);
    }

    #[test]
    fn test_auto_large_buffer() {
        let lut = create_identity_lut();
        let src: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let mut dst = vec![0u8; src.len()];
        
        apply_lut_auto(&src, &mut dst, &lut);
        assert_eq!(src, dst);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_x86_avx2_identity() {
        let lut = create_identity_lut();
        let src = vec![0u8, 50, 100, 150, 200, 255];
        let mut dst = vec![0u8; src.len()];
        
        apply_lut_x86_avx2(&src, &mut dst, &lut);
        assert_eq!(src, dst);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_x86_avx2_various_sizes() {
        let lut = create_identity_lut();
        
        for size in [0, 1, 16, 31, 32, 33, 63, 64, 65, 100, 1000] {
            let src: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
            let mut dst = vec![0u8; src.len()];
            
            apply_lut_x86_avx2(&src, &mut dst, &lut);
            assert_eq!(src, dst, "Failed for size {}", size);
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_aarch64_neon_identity() {
        let lut = create_identity_lut();
        let src = vec![0u8, 50, 100, 150, 200, 255];
        let mut dst = vec![0u8; src.len()];
        
        apply_lut_aarch64_neon(&src, &mut dst, &lut);
        assert_eq!(src, dst);
    }

    #[test]
    fn test_all_implementations_consistent() {
        let lut = create_invert_lut();
        let src: Vec<u8> = (0..256).map(|i| i as u8).collect();
        
        let mut dst_basic = vec![0u8; src.len()];
        apply_lut_basic(&src, &mut dst_basic, &lut);
        
        let mut dst_auto = vec![0u8; src.len()];
        apply_lut_auto(&src, &mut dst_auto, &lut);
        
        assert_eq!(dst_basic, dst_auto, "Basic and auto implementations differ");
        
        #[cfg(target_arch = "x86_64")]
        {
            let mut dst_x86 = vec![0u8; src.len()];
            apply_lut_x86_avx2(&src, &mut dst_x86, &lut);
            assert_eq!(dst_basic, dst_x86, "Basic and x86 implementations differ");
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            let mut dst_neon = vec![0u8; src.len()];
            apply_lut_aarch64_neon(&src, &mut dst_neon, &lut);
            assert_eq!(dst_basic, dst_neon, "Basic and NEON implementations differ");
        }
    }

    #[test]
    #[should_panic(expected = "src and dst must have same length")]
    fn test_length_mismatch_panics() {
        let lut = create_identity_lut();
        let src = vec![0u8; 10];
        let mut dst = vec![0u8; 5];
        
        apply_lut_auto(&src, &mut dst, &lut);
    }
}
