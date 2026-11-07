use std::io::{self, BufRead};

use crate::{Cube, Result, cube};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Meta {
    title: Option<String>,
    comments: String,
    in_video_range: bool,
    out_video_range: bool,
    domain_min: Option<[f32; 3]>,
    domain_max: Option<[f32; 3]>,
    size: Option<u16>,
}

/// Maximum declared sizes accepted by full parsing, checked before allocating each table.
/// A zero limit disables that table dimension, including a 1D shaper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseLimits {
    /// Maximum entry count for a 1D table or shaper.
    pub max_1d_size: u16,
    /// Maximum edge length of a 3D table.
    pub max_3d_size: u16,
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_1d_size: u16::MAX,
            max_3d_size: u16::MAX,
        }
    }
}

impl ParseLimits {
    fn size(value: &str, limit: u16, directive: &str) -> Result<u16> {
        let size = value.parse()?;
        if size < 2 || size > limit {
            return Err(format!("{directive} {size} outside allowed range 2..={limit}").into());
        }
        Ok(size)
    }
}

pub struct Lut {
    meta: Meta,
    cube: Cube,
    shaper: Option<Cube>,
}

struct Collector {
    cube: Option<Cube>,
    len: usize,
    capacity: usize,
}

impl Collector {
    pub fn with_cube(cube: Cube) -> Self {
        let capacity = cube.r_len();
        Self {
            cube: Some(cube),
            len: 0,
            capacity,
        }
    }

    #[inline]
    fn push<const N: usize>(&mut self, rgb: &[f32; N]) -> Result<Option<Box<Cube>>> {
        let Some(cube) = self.cube.as_mut() else {
            return Err("Cube is full".into());
        };
        let len = self.len;
        let next = len + N;
        cube.rgbs[len..next].copy_from_slice(rgb);
        self.len = next;
        if next == self.capacity {
            Ok(Some(Box::new(self.cube.take().unwrap())))
        } else {
            Ok(None)
        }
    }
}

