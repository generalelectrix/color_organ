use std::{cell::Cell, f64::consts::PI};

use number::{Phase, UnipolarFloat};

const TWOPI: f64 = 2.0 * PI;

/// A trait for a color in a particular color space.
///
/// The default value for the type should correspond to black.
pub trait Color: Clone + Default {
    fn with_envelope(&self, envelope: UnipolarFloat) -> Self;

    /// Return the color with the given envelope applied.
    fn enveloped(&self, envelope: Option<UnipolarFloat>) -> Self {
        envelope.map(|e| self.with_envelope(e)).unwrap_or_default()
    }

    /// Perform a weighted interpolation towards another color.
    /// The details of the interpolation are left up to the color space.
    /// alpha is the linear interpolation parameter; alpha = 0 implies we should
    /// only have self; alpha = 1 implies we should only have other.
    fn weighted_interpolation(&self, target: &Self, alpha: UnipolarFloat) -> Self;
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

#[derive(Clone, PartialEq, Debug)]
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

    fn weighted_interpolation(&self, target: &Self, alpha: UnipolarFloat) -> Self {
        // Fade out the intensity of the previous color while allowing the intensity
        // of the incoming color to come through - select whichever is largest.
        // Since the incoming color will usually be a upwards-ramping envelope,
        // this should give us a reasonable transition.
        let outgoing_lightness = self.lightness * alpha.invert();
        let lightness = if outgoing_lightness > target.lightness {
            outgoing_lightness
        } else {
            target.lightness
        };

        // interpolate shade in rectangular coordinates
        let ((x_current, y_current), (x_target, y_target)) = (self.rect(), target.rect());
        let x = lerp(x_current, x_target, alpha.val());
        let y = lerp(y_current, y_target, alpha.val());
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
    let mut theta = (y / x).atan() / TWOPI;
    if x < 0. {
        theta += 0.5;
    }
    (r, Phase::new(theta))
}

/// Linear interpolation.
fn lerp(v_old: f64, v_new: f64, alpha: f64) -> f64 {
    alpha * v_new + (1. - alpha) * v_old
}

#[cfg(test)]
mod test {
    use number::{Phase, UnipolarFloat};

    use crate::{Color, HsluvColor};

    #[test]
    fn test_interpolation() {
        // Interpolating a color with itself should always produce the same result.
        let c = HsluvColor::new(Phase::new(0.3), UnipolarFloat::ONE, UnipolarFloat::new(0.3));

        assert_eq!(c.weighted_interpolation(&c.clone(), UnipolarFloat::ZERO), c);
        assert_eq!(c.weighted_interpolation(&c.clone(), UnipolarFloat::ONE), c);
        assert_eq!(
            c.weighted_interpolation(&c.clone(), UnipolarFloat::new(0.5)),
            c
        );
    }
}
