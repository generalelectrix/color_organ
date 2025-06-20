use std::time::Duration;

use number::UnipolarFloat;

use crate::envelope::EnvelopeParameters;
use crate::organ::{EmitStateChange as EmitOrganStateChange, StateChange as OrganStateChange};

/// Generate envelope parameters based on higher-level controls.
/// TODO: envelope shape controls, linear defaults now.
pub struct EnvelopeGenerator {
    attack: UnipolarFloat,
    attack_level: UnipolarFloat,
    decay: UnipolarFloat,
    sustain_level: UnipolarFloat,
    release: UnipolarFloat,
    /// The unit of time associated with the envelope paramters.
    /// For example, if attack is 1, it will have this length.
    time_scale: Duration,
}

impl EnvelopeGenerator {
    pub fn new() -> Self {
        Self {
            attack: UnipolarFloat::ONE,
            attack_level: UnipolarFloat::ZERO,
            decay: UnipolarFloat::ONE,
            sustain_level: UnipolarFloat::ONE,
            release: UnipolarFloat::ONE,
            time_scale: Duration::from_secs(1),
        }
    }

    /// Generate current envelope parameters.
    pub fn generate(&self) -> EnvelopeParameters {
        EnvelopeParameters::linear(
            self.time_scale.mul_f64(self.attack.val()),
            self.attack_level,
            self.time_scale.mul_f64(self.decay.val()),
            self.sustain_level,
            self.time_scale.mul_f64(self.release.val()),
        )
    }

    /// Emit all observable state using the provided emitter.
    pub fn emit_state<E: EmitStateChange>(&self, emitter: &E) {
        use StateChange::*;
        emitter.emit_envelope_generator_state_change(Attack(self.attack));
        emitter.emit_envelope_generator_state_change(AttackLevel(self.attack_level));
        emitter.emit_envelope_generator_state_change(Decay(self.decay));
        emitter.emit_envelope_generator_state_change(SustainLevel(self.sustain_level));
        emitter.emit_envelope_generator_state_change(Release(self.release));
        emitter.emit_envelope_generator_state_change(TimeScale(self.time_scale));
    }

    /// Handle a control message.
    pub fn control<E: EmitStateChange>(&mut self, msg: ControlMessage, emitter: &E) {
        use ControlMessage::*;
        match msg {
            Set(sc) => self.handle_state_change(sc, emitter),
        }
    }

    fn handle_state_change<E: EmitStateChange>(&mut self, sc: StateChange, emitter: &E) {
        use StateChange::*;
        match sc {
            Attack(v) => self.attack = v,
            AttackLevel(v) => self.attack_level = v,
            Decay(v) => self.decay = v,
            SustainLevel(v) => self.sustain_level = v,
            Release(v) => self.release = v,
            TimeScale(v) => self.time_scale = v,
        };
        emitter.emit_envelope_generator_state_change(sc);
    }
}

#[derive(Clone, Debug)]
pub enum ControlMessage {
    Set(StateChange),
}

#[derive(Clone, Debug)]
pub enum StateChange {
    Attack(UnipolarFloat),
    AttackLevel(UnipolarFloat),
    Decay(UnipolarFloat),
    SustainLevel(UnipolarFloat),
    Release(UnipolarFloat),
    TimeScale(Duration),
}

pub trait EmitStateChange {
    fn emit_envelope_generator_state_change(&self, sc: StateChange);
}

impl<T: EmitOrganStateChange> EmitStateChange for T {
    fn emit_envelope_generator_state_change(&self, sc: StateChange) {
        self.emit_state_change(OrganStateChange::Envelope(sc));
    }
}
