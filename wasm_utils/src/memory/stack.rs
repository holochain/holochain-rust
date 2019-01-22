use memory::{
    allocation::{AllocationError, Length, WasmAllocation},
    MemoryBits, MemoryInt, MEMORY_INT_MAX,
};
use std::convert::TryFrom;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
// pub in crate for testing
pub struct Top(pub(in crate::memory) MemoryInt);

impl From<Top> for MemoryInt {
    fn from(top: Top) -> Self {
        top.0
    }
}

impl From<Top> for usize {
    fn from(top: Top) -> Self {
        Self::from(MemoryInt::from(top))
    }
}

impl From<Top> for MemoryBits {
    fn from(top: Top) -> Self {
        top.0 as MemoryBits
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct WasmStack {
    // pub in crate for testing
    pub(in crate::memory) top: Top,
}

impl WasmStack {
    // represent the max as MemoryBits type to allow gt comparisons
    pub fn max() -> MemoryBits {
        MEMORY_INT_MAX
    }

    // min compares lt so can be a MemoryInt
    pub fn min() -> MemoryInt {
        0
    }

    // A stack can be initialized by giving the last know allocation on this stack
    pub fn new() -> WasmStack {
        WasmStack {
            top: Top(WasmStack::min()),
        }
    }

    pub fn next_allocation(&self, length: Length) -> Result<WasmAllocation, AllocationError> {
        WasmAllocation::new(MemoryInt::from(self.top()).into(), length)
    }

    pub fn allocate(&mut self, allocation: WasmAllocation) -> Result<Top, AllocationError> {
        if MemoryInt::from(self.top()) != MemoryInt::from(allocation.offset()) {
            Err(AllocationError::BadStackAlignment)
        } else if MemoryBits::from(self.top()) + MemoryBits::from(allocation.length())
            > WasmStack::max()
        {
            Err(AllocationError::OutOfBounds)
        } else {
            // @todo i don't know why we return the old top instead of new one?
            let old_top = self.top;
            self.top =
                Top(MemoryInt::from(allocation.offset()) + MemoryInt::from(allocation.length()));
            Ok(old_top)
        }
    }

    pub fn deallocate(&mut self, allocation: WasmAllocation) -> Result<Top, AllocationError> {
        if MemoryInt::from(self.top())
            != MemoryInt::from(allocation.offset()) + MemoryInt::from(allocation.length())
        {
            Err(AllocationError::BadStackAlignment)
        } else if MemoryInt::from(allocation.offset()) < WasmStack::min() {
            Err(AllocationError::OutOfBounds)
        } else {
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
        let mut stack = WasmStack {
            top: Top(allocation.offset().into()),
        };
        stack.allocate(allocation)?;
        Ok(stack)
    }
}

#[cfg(test)]
pub mod tests {

    use memory::{
        allocation::{AllocationError, Length, Offset, WasmAllocation},
        stack::{Top, WasmStack},
        MemoryBits, MemoryInt, MEMORY_INT_MAX,
    };
    use std::convert::TryFrom;

    pub fn fake_top() -> Top {
        Top(12345)
    }

    #[test]
    fn memory_int_from_top_test() {
        assert_eq!(12345 as MemoryInt, MemoryInt::from(fake_top()),);
    }

    #[test]
    fn usize_from_top_test() {
        assert_eq!(12345 as usize, usize::from(fake_top()),);
    }

    #[test]
    fn memory_bits_from_top_test() {
        assert_eq!(12345 as MemoryBits, MemoryBits::from(fake_top()),);
    }

    #[test]
    fn stack_max_test() {
        assert_eq!(MEMORY_INT_MAX, WasmStack::max(),);
    }

    #[test]
    fn stack_min_test() {
        assert_eq!(0, WasmStack::min(),);
    }

    #[test]
    fn stack_new_test() {
        assert_eq!(WasmStack { top: Top(0) }, WasmStack::new(),);
    }

    #[test]
    fn next_allocation_test() {
        let mut stack = WasmStack::new();
        let first_offset = Offset::from(0);
        let first_length = Length::from(5);
        let next_allocation = stack.next_allocation(first_length);
        assert_eq!(
            next_allocation,
            WasmAllocation::new(first_offset, first_length),
        );
        stack.allocate(next_allocation.unwrap()).ok();
        let second_offset = Offset::from(5);
        let second_length = Length::from(3);
        assert_eq!(
            stack.next_allocation(second_length),
            WasmAllocation::new(second_offset, second_length),
        );
    }

    #[test]
    fn allocate_test() {
        let mut stack = WasmStack::new();
        let unaligned_allocation = WasmAllocation::new(Offset::from(10), Length::from(10)).unwrap();
        assert_eq!(
            Err(AllocationError::BadStackAlignment),
            stack.allocate(unaligned_allocation),
        );

        let first_allocation = stack.next_allocation(Length::from(5));
        stack.allocate(first_allocation.unwrap()).ok();

        let second_allocation = stack.next_allocation(Length::from(8));
        assert_eq!(stack.allocate(second_allocation.unwrap()), Ok(Top(5)),);
        assert_eq!(stack.top(), Top(13),);

        let out_of_bounds_allocation = WasmAllocation {
            offset: Offset::from(13),
            length: Length::from(std::u16::MAX),
        };
        assert_eq!(
            Err(AllocationError::OutOfBounds),
            stack.allocate(out_of_bounds_allocation),
        );
    }

    #[test]
    fn deallocate_test() {
        let mut stack = WasmStack { top: Top(50) };
        let unaligned_allocation = WasmAllocation::new(Offset::from(50), Length::from(5)).unwrap();
        assert_eq!(
            Err(AllocationError::BadStackAlignment),
            stack.deallocate(unaligned_allocation),
        );

        // can't test out of bounds for deallocate because unsigned integers don't go below min

        let deallocation = WasmAllocation::new(Offset::from(20), Length::from(30)).unwrap();
        assert_eq!(stack.deallocate(deallocation), Ok(Top(50)),);
        assert_eq!(stack.top(), Top(20),);
    }

    #[test]
    fn top_test() {
        let top = Top(123);
        let stack = WasmStack { top };
        assert_eq!(top, stack.top(),);
    }

    #[test]
    fn try_stack_from_allocation_test() {
        // can't test bad alignment as it should not be possible

        assert_eq!(
            Err(AllocationError::OutOfBounds),
            WasmStack::try_from(WasmAllocation {
                offset: Offset::from(std::u16::MAX),
                length: Length::from(1)
            }),
        );

        assert_eq!(
            Ok(WasmStack { top: Top(60) }),
            WasmStack::try_from(WasmAllocation {
                offset: Offset::from(30),
                length: Length::from(30)
            }),
        );
    }

}
