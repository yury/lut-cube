type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod cube;
pub use cube::Cube;

mod lut;
pub use lut::{Lut, Meta};

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::{Cube, Lut};

    #[test]
    fn basics() {
        let c1 = Cube::one_d(3);
        assert_eq!(c1.dim(), 1);
        assert_eq!(c1.r_len(), 3 * 3);

        let c3 = Cube::three_d(75);
        assert_eq!(c3.dim(), 3);
        assert_eq!(c3.r_len(), 75 * 75 * 75 * 3);
    }

    #[test]
    fn metadata_matches_full_parse() {
        let source = b"# first comment\nTITLE \"Test LUT\"\n\n\
            LUT_IN_VIDEO_RANGE 1\nLUT_OUT_VIDEO_RANGE 1\n\
            DOMAIN_MIN -1 0 0.25\nDOMAIN_MAX 1 2 3\n\
            LUT_1D_SIZE 2\nLUT_1D_INPUT_RANGE 0 1\n\
            LUT_3D_SIZE 2\nLUT_3D_INPUT_RANGE -1 2\n# last comment\n\
            0 0 0\n1 1 1\n\
            0 0 0\n1 0 0\n0 1 0\n1 1 0\n\
            0 0 1\n1 0 1\n0 1 1\n1 1 1\n";
        let meta = Lut::parse_meta(&mut &source[..]).unwrap();
        let lut = Lut::parse(&mut &source[..]).unwrap();
        assert_eq!(lut.meta(), &meta);
        assert_eq!(meta.size(), Some(2));
        assert_eq!(meta.title(), Some("\"Test LUT\""));
        assert_eq!(meta.comments(), "# first comment\n# last comment");
        assert!(meta.in_video_range());
        assert!(meta.out_video_range());
        assert_eq!(meta.domain_min(), Some(&[-1.0, 0.0, 0.25]));
        assert_eq!(meta.domain_max(), Some(&[1.0, 2.0, 3.0]));
        assert_eq!(lut.title(), meta.title());
        assert_eq!(lut.comments(), meta.comments());
        assert_eq!(lut.in_video_range(), meta.in_video_range());
        assert_eq!(lut.out_video_range(), meta.out_video_range());
        assert_eq!(lut.domain_min(), meta.domain_min());
        assert_eq!(lut.domain_max(), meta.domain_max());
        assert_eq!(lut.cube().input_range(), Some(&(-1.0..=2.0)));
        let shaper = lut.shaper().unwrap();
        assert_eq!(shaper.raw(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        assert_eq!(shaper.input_range(), Some(&(0.0..=1.0)));
        assert_eq!(lut.cube().raw().len(), 24);
    }

    #[test]
    fn metadata_skips_cube_parsing_and_stops_at_samples() {
        let source = b"LUT_3D_SIZE 65535\nLUT_3D_INPUT_RANGE not-parsed\n\
            TITLE Header\ninvalid sample values\nDOMAIN_MIN invalid\n";
        let mut reader = &source[..];
        let meta = Lut::parse_meta(&mut reader).unwrap();
        assert_eq!(meta.title(), Some("Header"));
        assert_eq!(meta.size(), Some(65535));
        assert_eq!(reader, b"DOMAIN_MIN invalid\n");
    }

    #[test]
    fn metadata_accepts_header_only_and_empty_input() {
        let meta = Lut::parse_meta(&mut &b""[..]).unwrap();
        assert_eq!(meta, crate::Meta::default());
        let meta = Lut::parse_meta(&mut &b"# header\nTITLE Test\nDOMAIN_MAX 1 1 1"[..]).unwrap();
        assert_eq!(meta.title(), Some("Test"));
        assert_eq!(meta.comments(), "# header");
        assert_eq!(meta.domain_max(), Some(&[1.0; 3]));
        assert_eq!(meta.size(), None);
        let meta = Lut::parse_meta(&mut &b"LUT_1D_SIZE 4096\n"[..]).unwrap();
        assert_eq!(meta.size(), None);
    }

    #[test]
    fn metadata_rejects_invalid_3d_sizes() {
        for source in [
            "LUT_3D_SIZE invalid\n",
            "LUT_3D_SIZE 65536\n",
            "LUT_3D_SIZE\n",
        ] {
            assert!(Lut::parse_meta(&mut source.as_bytes()).is_err());
        }
    }

    #[test]
    fn metadata_rejects_invalid_domains() {
        for source in ["DOMAIN_MIN 0 1\n", "DOMAIN_MAX 0 invalid 1\n"] {
            assert!(Lut::parse_meta(&mut source.as_bytes()).is_err());
            assert!(Lut::parse(&mut source.as_bytes()).is_err());
        }
    }

    #[test]
    fn black_magic() {
        let mut r = std::io::BufReader::new(File::open("/Library/Application Support/Blackmagic Design/DaVinci Resolve/LUT/VFX IO/Linear to Cineon Log.cube").unwrap());
        let lut = Lut::parse(&mut r).expect("lut");
        assert_eq!(lut.cube().dim(), 1);
        assert_eq!(lut.cube().size(), 4096);
        assert_eq!(lut.meta().size(), None);

        let mut r = std::io::BufReader::new(File::open("/Library/Application Support/Blackmagic Design/DaVinci Resolve/LUT/ACES/LMT ACES v0.1.1.cube").unwrap());
        let lut = Lut::parse(&mut r).expect("lut");
        assert_eq!(lut.cube().dim(), 3);
        assert_eq!(lut.cube().size(), 65);
        assert_eq!(lut.meta().size(), Some(65));
        let shaper = lut.shaper().unwrap();
        assert_eq!(shaper.dim(), 1);
        assert_eq!(shaper.size(), 4095);
        println!("comments:\n{}", lut.comments());
    }

    // #[test]
    // fn apple() {
    //     let mut r = std::io::BufReader::new(
    //         File::open("/Users/yury/Projects/yoml/multi/Luts/AppleLogToRec709-v1.0.cube").unwrap(),
    //     );
    //     let lut = Lut::parse(&mut r).expect("lut");
    //     assert_eq!(lut.cube().dim(), 3);
    //     assert_eq!(lut.cube().size(), 65);
    //     println!("comments:\n{}", lut.comments());
    // }
}
