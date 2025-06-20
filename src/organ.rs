use std::{cell::RefCell, rc::Rc, time::Duration};

use log::error;
use number::UnipolarFloat;

use crate::{
    bank::Banks,
    color::Color,
    envelope::Envelope,
    envelope_gen::EnvelopeGenerator,
    event::ColorEvent,
    fixture::{Fixture, FixtureId},
    store::ColorEventStore,
    HsluvColor,
};
use crate::{
    envelope_gen::{ControlMessage as EnvelopeControlMessage, StateChange as EnvelopeStateChange},
    event::ReleaseId,
};

pub type ColorOrganHsluv = ColorOrgan<HsluvColor>;

pub struct ColorOrgan<C: Color> {
    envelope_gen: EnvelopeGenerator,
    event_store: ColorEventStore<C>,
    banks: Banks,
    fixture_state: Vec<Fixture<C>>,
}

impl<C: Color> ColorOrgan<C> {
    pub fn new(fixture_count: usize) -> Self {
        Self {
            envelope_gen: EnvelopeGenerator::new(),
            event_store: ColorEventStore::new(),
            banks: Banks::new(),
            fixture_state: (0..fixture_count).map(|_| Fixture::new()).collect(),
        }
    }

    /// Handle a note on event.
    pub fn note_on<T: Into<C>>(
        &mut self,
        color: T,
        velocity: UnipolarFloat,
        release_id: ReleaseId,
    ) {
        let event = Rc::new(RefCell::new(ColorEvent::new(
            color.into(),
            Envelope::new(self.envelope_gen.generate()),
            release_id,
        )));
        self.event_store.add(&event);
        self.banks.next(velocity, |fixture_id| {
            // Get the fixture state for this ID.
            let Some(fixture) = self.fixture_state.get_mut(fixture_id.0 as usize) else {
                error!("fixture ID {} out of range", fixture_id.0);
                return;
            };
            fixture.add_event(event.clone());
        });
    }

    /// Handle a note off event.
    /// Release all of the notes with the given release ID.
    pub fn note_off(&mut self, release_id: ReleaseId) {
        self.event_store.release(release_id);
    }

    pub fn update_state(&mut self, delta_t: Duration) {
        // update the events
        self.event_store.update_state(delta_t);
        // then update the fixtures
        for fixture in self.fixture_state.iter_mut() {
            fixture.update_state();
        }
    }

    /// Get the current color for a specific fixture by ID.
    ///
    /// Return None if the ID is out of range.
    pub fn render(&self, id: FixtureId) -> Option<C> {
        self.fixture_state.get(id.0 as usize).map(Fixture::render)
    }

    pub fn emit_state<E: EmitStateChange>(&self, emitter: &E) {
        self.envelope_gen.emit_state(emitter);
    }

    pub fn control<E: EmitStateChange, T: Into<C>>(&mut self, msg: ControlMessage<T>, emitter: &E) {
        use ControlMessage::*;
        match msg {
            Envelope(em) => self.envelope_gen.control(em, emitter),
            NoteOn {
                color,
                velocity,
                release_id,
            } => self.note_on(color, velocity, release_id),
            NoteOff(release_id) => self.note_off(release_id),
            Blackout => {
                for fixture in &mut self.fixture_state {
                    fixture.clear();
                }
            }
        }
    }
}

pub trait EmitStateChange {
    fn emit_state_change(&self, sc: StateChange);
}

#[derive(Clone, Debug)]
pub enum ControlMessage<C> {
    Envelope(EnvelopeControlMessage),
    NoteOn {
        color: C,
        velocity: UnipolarFloat,
        release_id: ReleaseId,
    },
    NoteOff(ReleaseId),
    /// Clear all fixture state, creating an instant blackout.
    Blackout,
}

pub enum StateChange {
    Envelope(EnvelopeStateChange),
}

pub struct IgnoreEmitter;

impl EmitStateChange for IgnoreEmitter {
    fn emit_state_change(&self, _sc: StateChange) {}
}
