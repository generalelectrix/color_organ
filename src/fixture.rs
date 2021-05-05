//! Models for fixtures that can receieve color events.

use number::UnipolarFloat;
use std::collections::VecDeque;

use crate::{color::Color, store::ColorEventStrong};

/// A fixture that can receive color organ events.
/// Stores a buffer or color events it is listening to, and knows how to
/// interpolate between them if multiple events are present.
pub struct Fixture<C: Color> {
    name: String,
    /// FIFO buffer of color events.  Newer events will evict older events after
    /// an interpolated transition.
    event_buffer: VecDeque<ColorEventStrong<C>>,
}

impl<C: Color> Fixture<C> {
    pub fn new<N: Into<String>>(name: N) -> Self {
        Self {
            name: name.into(),
            event_buffer: VecDeque::new(),
        }
    }

    pub fn add_event(&mut self, event: ColorEventStrong<C>) {
        self.event_buffer.push_back(event);
    }

    /// Update the state of this fixture's event buffer.
    /// Drop all events that the fixture is no longer responding to.
    pub fn update_state(&mut self) {
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
        // of colors.
        self.event_buffer
            .iter()
            .rev()
            .fold(None, |color_accum, event| match color_accum {
                None => Some(event.borrow().value().clone()),
                Some(color) => {
                    let e = event.borrow();
                    Some(e.value().weighted_interpolation(
                        &color,
                        e.envelope().value().unwrap_or(UnipolarFloat::ZERO),
                    ))
                }
            })
            .unwrap_or(C::BLACK)
    }
}
