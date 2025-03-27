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
