use eframe::egui::Ui;

use crate::envelope_gen::{ControlMessage, EnvelopeGeneratorState, StateChange};

/// UI wrapper for the envelope generator.
struct EnvelopeGenerator {
    state: EnvelopeGeneratorState,
}

impl EnvelopeGenerator {
    fn update(&mut self, sc: StateChange) {
        self.gen.update_from_state_change(&sc);
    }

    fn render<E: EmitControlMessage>(&self, ui: &mut Ui, emitter: E) {
        // TODO: do we want to optimistically update local state? Or wait for
        // response and then update?
    }
}

trait EmitControlMessage {
    fn emit_envelope_generator_control_message(&mut self, msg: ControlMessage);
}
