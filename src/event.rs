use crate::{color::Color, envelope::Envelope};

/// A color, shaped by an envelope, including envelope evolution state.
pub struct ColorEvent {
    pub color: Color,
    pub envelope: Envelope,
    pub release_id: ReleaseID,
}

impl ColorEvent {
    pub fn new(color: Color, envelope: Envelope, release_id: ReleaseID) -> Self {
        Self {
            color,
            envelope,
            release_id,
        }
    }
}

/// An identifier given to a color event to tie a subsequent off event to
/// existing running color events.  For midi inputs, this is the same as the
/// midi note.  For other organ inputs, this may be an ID managed by the
/// color event source.
pub type ReleaseID = i32;
