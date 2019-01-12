use memory::MemoryInt;
use memory::MemoryBits;
use memory::MEMORY_INT_MAX;
use memory::allocation::WasmAllocation;
use std::convert::TryFrom;

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
    fn max()-> MemoryBits {
        MEMORY_INT_MAX
    }

    // min compares lt so can be a MemoryInt
    pub fn min() -> MemoryInt {
        0
    }

    pub fn allocation_is_valid(&self, allocation: WasmAllocation) -> bool {
        // can't allocate anywhere other than top
        if MemoryInt::from(self.top()) != MemoryInt::from(allocation.offset()) {
            false
        }
        // can't allocate past max
        else if MemoryBits::from(self.top()) + MemoryBits::from(allocation.length()) > WasmStack::max() {
            false
        }
        else {
            true
        }
    }

    pub fn deallocation_is_valid(&self, allocation: WasmAllocation) -> bool {
        // can't deallocate anywhere other than top
        if MemoryInt::from(self.top()) == MemoryInt::from(allocation.offset()) + MemoryInt::from(allocation.length()) {
            false
        }
        // can't deallocate past min
        else if MemoryInt::from(allocation.offset()) < WasmStack::min() {
            false
        }
        else {
            true
        }
    }

    // A stack can be initialized by giving the last know allocation on this stack
    pub fn new() -> WasmStack {
        WasmStack { top: WasmStack::min().into() }
    }

    pub fn allocate(&mut self, allocation: WasmAllocation) -> Result<Top, ()> {
        if self.allocation_is_valid(allocation) {
            // @todo i don't know why we return the old top instead of new one?
            let old_top = self.top;
            self.top = Top(MemoryInt::from(allocation.offset()) + MemoryInt::from(allocation.length()));
            Ok(old_top)
        }
        else {
            Err(())
        }
    }

    pub fn deallocate(&mut self, allocation: WasmAllocation) -> Result<(), ()> {
        if self.deallocation_is_valid(allocation) {
            self.top = Top(allocation.offset().into());
            Ok(())
        }
        else {
            Err(())
        }
    }

    // Getters
    pub fn top(self) -> Top {
        self.top
    }
}

impl TryFrom<WasmAllocation> for WasmStack {
    type Error = ();
    fn try_from(allocation: WasmAllocation) -> Result<Self, Self::Error> {
        let mut stack = WasmStack{ top: Top(allocation.offset().into()) };
        stack.allocate(allocation)?;
        Ok(stack)
    }
}
