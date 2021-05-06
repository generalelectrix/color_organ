//! This module is a placeholder for types that will eventually be supplied by
//! a full-fledged patch server.
use serde::{Deserialize, Serialize};
#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FixtureId(u32);
