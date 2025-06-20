//! Models for fixtures that can receieve color events.
//! TODO: we really don't need to call these fixtures. Too overloaded of a term.
//! Should come up with something better, perhaps `Target`.
use number::UnipolarFloat;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::{color::Color, store::ColorEventStrong};

#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FixtureId(pub u32);

/// A fixture that can receive color organ events.
/// Stores a buffer of color events it is listening to, and knows how to
/// interpolate between them if multiple events are present.
pub struct Fixture<C: Color> {
    /// FIFO buffer of color events.  Newer events will evict older events after
    /// an interpolated transition.
    event_buffer: VecDeque<ColorEventStrong<C>>,
}

impl<C: Color> Fixture<C> {
    pub fn new() -> Self {
        Self {
            event_buffer: VecDeque::new(),
        }
    }

    /// Clear all events from the buffer.
    pub fn clear(&mut self) {
        self.event_buffer.clear();
    }

    pub fn add_event(&mut self, event: ColorEventStrong<C>) {
        self.event_buffer.push_front(event);
    }

    /// Update the state of this fixture's event buffer.
    /// Drop all events that the fixture is no longer responding to.
    pub fn update(&mut self) {
        // Short-circuit if the event buffer is empty.
        if self.event_buffer.is_empty() {
            return;
        }
        // If an event has completed its attack, all older events are no longer relevant.
        // Iterate through the events, and as soon as we find one with a complete
        // attack, discard the rest.
        if let Some(newest_complete) = self
            .event_buffer
            .iter()
            .position(|e| e.borrow().envelope().attack_complete())
        {
            self.event_buffer.truncate(newest_complete + 1);
        }

        // If we're down to one event in the buffer and it is complete, drop it.
        if self.event_buffer.len() == 1 && self.event_buffer[0].borrow().envelope().closed() {
            self.event_buffer.clear();
        }
    }

    /// Return the current color for this fixture.
    pub fn render(&self) -> C {
        // Fold backwards over all events in the buffer, interpolating each pair
        // of color events from the oldest to the newest.
        self.event_buffer
            .iter()
            .rev()
            .fold(None, |color_accum, event| match color_accum {
                None => Some(event.borrow().value().clone()),
                Some(color_accum) => {
                    let e = event.borrow();
                    if e.envelope().attack_complete() {
                        return Some(e.value().clone());
                    }
                    Some(color_accum.weighted_interpolation(
                        e.value(),
                        e.envelope().value().unwrap_or(UnipolarFloat::ZERO),
                    ))
                }
            })
            .unwrap_or_default()
    }
}
