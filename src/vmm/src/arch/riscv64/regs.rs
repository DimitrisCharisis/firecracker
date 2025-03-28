use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Storage for riscv64 registers with different sizes.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Riscv64RegisterVec {
    ids: Vec<u64>,
    data: Vec<u8>,
}

impl Serialize for Riscv64RegisterVec {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unimplemented!();
    }
}

impl<'de> Deserialize<'de> for Riscv64RegisterVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!();
    }
}

// #[repr(C)]
// #[derive(Debug, Default, Copy, Clone, PartialEq)]
// pub struct kvm_riscv_config {
//     pub isa: u64,
//     pub zicbom_block_size: u64,
//     pub mvendorid: u64,
//     pub marchid: u64,
//     pub mimpid: u64,
//     pub zicboz_block_size: u64,
//     pub satp_mode: u64,
// }

// This macro gets the offset of a structure (i.e `str`) member (i.e `field`) without having
// an instance of that structure.
#[macro_export]
macro_rules! _offset_of {
    ($str:ty, $field:ident) => {{
        let tmp: std::mem::MaybeUninit<$str> = std::mem::MaybeUninit::uninit();
        let base = tmp.as_ptr();

        // Avoid warnings when nesting `unsafe` blocks.
        #[allow(unused_unsafe)]
        // SAFETY: The pointer is valid and aligned, just not initialised. Using `addr_of` ensures
        // that we don't actually read from `base` (which would be UB) nor create an intermediate
        // reference.
        let member = unsafe { core::ptr::addr_of!((*base).$field) } as *const u8;

        // Avoid warnings when nesting `unsafe` blocks.
        #[allow(unused_unsafe)]
        // SAFETY: The two pointers are within the same allocated object `tmp`. All requirements
        // from offset_from are upheld.
        unsafe {
            member.offset_from(base as *const u8) as usize
        }
    }};
}

#[macro_export]
macro_rules! offset_of {
    ($reg_struct:ty, $field:ident) => {
        $crate::_offset_of!($reg_struct, $field)
    };
    ($outer_reg_struct:ty, $outer_field:ident, $($inner_reg_struct:ty, $inner_field:ident), +) => {
        $crate::_offset_of!($outer_reg_struct, $outer_field) + offset_of!($($inner_reg_struct, $inner_field), +)
    };
}
pub(crate) use offset_of;

// Get the ID of a register
#[macro_export]
macro_rules! riscv64_reg_id {
    ($reg_type: tt, $offset: tt) => {
        // The core registers of an riscv64 machine are represented
        // in kernel by the `kvm_riscv_core` structure:
        //
        // struct kvm_riscv_core {
        //     struct user_regs_struct regs;
        //     unsigned long mode;
        // };
        //
        // struct user_regs_struct {
        //     unsigned long pc;
        //     unsigned long ra;
        //     unsigned long sp;
        //     unsigned long gp;
        //     unsigned long tp;
        //     unsigned long t0;
        //     unsigned long t1;
        //     unsigned long t2;
        //     unsigned long s0;
        //     unsigned long s1;
        //     unsigned long a0;
        //     unsigned long a1;
        //     unsigned long a2;
        //     unsigned long a3;
        //     unsigned long a4;
        //     unsigned long a5;
        //     unsigned long a6;
        //     unsigned long a7;
        //     unsigned long s2;
        //     unsigned long s3;
        //     unsigned long s4;
        //     unsigned long s5;
        //     unsigned long s6;
        //     unsigned long s7;
        //     unsigned long s8;
        //     unsigned long s9;
        //     unsigned long s10;
        //     unsigned long s11;
        //     unsigned long t3;
        //     unsigned long t4;
        //     unsigned long t5;
        //     unsigned long t6;
        // };
        // The id of a core register can be obtained like this: offset = id &
        // ~(KVM_REG_ARCH_MASK | KVM_REG_SIZE_MASK | KVM_REG_RISCV_CORE). Thus,
        // id = KVM_REG_RISCV | KVM_REG_SIZE_U64 | KVM_REG_RISCV_CORE | offset
        //
        // To generalize, the id of a register can be obtained by:
        // id = KVM_REG_RISCV | KVM_REG_SIZE_U64 |
        //      KVM_REG_RISCV_CORE/KVM_REG_RISCV_CONFIG/KVM_REG_RISCV_TIMER |
        //      offset
        KVM_REG_RISCV as u64
            | $reg_type as u64
            | KVM_REG_SIZE_U64 as u64
            | ($offset as u64 / std::mem::size_of::<u64>() as u64)
    };
}
pub(crate) use riscv64_reg_id;

#[macro_export]
macro_rules! riscv64_reg_core_id {
    ($offset: tt) => {
        riscv64_reg_id!(KVM_REG_RISCV_CORE, $offset)
    };
}
pub(crate) use riscv64_reg_core_id;

#[macro_export]
macro_rules! riscv64_reg_config_id {
    ($offset: tt) => {
        riscv64_reg_id!(KVM_REG_RISCV_CONFIG, $offset)
    };
}
pub(crate) use riscv64_reg_config_id;


#[macro_export]
macro_rules! riscv64_reg_timer_id {
    ($offset: tt) => {
        riscv64_reg_id!(KVM_REG_RISCV_TIMER, $offset)
    };
}
pub(crate) use riscv64_reg_timer_id;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isa_offset() {
        let off = offset_of!(kvm_riscv_config, isa);
        assert_eq!(off, 0);
    }

    #[test]
    fn test_zicbom_block_size_offsset() {
        let off = offset_of!(kvm_riscv_config, zicbom_block_size);
        assert_eq!(off, 8);
    }

    #[test]
    fn test_mvendorid_offset() {
        let off = offset_of!(kvm_riscv_config, mvendorid);
        assert_eq!(off, 16);
    }

    #[test]
    fn test_marchid_offset() {
        let off = offset_of!(kvm_riscv_config, marchid);
        assert_eq!(off, 24);
    }

    #[test]
    fn test_mimpid_offset() {
        let off = offset_of!(kvm_riscv_config, mimpid);
        assert_eq!(off, 32);
    }

    #[test]
    fn test_zicboz_block_size_offset() {
        let off = offset_of!(kvm_riscv_config, zicboz_block_size);
        assert_eq!(off, 40);
    }

    #[test]
    fn test_satp_mode_offset() {
        let off = offset_of!(kvm_riscv_config, satp_mode);
        assert_eq!(off, 48);
    }

    #[test]
    fn test_isa_id() {
        let off = offset_of!(kvm_riscv_config, isa);
        let id = riscv64_reg_config_id!(off);
        assert_eq!(id, 0x8030000001000000);
    }

    // struct kvm_riscv_core {
    //     struct user_regs_struct regs;
    //     unsigned long mode;
    // };
    //
    // struct user_regs_struct {
    //      [...]
    // }

    #[test]
    fn test_pc_id() {
        let off = offset_of!(kvm_riscv_core, regs, user_regs_struct, pc);
        let id = riscv64_reg_core_id!(off);
        assert_eq!(id, 0x8030000002000000);
    }

    #[test]
    fn test_a0_id() {
        let off = offset_of!(kvm_riscv_core, regs, user_regs_struct, a0);
        let id = riscv64_reg_core_id!(off);
        assert_eq!(id, 0x803000000200000a);
    }

    #[test]
    fn test_a1_id() {
        let off = offset_of!(kvm_riscv_core, regs, user_regs_struct, a1);
        let id = riscv64_reg_core_id!(off);
        assert_eq!(id, 0x803000000200000b);
    }
}
