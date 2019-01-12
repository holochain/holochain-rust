use holochain_core_types::bits_n_pieces::U16_MAX;

pub mod allocation;

type MemoryBits = u32;
type MemoryInt = u16;
/// represents the max MemoryInt in MemoryBits to facilitate gt comparisons
const MemoryIntMax: MemoryBits = U16_MAX;
