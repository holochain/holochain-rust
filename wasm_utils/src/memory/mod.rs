use holochain_core_types::bits_n_pieces::U16_MAX;

pub mod allocation;
pub mod stack;

/// offsets, lengths, etc.
type MemoryInt = u16;

/// encodes allocations as 2x MemoryInt in high/low bits etc.
/// must be 2x larger than MemoryInt
type MemoryBits = u32;

/// represents the max MemoryInt in MemoryBits to facilitate gt comparisons
const MEMORY_INT_MAX: MemoryBits = U16_MAX;
