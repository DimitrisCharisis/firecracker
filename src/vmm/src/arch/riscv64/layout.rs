//       1022GB +------------------------------------------------------+ DRAM_MEM_MAX_SIZE
//              |                         .                            |
//              |                         .                            |
//              |                         .                            |
//  128MB + 2GB +------------------------------------------------------+ ACTUAL VM RAM_SIZE
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//    2MB + 2GB +------------------------------------------------------+ RISCV_KERNEL_START
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//          2GB +------------------------------------------------------+ DRAM_MEM_START / FDT_START
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//          1GB +------------------------------------------------------+ MMIO_MEM_START
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//  128MB + 4KB +------------------------------------------------------+ IMSIC_START + IMSIC_SZ_PH
//              |                                                      |
//        128MB +------------------------------------------------------+ IMSIC_START
//              |                                                      |
//              |                                                      |
//              |                                                      |
//              |                                                      |
//            0 +------------------------------------------------------+ APLIC_START

/// Start of RAM on 64 bit RISCV.
pub const DRAM_MEM_START: u64 = 0x8000_0000; // 2 GB.
/// The maximum RAM size.
pub const DRAM_MEM_MAX_SIZE: usize = 0x00FF_8000_0000; // 1024 - 2 = 1022G.

/// Start of RAM on 64 bit RISCV.
pub const SYSTEM_MEM_START: u64 = DRAM_MEM_START;

pub const SYSTEM_MEM_SIZE: u64 = 0x20_0000;

/// Kernel command line maximum size.
/// As per `arch/riscv/include/uapi/asm/setup.h`.
pub const CMDLINE_MAX_SIZE: usize = 1024;

pub const FDT_START: u64 = DRAM_MEM_START;
pub const FDT_MAX_SIZE: usize = 0x1_0000;

//
// From the RISC-V Privlidged Spec v1.10:
//
// Global interrupt sources are assigned small unsigned integer identifiers,
// beginning at the value 1.  An interrupt ID of 0 is reserved to mean no
// interrupt.  Interrupt identifiers are also used to break ties when two or
// more interrupt sources have the same assigned priority. Smaller values of
// interrupt ID take precedence over larger values of interrupt ID.
//
// While the RISC-V supervisor spec doesn't define the maximum number of
// devices supported by the PLIC, the largest number supported by devices
// marked as 'riscv,plic0' (which is the only device type this driver supports,
// and is the only extant PLIC as of now) is 1024.  As mentioned above, device
// 0 is defined to be non-existant so this device really only supports 1023
// devices.
pub const IRQ_MAX: u32 = 1023;
/// First usable interrupt on riscv64.
pub const IRQ_BASE: u32 = 1;

/// Below this address will reside the AIA, above this address will reside the MMIO devices.
pub const MAPPED_IO_START: u64 = 1 << 30; // 1 GB

pub const IMSIC_START: u64 = 0x0800_0000;
// IMISC size per hart
pub const IMSIC_SZ_PH: u64 = 0x1000;

pub const APLIC_START: u64 = 0x00;
