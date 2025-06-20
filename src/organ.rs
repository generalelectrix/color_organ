use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use number::UnipolarFloat;

use crate::{
    bank::Banks, color::Color, envelope::Envelope, envelope_gen::EnvelopeGenerator,
    event::ColorEvent, fixture::Fixture, patch::FixtureId, store::ColorEventStore,
};
use crate::{
    envelope_gen::{ControlMessage as EnvelopeControlMessage, StateChange as EnvelopeStateChange},
    event::ReleaseID,
};

pub struct ColorOrgan<C: Color> {
    envelope_gen: EnvelopeGenerator,
    event_store: ColorEventStore<C>,
    banks: Banks,
    fixture_state: HashMap<FixtureId, Fixture<C>>,
}

impl<C: Color> Default for ColorOrgan<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Color> ColorOrgan<C> {
    pub fn new() -> Self {
        Self {
            envelope_gen: EnvelopeGenerator::new(),
            event_store: ColorEventStore::new(),
            banks: Banks::new(),
            fixture_state: HashMap::new(),
        }
    }

    /// Handle a note on event.
    pub fn note_on(&mut self, color: C, velocity: UnipolarFloat, release_id: ReleaseID) {
        let event = Rc::new(RefCell::new(ColorEvent::new(
            color,
            Envelope::new(self.envelope_gen.generate()),
            release_id,
        )));
        self.event_store.add(&event);
        let fixture_state = &mut self.fixture_state;
        self.banks.next(velocity, |fixture_id| {
            // Get the fixture state for this ID.
            // If this is the first event for this fixture, create it.
            fixture_state
                .entry(fixture_id)
                .or_insert_with(Fixture::new)
                .add_event(event.clone());
        });
    }

    /// Handle a note off event.
    /// Release all of the notes with the given release ID.
    pub fn note_off(&mut self, release_id: ReleaseID) {
        self.event_store.release(release_id);
    }

    pub fn update_state(&mut self, delta_t: Duration) {
        // update the events
        self.event_store.update_state(delta_t);
        // then update the fixtures
        for (_, fixture) in self.fixture_state.iter_mut() {
            fixture.update_state();
        }
    }

    /// Render the current color for every fixture.
    pub fn render<R: EmitFixtureColor<C>>(&self, mut handler: R) {
        for (id, fixture) in self.fixture_state.iter() {
            handler(*id, fixture.render());
        }
    }

    pub fn emit_state<E: EmitStateChange>(&self, emitter: &mut E) {
        self.envelope_gen.emit_state(emitter);
    }

    pub fn control<E: EmitStateChange>(&mut self, msg: ControlMessage, emitter: &mut E) {
        use ControlMessage::*;
        match msg {
            Envelope(em) => self.envelope_gen.control(em, emitter),
        }
    }
}

pub trait EmitStateChange {
    fn emit_state_change(&mut self, sc: StateChange);
}

pub enum ControlMessage {
    Envelope(EnvelopeControlMessage),
}

pub enum StateChange {
    Envelope(EnvelopeStateChange),
}
pub trait EmitFixtureColor<C: Color>: FnMut(FixtureId, C) {}

impl<T: FnMut(FixtureId, C), C: Color> EmitFixtureColor<C> for T {}
