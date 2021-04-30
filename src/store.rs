use derive_more::Display;
use log::error;
use number::UnipolarFloat;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    time::{Duration, Instant},
};

use crate::{
    color::Color,
    event::{ColorEvent, ReleaseID},
};

/// A strong reference to a color event.
pub type ColorEventStrong<C> = Rc<RefCell<ColorEvent<C>>>;

/// A weak reference to a color event.
type ColorEventWeak<C> = Weak<RefCell<ColorEvent<C>>>;

/// A collection of active color events.
pub struct ColorEventStore<C: Color>(Vec<ColorEventWeak<C>>);

impl<C: Color> ColorEventStore<C> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, event: &ColorEventStrong<C>) {
        self.0.push(Rc::downgrade(event));
    }

    /// Release all events with the given release ID.
    pub fn release(&mut self, release_id: ReleaseID) {
        for event in self.0.iter() {
            if let Some(e) = event.upgrade() {
                e.borrow_mut().release(release_id);
            }
        }
    }

    /// Remove all events that are no longer alive.
    fn clean(&mut self) {
        self.0.retain(|e| e.strong_count() > 0);
    }
}

#[cfg(test)]
mod test {
    use number::Phase;

    use crate::{
        color::{Color, HsvColor},
        envelope::{Envelope, EnvelopeParameters},
    };

    use super::*;

    fn mkevent(release_id: ReleaseID) -> ColorEventStrong<HsvColor> {
        Rc::new(RefCell::new(ColorEvent::new(
            color(),
            envelope(),
            release_id,
        )))
    }

    #[test]
    fn test_release() {
        let mut store = ColorEventStore::new();
        let event_0 = mkevent(0);
        let event_1 = mkevent(1);
        store.add(&event_0);
        store.add(&event_1);
        store.release(0);
        assert!(event_0.borrow().released());
        assert!(!event_1.borrow().released());
    }

    fn envelope() -> Envelope {
        Envelope::new(EnvelopeParameters::linear(
            Duration::from_secs(1),
            UnipolarFloat::ZERO,
            Duration::from_secs(1),
            UnipolarFloat::new(0.5),
            Duration::from_secs(1),
        ))
    }

    fn color() -> HsvColor {
        HsvColor {
            hue: Phase::ZERO,
            saturation: UnipolarFloat::ONE,
            value: UnipolarFloat::ONE,
        }
    }
}
