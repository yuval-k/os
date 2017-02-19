use device;
use io;
use super::super::super::pl011;
use collections::boxed::Box;
use super::gpio;

const SERIAL_BASE_VADDR: ::mem::VirtualAddress = super::GPIO_BASE.uoffset(0x1000);

pub struct Serial {
    pl: &'static mut pl011::PL011,
}

const GPIOTX : usize =14;
const GPIORX : usize =15;

impl Serial {
    pub fn new(gpio : &mut gpio::GPIO) -> Self {

        gpio.set_function(GPIOTX, gpio::FunctionSelect::Function0);
        gpio.set_function(GPIORX, gpio::FunctionSelect::Function0);

        let s = unsafe { // TODO: init gpio..
            Serial{ pl : pl011::PL011::new(SERIAL_BASE_VADDR)}
        };

        s
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
