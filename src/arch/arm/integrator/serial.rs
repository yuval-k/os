use ::device::serial;

use core::intrinsics::{volatile_load, volatile_store};

pub struct Writer {
}

const SERIAL_BASE: *mut () = 0x16000000 as *mut ();
const SERIAL_FLAG_REGISTER : isize = 0x18;
const SERIAL_BUFFER_FULL: u32  = (1 << 5);

impl Writer {
    pub fn new() -> Self {
        Writer{
        }
    }
}
impl serial::SerialMMIO for Writer {

    fn write_byte_async(&mut self, b :u8) {
        let ptr : *mut u8;
        ptr = SERIAL_BASE as *mut u8;
        unsafe {volatile_store(ptr, b);}
    }

    fn is_done(&self) -> bool {
        let ptr :  *const u32;
        ptr = unsafe{SERIAL_BASE.offset(SERIAL_FLAG_REGISTER) as *const u32};
        return (unsafe {volatile_load(ptr)} & SERIAL_BUFFER_FULL) == 0
    }

}