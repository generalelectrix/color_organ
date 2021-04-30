use crate::{color::Color, envelope::Envelope};

/// A color, shaped by an envelope, including envelope evolution state.
pub struct ColorEvent {
    color: Color,
    envelope: Envelope,
    release_id: ReleaseID,
}

impl ColorEvent {
    pub fn new(color: Color, envelope: Envelope, release_id: ReleaseID) -> Self {
        Self {
            color,
            envelope,
            release_id,
        }
    }

    /// Release the envelope in this event if the release ID matches the provided one.
    pub fn release(&mut self, release_id: ReleaseID) {
        if self.release_id == release_id {
            self.envelope.release();
        }
    }

    /// Return true if the envelope in this event is released.
    pub fn released(&self) -> bool {
        self.envelope.released()
    }
}

/// An identifier given to a color event to tie a subsequent off event to
/// existing running color events.  For midi inputs, this is the same as the
/// midi note.  For other organ inputs, this may be an ID managed by the
/// color event source.
pub type ReleaseID = i32;
