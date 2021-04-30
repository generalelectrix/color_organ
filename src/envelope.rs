use derive_more::Display;
use log::error;
use number::UnipolarFloat;
use std::time::{Duration, Instant};

/// A function defining the shape of an envelope transition edge.
/// EdgeShapes should always map 0 to 0 and 1 to 1, but may provide any other
/// profile.  EdgeShapes should define a rising edge; the domain will be reversed
/// to create falling edges.
type EdgeShape = fn(UnipolarFloat) -> UnipolarFloat;

#[inline(always)]
/// A linear edge function.
pub fn linear_edge(alpha: UnipolarFloat) -> UnipolarFloat {
    alpha
}

/// The parameters of an ADSR envelope.
/// TODO: do we want to store these parameters as durations, or as fractions of
/// a time scale?
pub struct EnvelopeParameters {
    pub attack: Duration,
    pub attack_level: UnipolarFloat,
    pub attack_shape: EdgeShape,
    pub decay: Duration,
    pub decay_shape: EdgeShape,
    pub sustain_level: UnipolarFloat,
    pub release: Duration,
    pub release_shape: EdgeShape,
}

/// An evolving ADSR envelope.
/// The current envelope value is computed during update and stored.
pub struct Envelope {
    parameters: EnvelopeParameters,
    elapsed: Duration,
    released: bool,
    /// The current value of the envelope. Updated during state update.
    /// If None, the envelope has closed.
    value: Option<UnipolarFloat>,
    release_elapsed: Duration,
}

impl Envelope {
    pub fn new(parameters: EnvelopeParameters) -> Self {
        Self {
            // Initialize value at the attack level.
            value: Some(parameters.attack_level),
            parameters,
            elapsed: Duration::from_secs(0),
            released: false,
            release_elapsed: Duration::from_secs(0),
        }
    }

    /// Return true if this envelope is released.
    pub fn released(&self) -> bool {
        self.released
    }

    /// Set this envelope as released.
    pub fn release(&mut self) {
        self.released = true;
    }

    /// Get the current value of this envelope.
    /// Return None if the envelope has closed.
    pub fn value(&self) -> Option<UnipolarFloat> {
        self.value
    }

    /// Update the state of this envelope.
    pub fn update_state(&mut self, delta_t: Duration) {
        // Short-circuit state update for closed envelopes.
        if self.value.is_none() {
            return;
        }
        self.elapsed += delta_t;
        if self.released {
            self.release_elapsed += delta_t;
        }
        self.value = self.current_value();
    }

    /// Return the fraction of the attack that has already been completed.
    /// Return 1 if the attack is complete.
    pub fn attack_fraction(&self) -> UnipolarFloat {
        if self.parameters.attack == Duration::from_secs(0) {
            return UnipolarFloat::ONE;
        }
        UnipolarFloat::new(self.elapsed.as_secs_f64() / self.parameters.attack.as_secs_f64())
    }

    /// Compute the current value of this envelope.
    /// Return None if the envelope has closed.
    fn current_value(&self) -> Option<UnipolarFloat> {
        // attack portion
        if self.elapsed <= self.parameters.attack {
            let alpha = self.attack_fraction();
            return Some(rising_edge(
                self.parameters.attack_shape,
                alpha,
                self.parameters.attack_level,
            ));
        }
        // decay portion
        if self.elapsed <= self.parameters.attack + self.parameters.decay {
            // if decay is 0, we take the attack branch of this function so we
            // do not need to treat decay of 0 explicitly here.
            let decay_elapsed = self.elapsed - self.parameters.attack;
            let alpha = UnipolarFloat::new(
                decay_elapsed.as_secs_f64() / self.parameters.decay.as_secs_f64(),
            );
            return Some(falling_edge(
                self.parameters.decay_shape,
                alpha,
                self.parameters.sustain_level,
            ));
        }
        // attack and decay are complete, either sustain or release

        // if sustain level is 0, the envelope has closed.
        if self.parameters.sustain_level == UnipolarFloat::ZERO {
            return None;
        }

        // if not released, holding the sustain level
        if !self.released {
            return Some(self.parameters.sustain_level);
        }

        // releasing
        if self.release_elapsed >= self.parameters.release {
            // Release complete, envelope is closed.
            return None;
        }
        let alpha = UnipolarFloat::new(
            self.release_elapsed.as_secs_f64() / self.parameters.release.as_secs_f64(),
        );
        Some(
            self.parameters.sustain_level
                * falling_edge(self.parameters.release_shape, alpha, UnipolarFloat::ZERO),
        )
    }
}

/// Return the value for a rising edge.
fn rising_edge(shape: EdgeShape, alpha: UnipolarFloat, offset: UnipolarFloat) -> UnipolarFloat {
    offset + shape(alpha) * (UnipolarFloat::ONE - offset)
}

/// Return the value for a falling edge.
fn falling_edge(shape: EdgeShape, alpha: UnipolarFloat, offset: UnipolarFloat) -> UnipolarFloat {
    // Create a falling edge by inverting alpha, essentially running the edge backwards.
    rising_edge(shape, UnipolarFloat::ONE - alpha, offset)
}
