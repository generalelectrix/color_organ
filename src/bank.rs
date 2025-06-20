use crate::fixture::FixtureId;

use number::UnipolarFloat;
use serde::{Deserialize, Serialize};

/// The collection of banks defined for a color organ.
#[derive(Serialize, Deserialize)]
pub struct Banks {
    banks: Vec<Bank>,
    current_bank: Option<usize>,
}

impl Banks {
    pub fn new(fixture_count: usize) -> Self {
        Self {
            banks: vec![Bank {
                name: "test".to_string(),
                sequences: vec![PatternSequence::Run(FixtureRun {
                    fixtures: (0u32..fixture_count as u32).map(FixtureId).collect(),
                    current_index: 0,
                })],
                current_sequence: Some(0),
            }],
            current_bank: None,
        }
    }

    /// Next passes the fixture IDs in the next pattern to handler.
    pub fn next<T: UseFixtureId>(&mut self, velocity: UnipolarFloat, handler: T) {
        if let Some(bank_id) = self.current_bank {
            self.banks[bank_id].next(velocity, handler);
        }
    }

    /// Return the name of the current bank.
    pub fn current_bank(&self) -> Option<&str> {
        self.current_bank.map(|id| self.banks[id].name.as_ref())
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
    /// Velocity will eventually be used to support velocity bucketing.
    pub fn next<T: UseFixtureId>(&mut self, _velocity: UnipolarFloat, handler: T) {
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
    fn next<T: UseFixtureId>(&mut self, mut handler: T) -> bool {
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
    fn next<T: UseFixtureId>(&mut self, mut handler: T) -> bool {
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
pub trait UseFixtureId: FnMut(FixtureId) {}

impl<T: FnMut(FixtureId)> UseFixtureId for T {}
