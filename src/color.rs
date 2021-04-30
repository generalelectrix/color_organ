use number::{Phase, UnipolarFloat};

/// A trait for a color in a particular color space.
pub trait Color: Sized + Clone {
    const BLACK: Self;

    fn with_envelope(&self, envelope: UnipolarFloat) -> Self;

    /// Return the enveloped color, or None if the envelope has closed.
    fn enveloped(&self, envelope: Option<UnipolarFloat>) -> Option<Self> {
        envelope.map(|e| self.with_envelope(e))
    }
}

#[derive(Clone)]
/// A color in the HSV space.
pub struct HsvColor {
    pub hue: Phase,
    pub saturation: UnipolarFloat,
    pub value: UnipolarFloat,
}

impl Color for HsvColor {
    const BLACK: Self = Self {
        hue: Phase::ZERO,
        saturation: UnipolarFloat::ONE,
        value: UnipolarFloat::ZERO,
    };

    fn with_envelope(&self, envelope: UnipolarFloat) -> Self {
        let mut copy = self.clone();
        copy.value *= envelope;
        copy
    }
}
