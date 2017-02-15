use device;
use io;
use super::super::super::pl011;
use collections::boxed::Box;

const SERIAL_BASE_VADDR: ::mem::VirtualAddress = super::GPIO_BASE.uoffset(0x1000);

pub struct Serial {
    pl: &'static mut pl011::PL011,
}

impl Serial {
    pub fn new() -> device::IoDevice {
        let s = unsafe {
            Serial{ pl : pl011::PL011::new(SERIAL_BASE_VADDR)}
        };
        device::IoDevice::new(Box::new(s))
    }

}

impl io::WriteFifo for Serial {
    fn can_write(&self) -> bool {self.pl.can_write()}

    fn write_one(&mut self, b : u8) {self.pl.write_one(b)}
}

impl io::ReadFifo for Serial {

    fn can_read(&self) -> bool {self.pl.can_read()}
    fn read_one(&mut self) -> u8 {self.pl.read_one()}

}
