#![allow(unused)]
use derive_more::Display;
use log::error;
use number::UnipolarFloat;
use std::time::{Duration, Instant};

fn main() {
    println!("Hello, world!");
}

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
}

pub struct Color {}

/// A color, shaped by an envelope, including envelope evolution state.
pub struct ColorEvent {
    color: Color,
    envelope: Envelope,
    state: EnvelopeState,
    release_id: ReleaseID,
}

impl ColorEvent {
    pub fn new(color: Color, envelope: Envelope, release_id: ReleaseID) -> Self {
        Self {
            color,
            envelope,
            release_id,
            state: EnvelopeState::new(),
        }
    }
}

/// An identifier given to a color event to tie a subsequent off event to
/// existing running color events.  For midi inputs, this is the same as the
/// midi note.  For other organ inputs, this may be an ID managed by the
/// color event source.
pub type ReleaseID = i32;

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

    /// Return the number of active events.
    pub fn active_event_count(&self) -> usize {
        self.0.iter().filter(|slot| slot.is_some()).count()
    }

    /// Add an event to the store.
    /// Return the ID of the stored event.
    pub fn add(&mut self, event: ColorEvent, listener_count: usize) -> ColorEventID {
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_color_event_store() {
        let mut store = ColorEventStore::new();
        assert_eq!(0, store.active_event_count());
        let release_id = 0;
        let event = ColorEvent::new(Color {}, test_envelope(), release_id);
        let event_id = store.add(event, 1);
        assert_eq!(1, store.active_event_count());
        store.unlisten(event_id);
        assert_eq!(0, store.active_event_count());
    }

    fn test_envelope() -> Envelope {
        Envelope {
            attack: Duration::from_secs(1),
            attack_level: UnipolarFloat::ZERO,
            decay: Duration::from_secs(1),
            sustain_level: UnipolarFloat::new(0.5),
            release: Duration::from_secs(1),
        }
    }
}
