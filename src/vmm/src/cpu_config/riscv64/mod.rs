/// Module for custom CPU templates
pub mod custom_cpu_template;
/// Module for static CPU templates
pub mod static_cpu_templates;

use super::templates::CustomCpuTemplate;
use crate::arch::riscv64::vcpu::VcpuArchError;

/// Errors thrown while configuring templates.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Failed to create a guest cpu configuration: {0}")]
pub struct CpuConfigurationError(#[from] pub VcpuArchError);

/// CPU configuration for riscv64
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CpuConfiguration;

impl CpuConfiguration {
    /// Creates new guest CPU config based on the provided template
    pub fn apply_template(
        mut self,
        template: &CustomCpuTemplate,
    ) -> Result<Self, CpuConfigurationError> {
        Ok(self)
    }

    /// Returns ids of registers that are changed
    /// by this template
    pub fn register_ids(&self) -> Vec<u64> {
        self.regs.iter().map(|reg| reg.id).collect()
    }
}

