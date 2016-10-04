use core::ops::Sub;
use core::cmp;

#[derive(Copy, Clone)]
pub struct VirtualAddress(pub usize);
#[derive(Copy, Clone)]
pub struct PhysicalAddress(pub usize);

impl VirtualAddress {
    pub fn offset(&self, off : isize) -> VirtualAddress {
        VirtualAddress((self.0 as isize + off) as usize)
    }

    pub fn uoffset(&self, off : usize) -> VirtualAddress {
        VirtualAddress(self.0 + off)
    }
}

impl cmp::PartialOrd for PhysicalAddress {
    fn partial_cmp(&self, other: &PhysicalAddress) -> Option<cmp::Ordering> {
                Some(self.cmp(other))
    }
}

impl cmp::PartialEq for PhysicalAddress {
    fn eq(&self, other: &PhysicalAddress) -> bool {
        self.0 == other.0
    }
}

impl cmp::Eq for PhysicalAddress {}

impl cmp::Ord for PhysicalAddress {
    fn cmp(&self, other: &PhysicalAddress) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PhysicalAddress {
    pub fn offset(&self, off : isize) -> PhysicalAddress {
        PhysicalAddress((self.0 as isize + off) as usize)
    }
    pub fn uoffset(&self, off : usize) -> PhysicalAddress {
        PhysicalAddress(self.0 + off)
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
    fn map(&mut self, fa : &mut FrameAllocator, p : ::mem::PhysicalAddress, v : ::mem::VirtualAddress, length : usize);
}
