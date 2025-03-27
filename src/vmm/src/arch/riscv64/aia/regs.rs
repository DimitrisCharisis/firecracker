use serde::{Deserialize, Serialize};

/// Structure used for serializing the state of the GIC registers.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AiaState; // Unimplemented for now
