use number::{Phase, UnipolarFloat};

/// A color, expressed as one of several options for color spaces.
/// TODO: HSLuv support.
pub enum Color {
    Hsv(HsvColor),
}

#[derive(Clone)]
/// A color in the HSV space.
pub struct HsvColor {
    pub hue: Phase,
    pub saturation: UnipolarFloat,
    pub value: UnipolarFloat,
}

impl HsvColor {
    pub fn enveloped(&self, envelope: UnipolarFloat) -> Self {
        let mut copy = self.clone();
        copy.value *= envelope;
        copy
    }
}
