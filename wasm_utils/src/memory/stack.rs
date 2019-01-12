pub struct Top(u16)

#[derive(Copy, Clone, Default, Debug)]
pub struct WasmStack {
    top: WasmStackTop,
}

impl WasmStack {
    pub fn max(&self) -> u16 {
        U16_MAX
    }

    pub fn min(&self) -> u16 {
        0
    }

    pub fn allocation_is_valid(&self, allocation: WasmAllocation) -> bool {
        self.top() == allocation.offset() && u32::from(self.top()) + u32::from(allocation.length()) <= WasmStack::max()
    }

    pub fn deallocation_is_valid(&self, allocation: WasmAllocation) -> bool {
        self.top() == allocation.offset() + allocation.length() && allocation.offset() >= WasmStack::min()
    }

    // A stack can be initialized by giving the last know allocation on this stack
    pub fn new() -> WasmStack {
        WasmStack { top: WasmStack::min() }
    }

    pub fn allocate(&mut self, allocation: WasmAllocation) -> Result<WasmStackTop, ()> {
        if self.allocation_is_valid(allocation) {
            old_top = self.top;
            self.top = Top(allocation.offset() + allocation.length());
            Ok(old_top)
        }
        else {
            Err(())
        }
    }

    pub fn deallocate(&mut self, allocation: WasmAllocation) -> Result<(), ()> {
        if self.deallocation_is_valid(allocation) {
            self.top = Top(allocation.offset())
            Ok(())
        }
        else {
            Err(())
        }
    }

    // Getters
    pub fn top(self) -> u16 {
        self.top
    }
}

impl TryFrom<WasmAllocation> for WasmStack {
    type Error = ();
    fn try_from(allocation: WasmAllocation) -> Result<Self, Self::Error> {
        let stack = WasmStack{ top: allocation.offset() };
        stack.allocate(allocation)?;
        Ok(stack)
    }
}
