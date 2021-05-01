use derive_more::Display;
use log::error;
use number::UnipolarFloat;
use std::time::{Duration, Instant};

/// A function defining the shape of an envelope transition edge.
/// EdgeShapes should always map 0 to 0 and 1 to 1, but may provide any other
/// profile.  EdgeShapes should define a rising edge; the domain will be reversed
/// to create falling edges.
type EdgeShape = fn(UnipolarFloat) -> UnipolarFloat;

/// A linear edge function.
pub fn linear_edge(alpha: UnipolarFloat) -> UnipolarFloat {
    alpha
}

/// The parameters of an ADSR envelope.
/// TODO: do we want to store these parameters as durations, or as fractions of
/// a time scale?
#[derive(Clone)]
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

impl EnvelopeParameters {
    /// Return envelope parameters with linear edges.
    pub fn linear(
        attack: Duration,
        attack_level: UnipolarFloat,
        decay: Duration,
        sustain_level: UnipolarFloat,
        release: Duration,
    ) -> Self {
        Self {
            attack,
            attack_level,
            attack_shape: linear_edge,
            decay,
            decay_shape: linear_edge,
            sustain_level,
            release,
            release_shape: linear_edge,
        }
    }
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
        let mut envelope = Self {
            value: None,
            parameters,
            elapsed: Duration::from_secs(0),
            released: false,
            release_elapsed: Duration::from_secs(0),
        };
        // Initialize value.
        envelope.update_value();
        envelope
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
        self.update_value();
    }
    /// Return true if this envelope has completed the attack.
    pub fn attack_complete(&self) -> bool {
        self.elapsed > self.parameters.attack
    }

    /// Update the current stored value of this envelope.
    /// Set None if the envelope has closed.
    fn update_value(&mut self) {
        self.value = if self.elapsed <= self.parameters.attack {
            // attack portion
            let alpha = if self.parameters.attack == Duration::from_secs(0) {
                UnipolarFloat::ONE
            } else {
                UnipolarFloat::new(
                    self.elapsed.as_secs_f64() / self.parameters.attack.as_secs_f64(),
                )
            };
            Some(rising_edge(
                self.parameters.attack_shape,
                alpha,
                self.parameters.attack_level,
            ))
        }
        // decay portion
        else if self.elapsed <= self.parameters.attack + self.parameters.decay {
            // if decay is 0, we take the attack branch of this function so we
            // do not need to treat decay of 0 explicitly here.
            let decay_elapsed = self.elapsed - self.parameters.attack;
            let alpha = UnipolarFloat::new(
                decay_elapsed.as_secs_f64() / self.parameters.decay.as_secs_f64(),
            );
            Some(falling_edge(
                self.parameters.decay_shape,
                alpha,
                self.parameters.sustain_level,
            ))
        }
        // attack and decay are complete, either sustain or release

        // if sustain level is 0, the envelope has closed.
        else if self.parameters.sustain_level == UnipolarFloat::ZERO {
            None
        }
        // if not released, holding the sustain level
        else if !self.released {
            Some(self.parameters.sustain_level)
        }
        // releasing

        // Release complete, envelope is closed.
        else if self.release_elapsed >= self.parameters.release {
            None
        }
        // Releasing
        else {
            let alpha = UnipolarFloat::new(
                self.release_elapsed.as_secs_f64() / self.parameters.release.as_secs_f64(),
            );
            Some(
                self.parameters.sustain_level
                    * falling_edge(self.parameters.release_shape, alpha, UnipolarFloat::ZERO),
            )
        };
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

#[cfg(test)]
mod test {
    use super::*;

    /// Return basic envelope paramters for testing.
    fn params() -> EnvelopeParameters {
        EnvelopeParameters::linear(
            Duration::from_secs(1),
            UnipolarFloat::new(0.4),
            Duration::from_secs(1),
            UnipolarFloat::new(0.6),
            Duration::from_secs(1),
        )
    }

    #[test]
    /// Basic test of envelope shape.
    fn test_full_shape() {
        let params = params();
        let mut envelope = Envelope::new(params.clone());
        assert_eq!(Some(params.attack_level), envelope.value());

        // Evolve for half of the attack.
        envelope.update_state(Duration::from_millis(500));
        assert_eq!(Some(UnipolarFloat::new(0.7)), envelope.value());

        // Complete attack.
        envelope.update_state(Duration::from_millis(500));
        assert_eq!(Some(UnipolarFloat::ONE), envelope.value());

        // Half of decay.
        envelope.update_state(Duration::from_millis(500));
        assert_eq!(Some(UnipolarFloat::new(0.8)), envelope.value());

        // Complete decay.
        envelope.update_state(Duration::from_millis(500));
        assert_eq!(Some(params.sustain_level), envelope.value());

        // Nigel: The sustain... look at it...
        envelope.update_state(Duration::from_secs(1));
        assert_eq!(Some(params.sustain_level), envelope.value());
        // Marty: I'm not seeing anything.
        // Nigel: You would, though, if it were playing, because it really...
        // it's famous for its sustain... I mean, you could, just, hold it...
        envelope.update_state(Duration::from_secs(1));
        assert_eq!(Some(params.sustain_level), envelope.value());
        // bluuuuuuuuuuuuuuuuuuu... you could go and have a bite an'
        envelope.update_state(Duration::from_secs(1000));
        assert_eq!(Some(params.sustain_level), envelope.value());
        // ...uuuuuuuuuuu... you'd still be seein' that one.

        // Release.
        envelope.release();
        assert_eq!(Some(params.sustain_level), envelope.value());

        envelope.update_state(Duration::from_millis(500));
        assert_eq!(Some(UnipolarFloat::new(0.3)), envelope.value());

        // Complete release and confirm that envelope closes.
        envelope.update_state(Duration::from_millis(500));
        assert_eq!(None, envelope.value());
    }

    #[test]
    /// Test zero-attack envelope.
    fn test_zero_attack() {
        let mut params = params();
        params.attack = Duration::from_secs(0);
        let mut envelope = Envelope::new(params);
        assert_eq!(Some(UnipolarFloat::ONE), envelope.value());

        // Check expected decay.
        envelope.update_state(Duration::from_millis(500));
        assert_eq!(Some(UnipolarFloat::new(0.8)), envelope.value());
    }

    #[test]
    /// Test zero-decay envelope.
    fn test_zero_decay() {
        let mut params = params();
        params.decay = Duration::from_secs(0);
        let mut envelope = Envelope::new(params.clone());
        envelope.update_state(params.attack);
        assert_eq!(Some(UnipolarFloat::ONE), envelope.value());

        // Should immediately fall to sustain level.
        envelope.update_state(Duration::from_nanos(1));
        assert_eq!(Some(params.sustain_level), envelope.value());
    }

    #[test]
    /// Test zero-release envelope.
    fn test_zero_release() {
        let mut params = params();
        params.release = Duration::from_secs(0);
        let mut envelope = Envelope::new(params.clone());
        envelope.update_state(params.attack + params.decay);
        assert_eq!(Some(params.sustain_level), envelope.value());

        // nudge just past the decay into sustain.
        envelope.update_state(Duration::from_nanos(1));
        assert_eq!(Some(params.sustain_level), envelope.value());

        envelope.release();
        // Since release is a step-change in state, even a zero-duration state
        // update should force the envelope to close due to zero release.
        envelope.update_state(Duration::from_secs(0));
        assert_eq!(None, envelope.value());
    }

    #[test]
    /// Test zero-sustain envelope.
    fn test_zero_sustain() {
        let mut params = params();
        params.sustain_level = UnipolarFloat::ZERO;
        let mut envelope = Envelope::new(params.clone());
        envelope.update_state(params.attack);
        assert_eq!(Some(UnipolarFloat::ONE), envelope.value());

        envelope.update_state(Duration::from_millis(500));
        assert_eq!(Some(UnipolarFloat::new(0.5)), envelope.value());

        // Complete decay - envelope should not close until next update since we
        // are still on the trailing edge of decay.
        envelope.update_state(Duration::from_millis(500));
        assert_eq!(Some(UnipolarFloat::ZERO), envelope.value());

        // Any further evolution should close the envelope.
        envelope.update_state(Duration::from_nanos(1));
        assert_eq!(None, envelope.value());
    }

    #[test]
    /// Test weird edge case - the all-zero timing envelope.
    /// This envelope should start at full scale, then jump immediately
    /// to the sustain level, then close immediately when released.
    fn test_all_zero_envelope() {
        let mut params = params();
        params.attack = Duration::from_secs(0);
        params.decay = Duration::from_secs(0);
        params.release = Duration::from_secs(0);
        let mut envelope = Envelope::new(params.clone());
        assert_eq!(Some(UnipolarFloat::ONE), envelope.value());

        envelope.update_state(Duration::from_nanos(1));
        assert_eq!(Some(params.sustain_level), envelope.value());

        envelope.release();
        envelope.update_state(Duration::from_secs(0));
        assert_eq!(None, envelope.value());
    }
}
