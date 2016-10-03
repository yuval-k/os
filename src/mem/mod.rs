use core::ops::Sub;

#[derive(Copy, Clone)]
pub struct VirtualAddress(pub usize);
#[derive(Copy, Clone)]
pub struct PhysicalAddress(pub usize);

impl VirtualAddress {
    pub fn offset(&self, off : isize) -> VirtualAddress {
        VirtualAddress((self.0 as isize + off) as usize)
    }
}

impl PhysicalAddress {
    pub fn offset(&self, off : isize) -> PhysicalAddress {
        PhysicalAddress((self.0 as isize + off) as usize)
    }
}

impl Sub for VirtualAddress {
    type Output = isize;

    fn sub(self, _rhs: VirtualAddress) -> isize {
        (self.0 - _rhs.0) as isize 
    }
}

impl Sub for PhysicalAddress {
    type Output = isize;

    fn sub(self, _rhs: PhysicalAddress) -> isize {
        (self.0 - _rhs.0) as isize 
    }
}


pub trait FrameAllocator {
    fn allocate(&mut self, number: usize) -> Option<PhysicalAddress>;
    fn deallocate(&mut self, start : PhysicalAddress, number : usize);
}

pub trait MemoryMapper {
    fn map(&mut self, p : ::mem::PhysicalAddress, v : ::mem::VirtualAddress, length : usize);
}