impl Meta {
    /// The size declared by `LUT_3D_SIZE`, or `None` for a 1D-only LUT.
    pub fn size(&self) -> Option<u16> {
        self.size
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn comments(&self) -> &str {
        &self.comments
    }

    pub fn in_video_range(&self) -> bool {
        self.in_video_range
    }

    pub fn out_video_range(&self) -> bool {
        self.out_video_range
    }

    pub fn domain_min(&self) -> Option<&[f32; 3]> {
        self.domain_min.as_ref()
    }

    pub fn domain_max(&self) -> Option<&[f32; 3]> {
        self.domain_max.as_ref()
    }

    fn parse_line(&mut self, s: &str, delimiter: &[char]) -> Result<bool> {
        if s.is_empty() {
            return Ok(true);
        }
        if s.starts_with('#') {
            if !self.comments.is_empty() {
                self.comments.push('\n');
            }
            self.comments.push_str(s);
            return Ok(true);
        }
        let Some((key, value)) = s.split_once(delimiter) else {
            return Ok(false);
        };
        match key {
            "TITLE" => self.title = Some(value.to_owned()),
            "LUT_IN_VIDEO_RANGE" => self.in_video_range = true,
            "LUT_OUT_VIDEO_RANGE" => self.out_video_range = true,
            "DOMAIN_MIN" | "DOMAIN_MAX" => {
                let Some((r, gb)) = value.split_once(delimiter) else {
                    return Err(format!("invalid rgb in line: {s}").into());
                };
                let Some((g, b)) = gb.split_once(delimiter) else {
                    return Err(format!("invalid rgb in line: {s}").into());
                };
                let rgb = [r.parse()?, g.parse()?, b.parse()?];
                if key == "DOMAIN_MIN" {
                    self.domain_min = Some(rgb);
                } else {
                    self.domain_max = Some(rgb);
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl Lut {
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    pub fn title(&self) -> Option<&str> {
        self.meta.title()
    }

    pub fn comments(&self) -> &str {
        self.meta.comments()
    }

    pub fn in_video_range(&self) -> bool {
        self.meta.in_video_range()
    }

    pub fn out_video_range(&self) -> bool {
        self.meta.out_video_range()
    }

    pub fn cube(&self) -> &Cube {
        &self.cube
    }

    pub fn shaper(&self) -> Option<&Cube> {
        self.shaper.as_ref()
    }

    pub fn domain_min(&self) -> Option<&[f32; 3]> {
        self.meta.domain_min()
    }

    pub fn domain_max(&self) -> Option<&[f32; 3]> {
        self.meta.domain_max()
    }

    /// Parses header metadata without allocating cubes or parsing sample values.
    /// Stops after reading the first sample line, or at the end of the reader.
    /// Comments following sample data are not included.
    pub fn parse_meta(reader: &mut impl BufRead) -> Result<Meta> {
        let mut meta = Meta::default();
        let mut line = String::with_capacity(100);
        let delimiter = &[' ', '\t'];
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                return Ok(meta);
            }
            let s = line.trim();
            if meta.parse_line(s, delimiter)? {
                continue;
            }
            let key = s.split_once(delimiter).map_or(s, |(key, _)| key);
            match key {
                "LUT_3D_SIZE" => {
                    let (_, value) = s.split_once(delimiter).ok_or("missing LUT_3D_SIZE value")?;
                    meta.size = Some(value.parse()?);
                }
                "LUT_1D_SIZE" | "LUT_1D_INPUT_RANGE" | "LUT_3D_INPUT_RANGE" => {}
                _ => return Ok(meta),
            }
        }
    }

    /// Parses a LUT with the format's representable size limits.
    /// Use [`Self::parse_with_limits`] to cap table allocations for untrusted input.
    pub fn parse(reader: &mut impl BufRead) -> Result<Self> {
        Self::parse_with_limits(reader, ParseLimits::default())
    }

    /// Parses samples and metadata in one pass, checking every size declaration before
    /// allocating a table. Limits apply to both standalone tables and optional shapers.
    /// Table sizes must be at least 2; setting a limit to 0 disallows that dimension.
    /// Metadata-only parsing remains unrestricted because it does not allocate tables.
    pub fn parse_with_limits(reader: &mut impl BufRead, limits: ParseLimits) -> Result<Self> {
        let mut rgb = [0.0f32; 3];
        let mut meta = Meta::default();
        let mut possible_shaper = false;
        let mut cube: Option<Cube> = None;
        let mut shaper: Option<Cube> = None;
        let mut collector: Option<Collector> = None;
        let delimiter = &[' ', '\t'];

        let mut line = String::with_capacity(100);
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                Err(io::Error::from(io::ErrorKind::UnexpectedEof))?;
            }
            // remove '\n' and spaces at the end and start
            let s = line.trim();
            if (s.is_empty() || s.starts_with('#') || collector.is_none())
                && meta.parse_line(s, delimiter)?
            {
                continue;
            }

            let Some((a, b)) = s.split_once(delimiter) else {
                continue;
            };

            if let Some(coll) = collector.as_mut() {
                let Some((g, b)) = b.split_once(delimiter) else {
                    return Err(format!("invalid rgb in line: {line}").into());
                };

                rgb[0] = a.parse()?;
                rgb[1] = g.parse()?;
                rgb[2] = b.parse()?;

                if let Some(completed_cube) = coll.push(&rgb)? {
                    if let Some(cube) = cube.take() {
                        shaper = Some(*completed_cube);
                        collector = Some(Collector::with_cube(cube));
                    } else {
                        return Ok(Self {
                            meta,
                            cube: *completed_cube,
                            shaper,
                        });
                    };
                }
                continue;
            }

            match a {
                "LUT_1D_SIZE" if possible_shaper => {
                    let size = ParseLimits::size(b, limits.max_1d_size, a)?;
                    shaper = cube.take();
                    cube = Some(Cube::one_d(size));
                }
                "LUT_1D_SIZE" => {
                    let size = ParseLimits::size(b, limits.max_1d_size, a)?;
                    possible_shaper = true;
                    cube = Some(Cube::one_d(size));
                }
                "LUT_3D_SIZE" => {
                    let size = ParseLimits::size(b, limits.max_3d_size, a)?;
                    meta.size = Some(size);
                    if possible_shaper {
                        shaper = cube.take();
                    }
                    cube = Some(Cube::three_d(size));
                }
                "LUT_1D_INPUT_RANGE" | "LUT_3D_INPUT_RANGE" => {
                    let Some(c) = cube.as_mut() else {
                        return Err(format!("Unexpected input range: {line}").into());
                    };
                    c.set_input_range(Some(cube::parse_input_range(b, delimiter)?));
                }
                r => {
                    let Some((g, b)) = b.split_once(delimiter) else {
                        return Err(format!("invalid rgb in line: {line}").into());
                    };

                    rgb[0] = r.parse()?;
                    rgb[1] = g.parse()?;
                    rgb[2] = b.parse()?;

                    let mut coll = if let Some(shaper) = shaper.take() {
                        Collector::with_cube(shaper)
                    } else if let Some(cube) = cube.take() {
                        Collector::with_cube(cube)
                    } else {
                        return Err(io::Error::from(io::ErrorKind::InvalidData).into());
                    };

                    coll.push(&rgb)?;
                    collector = Some(coll);
                }
            }
        }
    }
}
