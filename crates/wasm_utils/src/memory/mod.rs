use holochain_core_types::bits_n_pieces::U32_MAX;

pub mod allocation;
pub mod handler;
pub mod stack;

/// offsets, lengths, etc.
pub type MemoryInt = u32;

/// encodes allocations as 2x MemoryInt in high/low bits etc.
/// must be 2x larger than MemoryInt
pub type MemoryBits = u64;

/// represents the max MemoryInt in MemoryBits to facilitate gt comparisons
const MEMORY_INT_MAX: MemoryBits = U32_MAX;

/// reserve 4 bytes to fit a single top MemoryInt at start of stack
pub type Top = MemoryInt;
const RESERVED: MemoryInt = std::mem::size_of::<Top>() as MemoryInt;
