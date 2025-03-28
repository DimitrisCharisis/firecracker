use std::convert::Infallible;

use kvm_ioctls::Kvm as KvmFd;

use crate::cpu_config::templates::KvmCapability;

/// ['Kvm'] initialization can't fail for Riscv64
pub type KvmArchError = Infallible;

/// Struct with kvm fd and kvm associated parameters.
#[derive(Debug)]
pub struct Kvm {
    /// KVM fd.
    pub fd: KvmFd,
    /// Maximum number of memory slots allowed by KVM.
    pub max_memslots: usize,
    /// Additional capabilities that were specified in cpu template.
    pub kvm_cap_modifiers: Vec<KvmCapability>,
}

impl Kvm {
    pub(crate) const DEFAULT_CAPABILITIES: [u32; 5] = [
        kvm_bindings::KVM_CAP_IOEVENTFD,
        kvm_bindings::KVM_CAP_USER_MEMORY,
        kvm_bindings::KVM_CAP_DEVICE_CTRL,
        kvm_bindings::KVM_CAP_MP_STATE,
        kvm_bindings::KVM_CAP_ONE_REG,
    ];

    pub fn init_arch(
        fd: KvmFd,
        max_memslots: usize,
        _: Vec<KvmCapability>,
    ) -> Result<Self, KvmArchError> {
        let kvm_cap_modifiers = vec![];
        Ok(Self {
            fd,
            max_memslots,
            kvm_cap_modifiers,
        })
    }
}
