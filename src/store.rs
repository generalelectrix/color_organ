use derive_more::Display;
use log::error;
use number::UnipolarFloat;
use std::time::{Duration, Instant};

use crate::event::{ColorEvent, ReleaseID};

/// An ID that uniquely identifies a color event.
/// These IDs may be re-used once a color event has no existing subscribers.
#[derive(Display)]
pub struct ColorEventID(usize);

/// An active color event being listened to.
struct ActiveColorEvent {
    event: ColorEvent,
    listener_count: usize,
}

/// A collection of active color events.
/// Represented as Options to allow re-using the same memory for new events.
/// Tracks the number of listeners for each event, allowing for re-use of a slot
/// once the listener count hits zero.
pub struct ColorEventStore(Vec<Option<ActiveColorEvent>>);

impl ColorEventStore {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Get the event with the given id.
    pub fn get(&self, event_id: ColorEventID) -> Option<&ColorEvent> {
        match &self.0[event_id.0] {
            None => None,
            Some(e) => Some(&e.event),
        }
    }

    /// Return the number of active events.
    pub fn active_event_count(&self) -> usize {
        self.0.iter().filter(|slot| slot.is_some()).count()
    }

    /// Add an event to the store.
    /// Return the ID of the stored event.
    pub fn add(&mut self, event: ColorEvent, listener_count: usize) -> ColorEventID {
        assert_ne!(0, listener_count);
        let active_event = ActiveColorEvent {
            event,
            listener_count,
        };
        for (i, slot) in self.0.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(active_event);
                return ColorEventID(i);
            }
        }
        // No free slots, push a new one.
        self.0.push(Some(active_event));

        ColorEventID(self.0.len() - 1)
    }

    /// Decrement the listener count of the specified ID.
    /// If the listener count hits 0, clear the slot.
    pub fn unlisten(&mut self, event_id: ColorEventID) {
        match &mut self.0[event_id.0] {
            None => {
                error!("unlisten on empty slot ID {}", event_id.0);
                return;
            }
            Some(slot) => {
                slot.listener_count -= 1;
                if slot.listener_count == 0 {
                    self.0[event_id.0] = None
                }
            }
        }
    }

    /// Release all events with the given release ID.
    pub fn release(&mut self, release_id: ReleaseID) {
        self.0
            .iter_mut()
            .filter_map(|slot| slot.as_mut())
            .filter(|stored| stored.event.release_id == release_id)
            .for_each(|stored| stored.event.state.release());
    }
}

#[cfg(test)]
mod test {
    use crate::{
        color::Color,
        envelope::{Envelope, EnvelopeParameters},
    };

    use super::*;

    #[test]
    fn test_add_unlisten() {
        let mut store = ColorEventStore::new();
        assert_eq!(0, store.active_event_count());
        let release_id = 0;
        let event = ColorEvent::new(Color {}, envelope(), release_id);
        let event_id = store.add(event, 1);
        assert_eq!(1, store.active_event_count());
        store.unlisten(event_id);
        assert_eq!(0, store.active_event_count());
    }

    #[test]
    fn test_release() {
        let mut store = ColorEventStore::new();
        let event_0 = store.add(ColorEvent::new(Color {}, envelope(), 0), 1);
        let event_1 = store.add(ColorEvent::new(Color {}, envelope(), 1), 1);
        store.release(0);
        assert!(store.get(event_0).unwrap().state.released());
        assert!(!store.get(event_1).unwrap().state.released());
    }

    fn envelope() -> Envelope {
        Envelope::new(EnvelopeParameters {
            attack: Duration::from_secs(1),
            attack_level: UnipolarFloat::ZERO,
            decay: Duration::from_secs(1),
            sustain_level: UnipolarFloat::new(0.5),
            release: Duration::from_secs(1),
        })
    }
}
