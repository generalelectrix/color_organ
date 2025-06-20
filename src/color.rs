use std::{cell::Cell, f64::consts::PI};

use number::{Phase, UnipolarFloat};

const TWOPI: f64 = 2.0 * PI;

/// A trait for a color in a particular color space.
///
/// The default value for the type should correspond to black.
pub trait Color: Sized + Clone + Default {
    fn with_envelope(&self, envelope: UnipolarFloat) -> Self;

    /// Return the color with the given envelope applied.
    fn enveloped(&self, envelope: Option<UnipolarFloat>) -> Self {
        envelope.map(|e| self.with_envelope(e)).unwrap_or_default()
    }

    /// Perform a weighted interpolation with other color.
    /// The details of the interpolation are left up to the color space.
    /// The scaling factor should be used to dim the other color, but should not
    /// impact the brightness contribution of self.
    fn weighted_interpolation(&self, other: &Self, scale_factor: UnipolarFloat) -> Self;
}

// #[derive(Clone)]
// /// A color in the HSV space.
// pub struct HsvColor {
//     pub hue: Phase,
//     pub saturation: UnipolarFloat,
//     pub value: UnipolarFloat,
// }

// impl Color for HsvColor {
//     const BLACK: Self = Self {
//         hue: Phase::ZERO,
//         saturation: UnipolarFloat::ONE,
//         value: UnipolarFloat::ZERO,
//     };

//     fn with_envelope(&self, envelope: UnipolarFloat) -> Self {
//         let mut copy = self.clone();
//         copy.value *= envelope;
//         copy
//     }
// }

#[derive(Clone)]
/// A color in the HSLuv space.
pub struct HsluvColor {
    pub hue: Phase,
    pub saturation: UnipolarFloat,
    pub lightness: UnipolarFloat,
    /// Lazy memoization of the hue/saturation component in rectangular coordinates.
    rect: Cell<Option<(f64, f64)>>,
}

impl Default for HsluvColor {
    fn default() -> Self {
        Self::new(Phase::ZERO, UnipolarFloat::ONE, UnipolarFloat::ZERO)
    }
}

impl HsluvColor {
    pub fn new(hue: Phase, saturation: UnipolarFloat, lightness: UnipolarFloat) -> Self {
        Self {
            hue,
            saturation,
            lightness,
            rect: Cell::new(None),
        }
    }

    /// Get memoized rectangular coordinates.
    fn rect(&self) -> (f64, f64) {
        self.rect.get().unwrap_or_else(|| {
            let x = self.saturation.val() * (TWOPI * self.hue.val()).cos();
            let y = self.saturation.val() * (TWOPI * self.hue.val()).sin();
            self.rect.set(Some((x, y)));
            (x, y)
        })
    }
}

impl Color for HsluvColor {
    fn with_envelope(&self, envelope: UnipolarFloat) -> Self {
        let mut copy = self.clone();
        copy.lightness *= envelope;
        copy
    }

    fn weighted_interpolation(&self, other: &Self, scale_factor: UnipolarFloat) -> Self {
        let lightness = other.lightness * scale_factor + self.lightness;

        // interpolate shade in rectangular coordinates
        let alpha = self.lightness.val() / lightness.val();
        let ((x_self, y_self), (x_other, y_other)) = (self.rect(), other.rect());
        let x = lerp(x_other, x_self, alpha);
        let y = lerp(y_other, y_self, alpha);
        let (saturation, hue) = polar(x, y);
        Self {
            hue,
            saturation: UnipolarFloat::new(saturation),
            lightness,
            rect: Cell::new(Some((x, y))),
        }
    }
}

/// Convert rectangular coordinates into polar coordinates.
fn polar(x: f64, y: f64) -> (f64, Phase) {
    let r = (x.powi(2) + y.powi(2)).sqrt();
    let theta = Phase::new((y / x).atan() / TWOPI);
    (r, theta)
}

/// Linear interpolation.
fn lerp(v_old: f64, v_new: f64, alpha: f64) -> f64 {
    alpha * v_new + (1. - alpha) * v_old
}
