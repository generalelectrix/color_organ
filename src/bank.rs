use crate::patch::FixtureId;

pub struct Banks;

impl Banks {}

/// A bank is a collection of pattern sequences.
/// TODO: do we want to implement velocity bucketing?
pub struct Bank {
    sequences: Vec<PatternSequence>,
}

/// Options for color organ pattern sequences.
pub enum PatternSequence {
    /// A static collection of fixtures.
    FixtureSet(Vec<FixtureId>),
    /// TODO: sequence generators
    /// This is a proof of concept to ensure that we develop a stateful API
    /// for future support of more sophisticated generators.
    Run((Vec<FixtureId>, usize)),
}
