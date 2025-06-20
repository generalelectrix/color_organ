use std::time::Duration;

use crate::{color::Color, envelope::Envelope};

/// A color, shaped by an envelope, including envelope evolution state.
pub struct ColorEvent<C: Color> {
    /// The color this event is initialized with.
    color: C,
    envelope: Envelope,
    release_id: ReleaseID,
    /// The current enveloped color.
    value: C,
}

impl<C: Color> ColorEvent<C> {
    pub fn new(color: C, envelope: Envelope, release_id: ReleaseID) -> Self {
        let mut event = Self {
            color,
            envelope,
            release_id,
            value: Default::default(),
        };
        event.update_value();
        event
    }

    /// Release the envelope in this event if the release ID matches the provided one.
    pub fn release(&mut self, release_id: ReleaseID) {
        if self.release_id == release_id {
            self.envelope.release();
        }
    }

    /// Update the state of this color event.
    pub fn update_state(&mut self, delta_t: Duration) {
        self.envelope.update_state(delta_t);
        self.update_value();
    }

    /// Update the current color of this event using the current envelope value.
    fn update_value(&mut self) {
        self.value = self.color.enveloped(self.envelope.value());
    }

    /// Return the current value of this event.
    pub fn value(&self) -> &C {
        &self.value
    }

    /// Return an immutable reference to the envelope.
    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }
}

/// An identifier given to a color event to tie a subsequent off event to
/// existing running color events.  For midi inputs, this is the same as the
/// midi note.  For other organ inputs, this may be an ID managed by the
/// color event source.
pub type ReleaseID = i32;
