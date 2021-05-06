use crate::patch::FixtureId;

use serde::{Deserialize, Serialize};

/// The collection of banks defined for a color organ.
#[derive(Serialize, Deserialize)]
pub struct Banks {
    banks: Vec<Bank>,
    current_bank: Option<usize>,
}

impl Banks {
    pub fn new() -> Self {
        Self {
            banks: Vec::new(),
            current_bank: None,
        }
    }

    /// Next passes the fixture IDs in the next pattern to handler.
    pub fn next<T: UseFixtureId>(&mut self, handler: T) {
        if let Some(bank_id) = self.current_bank {
            self.banks[bank_id].next(handler);
        }
    }
}

/// A bank is a collection of pattern sequences.
/// TODO: do we want to implement velocity bucketing?
#[derive(Clone, Serialize, Deserialize)]
pub struct Bank {
    name: String,
    sequences: Vec<PatternSequence>,
    current_sequence: Option<usize>,
}

impl Bank {
    /// Next passes the fixture IDs in the next pattern to handler.
    pub fn next<T: UseFixtureId>(&mut self, handler: T) {
        if let Some(sequence_id) = self.current_sequence {
            let has_more = self.sequences[sequence_id].next(handler);
            if !has_more {
                // Advance to the next sequence.
                let next_sequence_id = (sequence_id + 1) % self.sequences.len();
                self.current_sequence = Some(next_sequence_id);
            }
        }
    }
}

/// Options for color organ pattern sequences.
#[derive(Clone, Serialize, Deserialize)]
enum PatternSequence {
    /// A static collection of fixtures.
    FixtureSet(Vec<FixtureId>),
    /// TODO: sequence generators
    /// This is a proof of concept to ensure that we develop a stateful API
    /// for future support of more sophisticated generators.
    Run(FixtureRun),
}

impl PatternSequence {
    /// Call handler on each of the fixture IDs in the next pattern.
    /// Return true if this sequence has additional patterns after this one.
    fn next<T: UseFixtureId>(&mut self, handler: T) -> bool {
        use PatternSequence::*;
        match self {
            FixtureSet(fixtures) => {
                for id in fixtures {
                    handler(*id);
                }
                false
            }
            Run(run) => run.next(handler),
        }
    }
}

/// A linear run pattern sequence, one fixture at a time.
#[derive(Clone, Serialize, Deserialize)]
struct FixtureRun {
    fixtures: Vec<FixtureId>,
    current_index: usize,
}

impl FixtureRun {
    fn next<T: UseFixtureId>(&mut self, handler: T) -> bool {
        handler(self.fixtures[self.current_index]);
        if self.current_index == self.fixtures.len() - 1 {
            self.current_index = 0;
            false
        } else {
            self.current_index += 1;
            true
        }
    }
}

/// Trait for a closure passed into the current bank, called once with each
/// fixture ID in the current pattern.
pub trait UseFixtureId: Fn(FixtureId) {}

impl<T: Fn(FixtureId)> UseFixtureId for T {}
