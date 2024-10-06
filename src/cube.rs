use std::str::FromStr;

pub struct InputRange(std::ops::RangeInclusive<f32>);

pub struct Cube {
    pub(crate) dim: u8,
    pub(crate) size: u16,
    pub(crate) input_range: Option<InputRange>,
    pub(crate) rgbs: Box<[f32]>,
}

impl FromStr for InputRange {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((min, max)) = s.split_once(' ') else {
            return Err("invalid input range".into());
        };
        let min: f32 = min.parse()?;
        let max: f32 = max.parse()?;
        Ok(Self(min..=max))
    }
}

impl Cube {
    pub fn one_d(size: u16) -> Self {
        Self {
            dim: 1,
            size,
            input_range: None,
            rgbs: vec![0.0; size as usize * 3].into(),
        }
    }

    pub fn three_d(size: u16) -> Self {
        Self {
            dim: 3,
            size,
            input_range: None,
            rgbs: vec![0.0; (size as usize).pow(3) * 3].into(),
        }
    }

    pub fn dim(&self) -> u8 {
        self.dim
    }

    pub fn size(&self) -> u16 {
        self.size
    }

    pub fn input_range(&self) -> Option<&InputRange> {
        self.input_range.as_ref()
    }

    pub fn set_input_range(&mut self, val: Option<InputRange>) {
        self.input_range = val;
    }

    pub fn r_len(&self) -> usize {
        self.rgbs.len()
    }

    pub fn raw(&self) -> &[f32] {
        &self.rgbs
    }
}
