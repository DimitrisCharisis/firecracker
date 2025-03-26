/// Module for the global interrupt controller configuration.
pub mod aia;
mod fdt;
/// Architecture specific KVM-related code
pub mod kvm;
/// Layout for this riscv64 system.
pub mod layout;
/// Logic for configuring riscv64 registers.
pub mod regs;
/// Architecture specific vCPU code
pub mod vcpu;
/// Architecture specific VM state code
pub mod vm;

use vm_memory::GuestMemoryError;

use crate::vstate::memory::GuestAddress;

/// Errors thrown while configuring riscv64 system.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum ConfigurationError {
    /// Failed to create a Flattened Device Tree for this riscv64 microVM: {0}
    SetupFDT(#[from] fdt::FdtError),
    /// Failed to write to guest memory.
    MemoryError(GuestMemoryError),
}

/// The start of the memory area reserved for MMIO devices.
pub const MMIO_MEM_START: u64 = layout::MAPPED_IO_START;
/// The size of the memory area reserved for MMIO devices.
pub const MMIO_MEM_SIZE: u64 = layout::DRAM_MEM_START - layout::MAPPED_IO_START; //>> 1GB

/// Returns a Vec of the valid memory addresses for riscv64.
/// See [`layout`](layout) module for a drawing of the specific memory model for this platform.
pub fn arch_memory_regions(size: usize) -> Vec<(GuestAddress, usize)> {
    let dram_size = min(size, layout::DRAM_MEM_MAX_SIZE);
    vec![(GuestAddress(layout::DRAM_MEM_START), dram_size)]
}

/// Configures the system and should be called once per vm before starting vcpu threads.
/// For riscv64, we only setup the FDT.
///
/// # Arguments
///
/// * `guest_mem` - The memory to be used by the guest.
/// * `cmdline_cstring` - The kernel commandline.
/// * `device_info` - A hashmap containing the attached devices for building FDT device nodes.
/// * `aia_device` - The AIA device.
pub fn configure_system(
    guest_mem: &GuestMemoryMmap,
    cmdline_cstring: CString,
    device_info: &HashMap<(DeviceType, String), MMIODeviceInfo>,
    aia_device: &AIADevice,
) -> Result<(), ConfigurationError> {
    let fdt = fdt::create_fdt(guest_mem, cmdline_cstring, device_info, aia_device)?;
    let fdt_address = GuestAddress(get_fdt_addr(guest_mem));
    guest_mem
        .write_slice(fdt.as_slice(), fdt_address)
        .map_err(ConfigurationError::MemoryError)?;
    Ok(())
}

/// Returns the memory address where the kernel could be loaded.
pub fn get_kernel_start() -> u64 {
    layout::SYSTEM_MEM_START + layout::SYSTEM_MEM_SIZE
}
