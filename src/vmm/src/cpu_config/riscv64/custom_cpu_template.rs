use serde::{Deserialize, Serialize};

/// Do not support any template feature for riscv64 for now.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CustomCpuTemplate;

impl CustomCpuTemplate {
    /// Get a list of register IDs that are modified by the CPU template.
    pub fn reg_list(&self) -> Vec<u64> {
        vec![]
    }

    /// Validate the correctness of the template.
    pub fn validate(&self) -> Result<(), serde_json::Error> {
        Ok(())
    }
}

