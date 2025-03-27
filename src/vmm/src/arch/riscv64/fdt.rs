use std::collections::HashMap;
use std::ffi::CString;

use vm_fdt::{Error as VmFdtError, FdtWriter, FdtWriterNode};
use vm_memory::GuestMemoryError;

use super::super::DeviceType;
use super::aia::AIADevice;
use crate::device_manager::mmio::MMIODeviceInfo;
use crate::vstate::memory::{Address, GuestMemory, GuestMemoryMmap};

const ADDRESS_CELLS: u32 = 0x2;
const SIZE_CELLS: u32 = 0x2;
const CPU_INTC_BASE_PHANDLE: u32 = 3;
const CPU_BASE_PHANDLE: u32 = 256 + CPU_INTC_BASE_PHANDLE;
const AIA_APLIC_PHANDLE: u32 = 1;
const AIA_IMSIC_PHANDLE: u32 = 2;
const S_MODE_EXT_IRQ: u32 = 9;
const IRQ_TYPE_LEVEL_HI: u32 = 4;
const IRQ_TYPE_EDGE_RISING: u32	= 0x00000001;

/// Errors thrown while configuring the Flattened Device Tree for riscv64.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum FdtError {
    /// Create FDT error: {0}
    CreateFdt(#[from] VmFdtError),
    /// Read cache info error: {0}
    ReadCacheInfo(String),
    /// Failure in writing FDT in memory.
    WriteFdtToMemory(#[from] GuestMemoryError),
    /// Get device attribute error
    GetDeviceAttr,
}

pub fn create_fdt(
    guest_mem: &GuestMemoryMmap,
    cmdline: CString,
    timer_freq: u32,
    device_info: &HashMap<(DeviceType, String), MMIODeviceInfo>,
    aia_device: &AIADevice,
) -> Result<Vec<u8>, FdtError> {
    let mut fdt_writer = FdtWriter::new()?;

    let vcpu_count = aia_device.vcpu_count();
    let root = fdt_writer.begin_node("")?;

    fdt_writer.property_string("compatible", "linux,dummy-virt")?;
    fdt_writer.property_u32("#address-cells", ADDRESS_CELLS)?;
    fdt_writer.property_u32("#size-cells", SIZE_CELLS)?;
    create_cpu_nodes(&mut fdt_writer, vcpu_count as u32, timer_freq)?;
    create_memory_node(&mut fdt_writer, guest_mem)?;
    create_chosen_node(&mut fdt_writer, cmdline)?;
    create_aia_node(&mut fdt_writer, aia_device)?;
    create_devices_node(&mut fdt_writer, device_info)?;

    fdt_writer.end_node(root)?;

    let fdt_final = fdt_writer.finish()?;

    Ok(fdt_final)
}

fn create_cpu_nodes(fdt: &mut FdtWriter, vcpu_count: u32, timer_freq: u32) -> Result<(), FdtError> {
    let cpus = fdt.begin_node("cpus")?;

    fdt.property_u32("#address-cells", 0x1)?;
    fdt.property_u32("#size-cells", 0x0)?;
    fdt.property_u32("timebase-frequency", timer_freq)?;

    for cpu_index in 0..vcpu_count {
        let cpu = fdt.begin_node(&format!("cpu@{:x}", cpu_index))?;

        // From CH
        fdt.property_string("device_type", "cpu")?;
        fdt.property_string("compatible", "riscv")?;
        fdt.property_string("mmu-type", "sv48")?;
        fdt.property_string("riscv,isa", "rv64imafdc_smaia_ssaia")?;
        fdt.property_string("status", "okay")?;
        fdt.property_u32("reg", cpu_index)?;
        fdt.property_u32("phandle", CPU_BASE_PHANDLE + cpu_index)?;

        // interrupt controller node
        let intc_node = fdt.begin_node("interrupt-controller")?;
        fdt.property_string("compatible", "riscv,cpu-intc")?;
        fdt.property_u32("#interrupt-cells", 1u32)?;
        fdt.property_null("interrupt-controller")?;
        fdt.property_u32("phandle", CPU_INTC_BASE_PHANDLE + cpu_index)?;
        fdt.end_node(intc_node)?;

        fdt.end_node(cpu)?;
    }

    fdt.end_node(cpus)?;

    Ok(())
}

fn create_memory_node(fdt: &mut FdtWriter, guest_mem: &GuestMemoryMmap) -> Result<(), FdtError> {
    let mem_size = guest_mem.last_addr().raw_value()
        - super::layout::DRAM_MEM_START
        - super::layout::SYSTEM_MEM_SIZE
        + 1;
    let mem_reg_prop = &[
        super::layout::DRAM_MEM_START + super::layout::SYSTEM_MEM_SIZE,
        mem_size,
    ];
    let mem = fdt.begin_node("memory@ram")?;
    fdt.property_string("device_type", "memory")?;
    fdt.property_array_u64("reg", mem_reg_prop)?;
    fdt.end_node(mem)?;

    Ok(())
}

fn create_chosen_node(fdt: &mut FdtWriter, cmdline: CString) -> Result<(), FdtError> {
    let chosen = fdt.begin_node("chosen")?;

    let cmdline_string = cmdline
        .into_string()
        .map_err(|_| vm_fdt::Error::InvalidString)?;
    fdt.property_string("bootargs", cmdline_string.as_str())?;

    fdt.end_node(chosen)?;

    Ok(())
}

fn create_aia_node(fdt: &mut FdtWriter, aia: &AIADevice) -> Result<(), FdtError> {
    if aia.msi_compatible() {
        let imsic_name = format!("imsics@{:08x}", super::layout::IMSIC_START);
        let imsic_node = fdt.begin_node(&imsic_name)?;

        fdt.property_string("compatible", aia.imsic_compatibility())?;
        let imsic_reg_prop = aia.imsic_properties();
        fdt.property_array_u32("reg", &imsic_reg_prop)?;
        fdt.property_u32("#interrupt-cells", 0u32)?;
        fdt.property_null("interrupt-controller")?;
        fdt.property_null("msi-controller")?;

        let mut aia_nr_ids: u32 = 0;
        let mut nr_ids_attr = ::kvm_bindings::kvm_device_attr::default();
        nr_ids_attr.group = ::kvm_bindings::KVM_DEV_RISCV_AIA_GRP_CONFIG;
        nr_ids_attr.attr = ::kvm_bindings::KVM_DEV_RISCV_AIA_CONFIG_IDS as u64;
        nr_ids_attr.addr = &mut aia_nr_ids as *mut u32 as u64;

        aia.get_device_attribute(&mut nr_ids_attr)
            .map_err(|_| FdtError::GetDeviceAttr)?;

        fdt.property_u32("riscv,num-ids", aia_nr_ids)?;
        fdt.property_u32("phandle", AIA_IMSIC_PHANDLE)?;

        let mut irq_cells = vec![];
        let num_cpus = aia.vcpu_count() as u32;
        for i in 0..num_cpus {
            irq_cells.push(CPU_INTC_BASE_PHANDLE + i);
            irq_cells.push(S_MODE_EXT_IRQ);
        }
        fdt.property_array_u32("interrupts-extended", &irq_cells)?;

        fdt.end_node(imsic_node)?;
    }

    let aplic_name = format!("aplic@{:x}", super::layout::APLIC_START);
    let aplic_node = fdt.begin_node(&aplic_name)?;

    fdt.property_string("compatible", aia.aplic_compatibility())?;
    let reg_cells = aia.aplic_properties();
    fdt.property_array_u32("reg", &reg_cells)?;
    fdt.property_u32("#interrupt-cells", 2u32)?;
    fdt.property_null("interrupt-controller")?;

    // TODO num-sources should be equal to the IRQ allocated lines
    fdt.property_u32("riscv,num-sources", 96u32)?;
    fdt.property_u32("phandle", AIA_APLIC_PHANDLE)?;
    fdt.property_u32("msi-parent", AIA_IMSIC_PHANDLE)?;

    fdt.end_node(aplic_node)?;

    Ok(())
}

fn create_devices_node(
    fdt: &mut FdtWriter,
    devices_info: &HashMap<(DeviceType, String), MMIODeviceInfo>,
) -> Result<(), FdtError> {
    // Create one temp Vec to store all virtio devices
    let mut ordered_virtio_device: Vec<&MMIODeviceInfo> = Vec::new();

    for ((device_type, _device_id), info) in devices_info {
        match device_type {
            DeviceType::Serial => create_serial_node(fdt, info)?,
            DeviceType::Virtio(_) => {
                ordered_virtio_device.push(info);
            }
        }
    }

    // Sort out virtio devices by address from low to high and insert them into fdt table.
    ordered_virtio_device.sort_by_key(|a| a.addr);
    for ordered_device_info in ordered_virtio_device.drain(..) {
        create_virtio_node(fdt, ordered_device_info)?;
    }

    Ok(())
}

fn create_virtio_node(fdt: &mut FdtWriter, dev_info: &MMIODeviceInfo) -> Result<(), FdtError> {
    let virtio_mmio = fdt.begin_node(&format!("virtio_mmio@{:x}", dev_info.addr))?;
    let irq = [dev_info.irq.unwrap().into(), IRQ_TYPE_EDGE_RISING];

    fdt.property_string("compatible", "virtio,mmio")?;
    fdt.property_array_u64("reg", &[dev_info.addr, dev_info.len])?;
    fdt.property_array_u32("interrupts", &irq)?;
    fdt.property_u32("interrupt-parent", AIA_APLIC_PHANDLE)?;
    fdt.end_node(virtio_mmio)?;

    Ok(())
}

fn create_serial_node(fdt: &mut FdtWriter, dev_info: &MMIODeviceInfo) -> Result<(), FdtError> {
    let serial_reg_prop = [dev_info.addr, dev_info.len];
    let irq = [dev_info.irq.unwrap().into(), IRQ_TYPE_LEVEL_HI];

    let serial_name = format!("serial@{:x}", dev_info.addr);
    let serial_node = fdt.begin_node(&serial_name)?;
    fdt.property_string("compatible", "ns16550a")?;
    fdt.property_array_u64("reg", &serial_reg_prop)?;
    fdt.property_u32("clock-frequency", 3686400)?;
    fdt.property_u32("interrupt-parent", AIA_APLIC_PHANDLE)?;
    fdt.property_array_u32("interrupts", &irq)?;
    fdt.end_node(serial_node)?;

    Ok(())
}
