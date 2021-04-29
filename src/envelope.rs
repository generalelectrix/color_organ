use derive_more::Display;
use log::error;
use number::UnipolarFloat;
use std::time::{Duration, Instant};

/// The parameters of an ADSR envelope.
/// TODO: do we want to store these parameters as durations, or as fractions of
/// a time scale?
pub struct Envelope {
    pub attack: Duration,
    pub attack_level: UnipolarFloat,
    pub decay: Duration,
    pub sustain_level: UnipolarFloat,
    pub release: Duration,
}

/// The state information for an envelope as it evolves.
pub struct EnvelopeState {
    start: Instant,
    released: bool,
}

impl EnvelopeState {
    pub fn new() -> Self {
        Self {
            // TODO: should we initialize this time here, or closer to whenvever
            // we first received the triggering event?
            start: Instant::now(),
            released: false,
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
}
