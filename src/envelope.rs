use derive_more::Display;
use log::error;
use number::UnipolarFloat;
use std::time::{Duration, Instant};

/// The parameters of an ADSR envelope.
/// TODO: do we want to store these parameters as durations, or as fractions of
/// a time scale?
pub struct EnvelopeParameters {
    pub attack: Duration,
    pub attack_level: UnipolarFloat,
    pub decay: Duration,
    pub sustain_level: UnipolarFloat,
    pub release: Duration,
}

/// An evolving ADSR envelope.
/// The current envelope value is computed during update and stored.
pub struct Envelope {
    parameters: EnvelopeParameters,
    elapsed: Duration,
    released: bool,
    value: UnipolarFloat,
}

impl Envelope {
    pub fn new(parameters: EnvelopeParameters) -> Self {
        Self {
            parameters,
            elapsed: Duration::ZERO,
            released: false,
            // Initialize value at the attack level.
            value: parameters.attack_level,
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

    /// Update the state of this envelope.
    pub fn update_state(&mut self, delta_t: Duration) {
        self.elapsed += delta_t;
        unimplemented!("finish envelope state update");
    }
}
