/// Guest config sub-module specifically for
/// config templates.
use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::cpu_config::templates::{
    CpuTemplateType, GetCpuTemplate, GetCpuTemplateError, KvmCapability,
};

use crate::cpu_config::templates_serde::*;

impl GetCpuTemplate for Option<CpuTemplateType> {
    fn get_cpu_template(&self) -> Result<Cow<CustomCpuTemplate>, GetCpuTemplateError> {
        match self {
            Some(template_type) => match template_type {
                CpuTemplateType::Custom(_) => unimplemented!(),
                CpuTemplateType::Static(_) => unimplemented!(),
            },
            None => Ok(Cow::Owned(CustomCpuTemplate::default())),
        }
    }
}

/// Wrapper type to containing riscv64 CPU config modifiers.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomCpuTemplate {
    /// Additional kvm capabilities to check before
    /// configuring vcpus.
    #[serde(default)]
    pub kvm_capabilities: Vec<KvmCapability>,
    /// Modifiers of enabled vcpu features for vcpu.
    #[serde(default)]
    pub vcpu_features: Vec<VcpuFeatures>,
    /// Modifiers for registers on Aarch64 CPUs.
    #[serde(default)]
    pub reg_modifiers: Vec<RegisterModifier>,
}

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

/// Struct for defining enabled vcpu features. For now is used just a placeholder
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VcpuFeatures;

/// Wrapper of a mask defined as a bitmap to apply
/// changes to a given register's value.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct RegisterModifier;
