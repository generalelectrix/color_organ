//! Models for fixtures that can receieve color events.

use std::collections::VecDeque;

use log::error;

use crate::{
    color::Color,
    store::{ColorEventStore, ColorEventStrong},
};

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
        // Work forwards through the buffer.  For each pair of events, if the
        // newer event has completed its attack, the older event is no longer
        // relevant, and can be dropped.
    }

    /// Render the current color for this fixture.
    pub fn render(&self) -> C {
        if self.event_buffer.len() > 1 {
            unimplemented!("TODO: implement multi-event interpolation");
        }
        match self.event_buffer.front() {
            None => C::BLACK,
            Some(e) => e.borrow().value().map(Clone::clone).unwrap_or(C::BLACK),
        }
    }
}