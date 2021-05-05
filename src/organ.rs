use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

impl<C: Color> ColorOrgan<C> {
    /// Handle a note on event.
    pub fn note_on(&mut self, color: C, velocity: UnipolarFloat, release_id: ReleaseID) {
        let event = Rc::new(RefCell::new(ColorEvent::new(
            color,
            Envelope::new(self.envelope_gen.generate()),
            release_id,
        )));
        self.event_store.add(&event);
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
