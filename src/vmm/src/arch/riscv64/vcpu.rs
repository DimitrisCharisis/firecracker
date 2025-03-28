use std::fmt::Debug;

use kvm_bindings::*;
use kvm_ioctls::{VcpuExit, VcpuFd};
use serde::{Deserialize, Serialize};

use super::regs::*;
use crate::arch::EntryPoint;
use crate::cpu_config::templates::CpuConfiguration;
use crate::logger::{error, IncMetric, METRICS};
use crate::vcpu::{VcpuConfig, VcpuError};
use crate::vstate::memory::{Address, GuestMemoryMmap};
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

// Three mandatory things to set before booting a vCPU
// a0 => hart/core id
// pc => kernel entry point
// a1 => fdt start
pub fn setup_boot_regs(
    vcpufd: &VcpuFd,
    cpu_id: u8,
    kernel_entry_addr: u64,
) -> Result<(), VcpuArchError> {
    let mut bytes = [0u8; 8];
    let off_isa = offset_of!(kvm_riscv_config, isa);
    let id_isa = riscv64_reg_config_id!(off_isa);

    // TODO: Why we get and set the the isa register? [this is what the kvmtool does]
    vcpufd
        .get_one_reg(id_isa, &mut bytes)
        .map_err(|err| VcpuArchError::GetOneReg(id_isa, err))?;

    vcpufd
        .set_one_reg(id_isa, &mut bytes)
        .map_err(|err| VcpuArchError::SetOneReg(id_isa, err))?;

    let off_a0 = offset_of!(kvm_riscv_core, regs, user_regs_struct, a0);
    let id_a0 = riscv64_reg_core_id!(off_a0);
    vcpufd
        .set_one_reg(id_a0, &u64::from(cpu_id).to_le_bytes())
        .map_err(|err| VcpuArchError::SetOneReg(id_a0, err))?;

    let off_pc = offset_of!(kvm_riscv_core, regs, user_regs_struct, pc);
    let id_pc = riscv64_reg_core_id!(off_pc);
    vcpufd
        .set_one_reg(id_pc, &kernel_entry_addr.to_le_bytes())
        .map_err(|err| VcpuArchError::SetOneReg(id_pc, err))?;

    let fdt_start: u64 = super::layout::FDT_START;
    let off_a1 = offset_of!(kvm_riscv_core, regs, user_regs_struct, a1);
    let id_a1 = riscv64_reg_core_id!(off_a1);
    vcpufd
        .set_one_reg(id_a1, &fdt_start.to_le_bytes())
        .map_err(|err| VcpuArchError::SetOneReg(id_a1, err))?;

    let timer_freq_off = offset_of!(kvm_riscv_timer, frequency);
    let timer_freq_id = riscv64_reg_timer_id!(timer_freq_off);
    let b = &mut [0u8; 8];
    vcpufd
        .get_one_reg(timer_freq_id, b)
        .map_err(|err| VcpuArchError::GetOneReg(timer_freq_id, err))?;

    // vcpu.timer_freq = Some(u64::from_le_bytes(*b));

    Ok(())
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

    /// Configures an aarch64 specific vcpu for booting Linux.
    ///
    /// # Arguments
    ///
    /// * `guest_mem` - The guest memory used by this microvm.
    /// * `kernel_entry_point` - Specifies the boot protocol and offset from `guest_mem` at which
    ///   the kernel starts.
    /// * `vcpu_config` - The vCPU configuration.
    pub fn configure(
        &mut self,
        guest_mem: &GuestMemoryMmap,
        kernel_entry_point: EntryPoint,
        vcpu_config: &VcpuConfig,
    ) -> Result<(), KvmVcpuError> {
        // TODO: Do not set any cpu configuration register
        // for reg in vcpu_config.cpu_config.regs.iter() {
        //     self.fd.set_one_reg(reg.id, reg.as_slice()).map_err(|err| {
        //         KvmVcpuError::ApplyCpuTemplate(VcpuArchError::SetOneReg(reg.id, err))
        //     })?;
        // }

        setup_boot_regs(
            &self.fd,
            self.index,
            kernel_entry_point.entry_addr.raw_value(),
        )
        .map_err(KvmVcpuError::ConfigureRegisters)?;

        Ok(())
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
