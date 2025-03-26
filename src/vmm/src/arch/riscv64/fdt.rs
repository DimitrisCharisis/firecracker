use vm_fdt::{Error as VmFdtError, FdtWriter, FdtWriterNode};
use vm_memory::GuestMemoryError;

/// Errors thrown while configuring the Flattened Device Tree for riscv64.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum FdtError {
    /// Create FDT error: {0}
    CreateFdt(#[from] VmFdtError),
    /// Read cache info error: {0}
    ReadCacheInfo(String),
    /// Failure in writing FDT in memory.
    WriteFdtToMemory(#[from] GuestMemoryError),
}

