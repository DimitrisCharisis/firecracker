use std::fmt::Debug;

use kvm_bindings::*;
use kvm_ioctls::{VcpuExit, VcpuFd};
use serde::{Deserialize, Serialize};

use super::regs::*;
use crate::cpu_config::templates::CpuConfiguration;
use crate::logger::{error, IncMetric, METRICS};
use crate::vcpu::VcpuError;
use crate::vstate::vcpu::VcpuEmulation;
use crate::vstate::vm::Vm;

/// Errors thrown while setting riscv64 registers.
#[derive(Debug, PartialEq, Eq, thiserror::Error, displaydoc::Display)]
pub enum VcpuArchError {
    /// Failed to get register {0}: {1}
    GetOneReg(u64, kvm_ioctls::Error),
    /// Failed to set register {0}: {1}
    SetOneReg(u64, kvm_ioctls::Error),
    /// Failed to retrieve list of registers: {0}
    GetRegList(kvm_ioctls::Error),
    /// Failed to get multiprocessor state: {0}
    GetMp(kvm_ioctls::Error),
    /// Failed to set multiprocessor state: {0}
    SetMp(kvm_ioctls::Error),
}

/// Errors associated with the wrappers over KVM ioctls.
#[derive(Debug, PartialEq, Eq, thiserror::Error, displaydoc::Display)]
pub enum KvmVcpuError {
    /// Error configuring the vcpu registers: {0}
    ConfigureRegisters(VcpuArchError),
    /// Error creating vcpu: {0}
    CreateVcpu(kvm_ioctls::Error),
    /// Failed to dump CPU configuration: {0}
    DumpCpuConfig(VcpuArchError),
    /// Error getting the vcpu preferred target: {0}
    GetPreferredTarget(kvm_ioctls::Error),
    /// Error initializing the vcpu: {0}
    Init(kvm_ioctls::Error),
    /// Error applying template: {0}
    ApplyCpuTemplate(VcpuArchError),
    /// Failed to restore the state of the vcpu: {0}
    RestoreState(VcpuArchError),
    /// Failed to save the state of the vcpu: {0}
    SaveState(VcpuArchError),
}

/// Error type for [`KvmVcpu::configure`].
pub type KvmVcpuConfigureError = KvmVcpuError;

/// A wrapper around creating and using a kvm riscv64 vcpu.
#[derive(Debug)]
pub struct KvmVcpu {
    /// Index of vcpu.
    pub index: u8,
    /// KVM vcpu fd.
    pub fd: VcpuFd,
    /// Vcpu peripherals, such as buses
    pub peripherals: Peripherals,
}

/// Vcpu peripherals
#[derive(Default, Debug)]
pub struct Peripherals {
    /// mmio bus.
    pub mmio_bus: Option<crate::devices::Bus>,
}

impl KvmVcpu {
    /// Constructs a new kvm vcpu with arch specific functionality.
    ///
    /// # Arguments
    ///
    /// * `index` - Represents the 0-based CPU index between [0, max vcpus).
    /// * `vm` - The vm to which this vcpu will get attached.
    pub fn new(index: u8, vm: &Vm) -> Result<Self, KvmVcpuError> {
        let kvm_vcpu = vm
            .fd()
            .create_vcpu(index.into())
            .map_err(KvmVcpuError::CreateVcpu)?;

        Ok(KvmVcpu {
            index,
            fd: kvm_vcpu,
            peripherals: Default::default(),
        })
    }

    /// Save the KVM internal state.
    pub fn save_state(&self) -> Result<VcpuState, KvmVcpuError> {
        unimplemented!();
    }

    /// Use provided state to populate KVM internal state.
    pub fn restore_state(&mut self, state: &VcpuState) -> Result<(), KvmVcpuError> {
        unimplemented!();
    }

    /// Dumps CPU configuration.
    pub fn dump_cpu_config(&self) -> Result<CpuConfiguration, KvmVcpuError> {
        unimplemented!();
    }
}

impl Peripherals {
    /// Runs the vCPU in KVM context and handles the kvm exit reason.
    ///
    /// Returns error or enum specifying whether emulation was handled or interrupted.
    pub fn run_arch_emulation(&self, exit: VcpuExit) -> Result<VcpuEmulation, VcpuError> {
        METRICS.vcpu.failures.inc();
        // TODO: Are we sure we want to finish running a vcpu upon
        // receiving a vm exit that is not necessarily an error?
        error!("Unexpected exit reason on vcpu run: {:?}", exit);
        Err(VcpuError::UnhandledKvmExit(format!("{:?}", exit)))
    }
}

/// Structure holding VCPU kvm state.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VcpuState {
    /// Multiprocessing state.
    pub mp_state: kvm_mp_state,
    /// Vcpu registers.
    pub regs: Riscv64RegisterVec,
}
