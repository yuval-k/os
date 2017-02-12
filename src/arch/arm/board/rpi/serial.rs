use device::serial;
use super::super::super::cpu;
use super::super::super::pl011;

use io::WriteFifo;

pub const SERIAL_BASE_PADDR: ::mem::PhysicalAddress = ::mem::PhysicalAddress(super::MMIO_PSTART.0 +
                                                                             (0x0020_1000));
pub struct Writer {
    pl: &'static mut pl011::PL011,
}

impl Writer {
    pub fn new(base: ::mem::VirtualAddress) -> Self {
        unsafe {
        let pl = pl011::PL011::new(base);
        pl.init();
        Writer {
                pl:  pl
            }
        }
    }

}

impl serial::SerialMMIO for Writer {
    fn write_byte_async(&mut self, b: u8) {
        self.pl.write_one(b);
        // wait for the memory operation to finish.
        cpu::data_synchronization_barrier();
    }

    fn is_done(&self) -> bool {
        self.pl.can_write()
    }
}
