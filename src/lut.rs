use std::io::BufRead;

use crate::{Cube, Error, InputRange};

pub struct Lut {
    title: Option<String>,
    comments: String,
    in_video_range: bool,
    out_video_range: bool,
    cube: Cube,
    shaper: Option<Cube>,
    domain_min: Option<[f32; 3]>,
    domain_max: Option<[f32; 3]>,
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
    pub fn push(&mut self, rgb: &[f32; 3]) -> Result<Option<Box<Cube>>, Error> {
        let Some(c) = self.cube.as_mut() else {
            return Err(Error::CubeIsFull);
        };
        let len = self.len;
        let next = len + 3;
        c.rgbs[len..next].copy_from_slice(rgb.as_slice());
        self.len = next;
        if next == self.capacity {
            Ok(Some(Box::new(self.cube.take().unwrap())))
        } else {
            Ok(None)
        }
    }
}

impl Lut {
    #[inline]
    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    #[inline]
    pub fn comments(&self) -> &String {
        &self.comments
    }

    #[inline]
    pub fn in_video_range(&self) -> bool {
        self.in_video_range
    }

    #[inline]
    pub fn out_video_range(&self) -> bool {
        self.out_video_range
    }

    #[inline]
    pub fn cube(&self) -> &Cube {
        &self.cube
    }

    #[inline]
    pub fn shaper(&self) -> Option<&Cube> {
        self.shaper.as_ref()
    }

    #[inline]
    pub fn domain_min(&self) -> Option<&[f32; 3]> {
        self.domain_min.as_ref()
    }

    #[inline]
    pub fn domain_max(&self) -> Option<&[f32; 3]> {
        self.domain_max.as_ref()
    }
}

impl Lut {
    pub fn parse(reader: &mut impl BufRead) -> Result<Self, Error> {
        let mut rgb = [0.0f32; 3];
        let mut title = None;
        let mut in_video_range = false;
        let mut out_video_range = false;
        let mut comments = String::new();
        let mut possible_shaper = false;
        let mut cube: Option<Cube> = None;
        let mut shaper: Option<Cube> = None;
        let mut collector: Option<Collector> = None;

        let mut domain_min: Option<[f32; 3]> = None;
        let mut domain_max: Option<[f32; 3]> = None;

        let mut line = String::with_capacity(100);
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                return Err(Error::Eof);
            }
            let len = line.len();
            if len < 2 {
                continue;
            }

            // remove '\n' at the end
            let s = &line[..len - 1];

            // # Comment
            if s.as_bytes()[0] == b'#' {
                if !comments.is_empty() {
                    comments.push_str("\n");
                }
                comments.push_str(s);
                continue;
            }

            let Some((a, b)) = s.split_once(' ') else {
                continue;
            };

            if let Some(coll) = collector.as_mut() {
                let Some((g, b)) = b.split_once(' ') else {
                    return Err(Error::InvalidRgb(line));
                };

                rgb[0] = a.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;
                rgb[1] = g.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;
                rgb[2] = b.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;

                if let Some(completed_cube) = coll.push(&rgb)? {
                    if let Some(cube) = cube.take() {
                        shaper = Some(*completed_cube);
                        collector = Some(Collector::with_cube(cube));
                    } else {
                        return Ok(Self {
                            title,
                            comments,
                            in_video_range,
                            out_video_range,
                            cube: *completed_cube,
                            shaper,
                            domain_min,
                            domain_max,
                        });
                    };
                }
                continue;
            }

            match a {
                "TITLE" => title = Some(b.to_owned()),
                "LUT_IN_VIDEO_RANGE" => in_video_range = true,
                "LUT_OUT_VIDEO_RANGE" => out_video_range = true,
                "LUT_1D_SIZE" if possible_shaper => {
                    shaper = cube.take();
                    cube = Some(Cube::one_d(b.parse()?));
                }
                "LUT_1D_SIZE" => {
                    possible_shaper = true;
                    cube = Some(Cube::one_d(b.parse()?));
                }
                "LUT_3D_SIZE" if possible_shaper => {
                    shaper = cube.take();
                    cube = Some(Cube::three_d(b.parse()?));
                }
                "LUT_3D_SIZE" => {
                    cube = Some(Cube::three_d(b.parse()?));
                }
                "LUT_1D_INPUT_RANGE" | "LUT_3D_INPUT_RANGE" => {
                    let Some(c) = cube.as_mut() else {
                        return Err(Error::UnexpectedInputRange(line.clone()));
                    };
                    c.set_input_range(Some(InputRange::try_from(b)?));
                }
                "DOMAIN_MIN" | "DOMAIN_MAX" => {
                    let Some((r, gb)) = b.split_once(' ') else {
                        return Err(Error::InvalidRgb(line));
                    };
                    let Some((g, b)) = gb.split_once(' ') else {
                        return Err(Error::InvalidRgb(line));
                    };
                    rgb[0] = r.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;
                    rgb[1] = g.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;
                    rgb[2] = b.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;
                    if a == "DOMAIN_MIN" {
                        domain_min = Some(rgb.clone());
                    } else {
                        domain_max = Some(rgb.clone());
                    }
                }
                r => {
                    let Some((g, b)) = b.split_once(' ') else {
                        return Err(Error::InvalidRgb(line));
                    };

                    rgb[0] = r.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;
                    rgb[1] = g.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;
                    rgb[2] = b.parse().map_err(|_| Error::InvalidRgb(line.clone()))?;

                    let mut coll = if let Some(shaper) = shaper.take() {
                        Collector::with_cube(shaper)
                    } else if let Some(cube) = cube.take() {
                        Collector::with_cube(cube)
                    } else {
                        return Err(Error::UnexpectedData);
                    };

                    coll.push(&rgb)?;
                    collector = Some(coll);
                }
            }
        }
    }
}
