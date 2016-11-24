use core::ops::Sub;
use alloc::rc::Rc;
use collections::boxed::Box;

#[derive(Copy, Clone, Debug)]
pub enum MemorySize {
    Bytes(usize),
    KiloBytes(usize),
    MegaBytes(usize),
    GigaBytes(usize),
    PageSizes(usize),
}

pub fn to_bytes(x: MemorySize) -> usize {
    match x {
        MemorySize::Bytes(b) => b,
        MemorySize::KiloBytes(k) => k << 10,
        MemorySize::MegaBytes(m) => m << 20,
        MemorySize::GigaBytes(g) => g << 30,
        MemorySize::PageSizes(p) => p << super::platform::PAGE_SHIFT,
    }
}

pub fn to_pages(x: MemorySize) -> Result<usize, ()> {
    let b = to_bytes(x);
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
    pub fn offset(&self, off: isize) -> VirtualAddress {
        VirtualAddress((self.0 as isize + off) as usize)
    }

    pub fn uoffset(&self, off: usize) -> VirtualAddress {
        VirtualAddress(self.0 + off)
    }
}

impl PhysicalAddress {
    pub fn offset(&self, off: isize) -> PhysicalAddress {
        PhysicalAddress((self.0 as isize + off) as usize)
    }
    pub fn uoffset(&self, off: usize) -> PhysicalAddress {
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
    fn allocate(&self, num_frames: usize) -> Option<PhysicalAddress>;
    fn deallocate(&self, start: PhysicalAddress, num_frames: usize);
}

pub trait PVMapper {
    fn v2p(&self, v: VirtualAddress) -> Option<PhysicalAddress>;
    fn p2v(&self, p: PhysicalAddress) -> Option<VirtualAddress>;
}

pub trait MemoryMapper : PVMapper{
    fn map(&self,
           fa: &FrameAllocator,
           p: PhysicalAddress,
           v: VirtualAddress,
           size: MemorySize)
           -> Result<(), ()>;
    fn unmap(&self,
             fa: &FrameAllocator,
             v: VirtualAddress,
             size: MemorySize)
             -> Result<(), ()>;
    fn map_device(&self,
                  fa: &FrameAllocator,
                  p: PhysicalAddress,
                  v: VirtualAddress,
                  size: MemorySize)
                  -> Result<(), ()>;

}


pub trait MemoryManagaer : PVMapper {
    fn map(&self,
           p: PhysicalAddress,
           v: VirtualAddress,
           size: MemorySize)
           -> Result<(), ()>;
    fn unmap(&self,
             v: VirtualAddress,
             size: MemorySize)
             -> Result<(), ()>;
    fn map_device(&self,
                  p: PhysicalAddress,
                  v: VirtualAddress,
                  size: MemorySize)
                  -> Result<(), ()>;
    }


pub struct DefaultMemoryManagaer {
    frame_allocator : Rc<FrameAllocator>,
    mem_mapper : Box<MemoryMapper>
}

impl DefaultMemoryManagaer {

    pub fn new(m: Box<MemoryMapper>, fa : Rc<FrameAllocator>) -> Self {
        DefaultMemoryManagaer{
            frame_allocator : fa,
            mem_mapper: m
        }
    }

}
impl MemoryManagaer for DefaultMemoryManagaer {

    fn map(&self,
           p: PhysicalAddress,
           v: VirtualAddress,
           size: MemorySize)
           -> Result<(), ()> {
        self.mem_mapper.map(self.frame_allocator.as_ref(), p, v, size)
    }

    fn unmap(&self,
             v: VirtualAddress,
             size: MemorySize)
             -> Result<(), ()> {
        // TODO change to result physical address
        self.mem_mapper.unmap(self.frame_allocator.as_ref(), v, size)
    }

    fn map_device(&self,
                  p: PhysicalAddress,
                  v: VirtualAddress,
                  size: MemorySize)
                  -> Result<(), ()> {
        self.mem_mapper.map_device(self.frame_allocator.as_ref(), p, v, size)
    }
}

impl PVMapper for DefaultMemoryManagaer {
    fn v2p(&self, v: VirtualAddress) -> Option<PhysicalAddress> {
        self.mem_mapper.v2p(v)
    }
    fn p2v(&self, p: PhysicalAddress) -> Option<VirtualAddress> {
        self.mem_mapper.p2v(p)
    }

}