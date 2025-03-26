use std::convert::Infallible;

use kvm_ioctls::Kvm as KvmFd;

/// ['Kvm'] initialization can't fail for Riscv64
pub type KvmArchError = Infallible;

/// Struct with kvm fd and kvm associated parameters.
#[derive(Debug)]
pub struct Kvm {
    /// KVM fd.
    pub fd: KvmFd,
    /// Maximum number of memory slots allowed by KVM.
    pub max_memslots: usize,
}

impl Kvm {
    pub fn init_arch(fd: KvmFd, max_memslots: usize) -> Result<Self, KvmArchError> {
        Ok(Self { fd, max_memslots })
    }
}
