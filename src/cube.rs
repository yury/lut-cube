use crate::Error;

pub struct InputRange(std::ops::RangeInclusive<f32>);

pub struct Cube {
    pub(crate) dim: u8,
    pub(crate) size: u16,
    pub(crate) input_range: Option<InputRange>,
    pub(crate) rgbs: Box<[f32]>,
}

impl TryFrom<&str> for InputRange {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let Some((min, max)) = value.split_once(' ') else {
            return Err(Error::InvalidInputRange(value.to_owned()));
        };
        let min: f32 = min
            .parse()
            .map_err(|_| Error::InvalidInputRange(value.to_owned()))?;
        let max: f32 = max
            .parse()
            .map_err(|_| Error::InvalidInputRange(value.to_owned()))?;
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

    #[inline]
    pub fn dim(&self) -> u8 {
        self.dim
    }

    #[inline]
    pub fn size(&self) -> u16 {
        self.size
    }

    #[inline]
    pub fn input_range(&self) -> Option<&InputRange> {
        self.input_range.as_ref()
    }

    #[inline]
    pub fn set_input_range(&mut self, val: Option<InputRange>) {
        self.input_range = val;
    }

    #[inline]
    pub fn r_len(&self) -> usize {
        self.rgbs.len()
    }
}
