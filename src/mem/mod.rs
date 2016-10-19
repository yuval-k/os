use core::ops::Sub;

#[derive(Copy, Clone, Debug)]
pub enum MemorySize {
    Bytes(usize),
    KiloBytes(usize),
    MegaBytes(usize),
    GigaBytes(usize),
    PageSizes(usize),
}

pub fn toBytes(x : MemorySize) -> usize {
    match x {
        MemorySize::Bytes(b) => b,
        MemorySize::KiloBytes(k) => k << 10,
        MemorySize::MegaBytes(m) => m << 20,
        MemorySize::GigaBytes(g) => g << 30,
        MemorySize::PageSizes(p) => p << super::platform::PAGE_SHIFT,
    }
}

pub fn toPages(x : MemorySize) -> Result<usize, ()> {
    let b = toBytes(x);
    if (b & super::platform::PAGE_MASK) != 0 {
        Err(())
    } else {
        Ok(b >> super::platform::PAGE_SHIFT)
    }
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct VirtualAddress(pub usize);

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct PhysicalAddress(pub usize);

impl VirtualAddress {
    pub fn offset(&self, off : isize) -> VirtualAddress {
        VirtualAddress((self.0 as isize + off) as usize)
    }

    pub fn uoffset(&self, off : usize) -> VirtualAddress {
        VirtualAddress(self.0 + off)
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
    type Output = MemorySize;

    fn sub(self, _rhs: VirtualAddress) -> MemorySize {
        MemorySize::Bytes(self.0 - _rhs.0) 
    }
}

impl Sub for PhysicalAddress {
    type Output = MemorySize;

    fn sub(self, _rhs: PhysicalAddress) -> MemorySize {
        MemorySize::Bytes(self.0 - _rhs.0)
    }
}

pub trait FrameAllocator {
    fn allocate(&mut self, num_frames: usize) -> Option<PhysicalAddress>;
    fn deallocate(&mut self, start : PhysicalAddress, num_frames : usize);
}

pub trait MemoryMapper {
    fn map(       &mut self, fa : &mut FrameAllocator, p : PhysicalAddress, v : VirtualAddress, size : MemorySize) -> Result<(), ()>;
    fn unmap(     &mut self, fa : &mut FrameAllocator, v : VirtualAddress, size : MemorySize) -> Result<(), ()>;
    fn map_device(&mut self, fa : &mut FrameAllocator, p : PhysicalAddress, v : VirtualAddress, size : MemorySize) -> Result<(), ()>;

    fn v2p(&mut self, v : VirtualAddress)  -> Option<PhysicalAddress>;
    fn p2v(&mut self, v : PhysicalAddress) -> Option<VirtualAddress>;
}


pub trait MemoryManagaer {
    fn allocate(&mut self,   v : VirtualAddress, size : MemorySize) -> Result<(), ()>;
    fn deallocate(&mut self, v : VirtualAddress, size : MemorySize) -> Result<(), ()>;
}

struct DefaultMemoryManager<F : FrameAllocator, M : MemoryMapper> {
    memMapper : M,
    frameAlloc : F
}
