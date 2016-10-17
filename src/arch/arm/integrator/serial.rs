use ::device::serial;

use core::intrinsics::{volatile_load, volatile_store};

pub const SERIAL_BASE_PADDR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1600_0000);

pub struct Writer {
    base :  *mut u8,
}

const SERIAL_FLAG_REGISTER : usize = 0x18;
const SERIAL_BUFFER_FULL: u32  = (1 << 5);

impl Writer {
    pub fn new(base : ::mem::VirtualAddress) -> Self {
        Writer{
            base: base.0 as *mut u8,
        }
    }
    pub fn new_bare() -> Self {
        Writer{
            base: SERIAL_BASE_PADDR.0 as *mut u8,
        }
    }
}
impl serial::SerialMMIO for Writer {

    fn write_byte_async(&mut self, b :u8) {
        let ptr : *mut u8;
        ptr = self.base;
        unsafe {volatile_store(ptr, b);}
    }

    fn is_done(&self) -> bool {
        let ptr :  *const u32 = (self.base as usize +  SERIAL_FLAG_REGISTER) as *const u32 ;
        return (unsafe {volatile_load(ptr)} & SERIAL_BUFFER_FULL) == 0
    }

}