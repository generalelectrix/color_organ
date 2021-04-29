#![allow(unused)]
use derive_more::Display;
use log::error;
use number::UnipolarFloat;
use std::time::{Duration, Instant};

fn main() {
    println!("Hello, world!");
}

/// The parameters of an ADSR envelope.
pub struct Envelope {
    attack: Duration,
    attack_level: UnipolarFloat,
    decay: Duration,
    sustain_level: UnipolarFloat,
    release: Duration,
}

/// The state information for an envelope as it evolves.
pub struct EnvelopeState {
    start: Instant,
    released: bool,
}

pub struct Color {}

/// A color, shaped by an envelope, including envelope evolution state.
pub struct ColorEvent {
    color: Color,
    envelope: Envelope,
    state: EnvelopeState,
    release_id: ReleaseID,
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
    fn new() -> Self {
        Self(Vec::new())
    }

    /// Add an event to the store.
    /// Return the ID of the stored event.
    fn add(&mut self, event: ColorEvent, listener_count: usize) -> ColorEventID {
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
    fn unlisten(&mut self, event_id: ColorEventID) {
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
