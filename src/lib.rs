type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

mod cube;
pub use cube::Cube;

mod lut;
pub use lut::Lut;

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
    fn black_magic() {
        let mut r = std::io::BufReader::new(File::open("/Library/Application Support/Blackmagic Design/DaVinci Resolve/LUT/VFX IO/Linear to Cineon Log.cube").unwrap());
        let lut = Lut::parse(&mut r).expect("lut");
        assert_eq!(lut.cube().dim(), 1);
        assert_eq!(lut.cube().size(), 4096);

        let mut r = std::io::BufReader::new(File::open("/Library/Application Support/Blackmagic Design/DaVinci Resolve/LUT/ACES/LMT ACES v0.1.1.cube").unwrap());
        let lut = Lut::parse(&mut r).expect("lut");
        assert_eq!(lut.cube().dim(), 3);
        assert_eq!(lut.cube().size(), 65);
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
