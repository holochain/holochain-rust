use memory::MemoryInt;
use memory::MemoryBits;
use memory::MEMORY_INT_MAX;
use memory::allocation::WasmAllocation;
use std::convert::TryFrom;
use memory::allocation::AllocationError;
use memory::allocation::Length;

#[derive(Copy, Clone, Default, Debug)]
pub struct Top(MemoryInt);

impl From<Top> for MemoryInt {
    fn from(top: Top) -> Self {
        top.0
    }
}

impl From<Top> for MemoryBits {
    fn from(top: Top) -> Self {
        top.0 as MemoryBits
    }
}

impl From<MemoryInt> for Top {
    fn from(i: MemoryInt) -> Self {
        Top(i)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct WasmStack {
    top: Top,
}

impl WasmStack {
    // represent the max as MemoryBits type to allow gt comparisons
    pub fn max()-> MemoryBits {
        MEMORY_INT_MAX
    }

    // min compares lt so can be a MemoryInt
    pub fn min() -> MemoryInt {
        0
    }

    // A stack can be initialized by giving the last know allocation on this stack
    pub fn new() -> WasmStack {
        WasmStack { top: WasmStack::min().into() }
    }

    pub fn next_allocation(&self, length: Length) -> Result<WasmAllocation, AllocationError> {
        WasmAllocation::new(MemoryInt::from(self.top()).into(), length)
    }

    pub fn allocate(&mut self, allocation: WasmAllocation) -> Result<Top, AllocationError> {
        if MemoryInt::from(self.top()) != MemoryInt::from(allocation.offset()) {
            Err(AllocationError::BadStackAlignment)
        }
        else if MemoryBits::from(self.top()) + MemoryBits::from(allocation.length()) > WasmStack::max() {
            Err(AllocationError::OutOfBounds)
        }
        else {
            // @todo i don't know why we return the old top instead of new one?
            let old_top = self.top;
            self.top = Top(MemoryInt::from(allocation.offset()) + MemoryInt::from(allocation.length()));
            Ok(old_top)
        }
    }

    pub fn deallocate(&mut self, allocation: WasmAllocation) -> Result<Top, AllocationError> {
        if MemoryInt::from(self.top()) != MemoryInt::from(allocation.offset()) + MemoryInt::from(allocation.length()) {
            Err(AllocationError::BadStackAlignment)
        }
        else if MemoryInt::from(allocation.offset()) < WasmStack::min() {
            Err(AllocationError::OutOfBounds)
        }
        else {
            let old_top = self.top;
            self.top = Top(allocation.offset().into());
            Ok(old_top)
        }
    }

    // Getters
    pub fn top(self) -> Top {
        self.top
    }
}

impl TryFrom<WasmAllocation> for WasmStack {
    type Error = AllocationError;
    fn try_from(allocation: WasmAllocation) -> Result<Self, Self::Error> {
        let mut stack = WasmStack{ top: Top(allocation.offset().into()) };
        stack.allocate(allocation)?;
        Ok(stack)
    }
}
