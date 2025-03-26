use kvm_ioctls::VmFd;
use serde::{Deserialize, Serialize};

use crate::Kvm;
use crate::arch::riscv64::aia::AiaState;
use crate::vstate::vm::VmError;

/// Structure representing the current architecture's understand of what a "virtual machine" is.
#[derive(Debug)]
pub struct ArchVm {
    /// KVM file descriptor of microVM
    pub fd: VmFd,
    // On aarch64 we need to keep around the fd obtained by creating the VGIC device.
    irqchip_handle: Option<crate::arch::riscv64::aia::AIADevice>,
}

/// Error type for [`Vm::restore_state`]
#[derive(Debug, PartialEq, Eq, thiserror::Error, displaydoc::Display)]
pub enum ArchVmError {
    /// Error creating the global interrupt controller: {0}
    VmCreateAIA(crate::arch::riscv64::aia::AiaError),
    /// Failed to save the VM's AIA state: {0}
    SaveAia(crate::arch::riscv64::aia::AiaError),
    /// Failed to restore the VM's AIA state: {0}
    RestoreAia(crate::arch::riscv64::aia::AiaError),
}

impl ArchVm {
    /// Create a new `Vm` struct.
    pub fn new(kvm: &Kvm) -> Result<ArchVm, VmError> {
        let fd = Self::create_vm(kvm)?;
        Ok(ArchVm {
            fd,
            irqchip_handle: None,
        })
    }

    /// Pre-vCPU creation setup.
    pub fn arch_pre_create_vcpus(&mut self, _: u8) -> Result<(), ArchVmError> {
        Ok(())
    }

    /// Post-vCPU creation setup.
    pub fn arch_post_create_vcpus(&mut self, nr_vcpus: u8) -> Result<(), ArchVmError> {
        self.setup_irqchip(nr_vcpus)
    }

    /// Creates the AIA (Advanced Interrupt Architecture) IRQchip.
    pub fn setup_irqchip(&mut self, vcpu_count: u8) -> Result<(), ArchVmError> {
        self.irqchip_handle = Some(
            crate::arch::riscv64::aia::create_gic(&self.fd, vcpu_count.into(), None)
                .map_err(ArchVmError::VmCreateAIA)?,
        );
        Ok(())
    }

    /// Gets a reference to the irqchip of the VM.
    pub fn get_irqchip(&self) -> &crate::arch::riscv64::aia::AIADevice {
        self.irqchip_handle.as_ref().expect("IRQ chip not set")
    }

    /// Saves and returns the Kvm Vm state.
    pub fn save_state(&self) -> Result<VmState, ArchVmError> {
        unimplmented!()
    }

    /// Restore the KVM VM state
    pub fn restore_state(&mut self) -> Result<(), ArchVmError> {
        unimplemented!()
    }
}

/// Structure holding an general specific VM state.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VmState {
    /// AIA state.
    pub aia: AiaState,
}

