use ::device::serial;

use core::intrinsics::{volatile_load, volatile_store};

pub const SERIAL_BASE_PADDR: ::mem::PhysicalAddress = ::mem::PhysicalAddress(super::MMIO_PSTART.0 + (0x0020_1000));
pub const DATA_REG_OFFSET : usize = 0;
pub const FLAG_REG_OFFSET : usize = 0x18;
pub const UARTFR_TXFE : u32 = 1 << 7;

pub struct Writer {
    base: *mut u8,
}

impl Writer {
    pub fn new(base: ::mem::VirtualAddress) -> Self {
        Writer { base: base.0 as *mut u8 }
    }
    pub fn new_bare() -> Self {
        Writer { base: SERIAL_BASE_PADDR.0 as *mut u8 }
    }
}
impl serial::SerialMMIO for Writer {
    fn write_byte_async(&mut self, b: u8) {
        let ptr: *mut u8;
        ptr = (self.base as usize + DATA_REG_OFFSET) as *mut u8;
        unsafe {
            volatile_store(ptr, b);
        }
    }

    fn is_done(&self) -> bool {
        let ptr: *const u32 = (self.base as usize + FLAG_REG_OFFSET) as *const u32;
        return (unsafe { volatile_load(ptr) } & UARTFR_TXFE) != 0;
    }
}
