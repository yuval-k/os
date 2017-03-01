use volatile;
use io;
use device;


pub use device::spi::ClockPhase;
pub use device::spi::ClockPolarity;
pub use device::spi::Hz;

const SPI0_ADDR : ::mem::VirtualAddress = super::super::GPIO_BASE.uoffset(0x4000);

bitflags! {
    #[repr(C,packed)] pub flags ControlStatusFlags: u32 {
        const LEN_LONG   = 1 << 25,
        const DMA_LEN    = 1 << 24,
        const CSPOL2     = 1 << 23,
        const CSPOL1     = 1 << 22,
        const CSPOL0     = 1 << 21,
        const RXF        = 1 << 20,
        const RXR        = 1 << 19,
        const TXD        = 1 << 18,
        const RXD        = 1 << 17,
        const DONE       = 1 << 16,
        const TE_EN      = 1 << 15,
        const LMONO      = 1 << 14,
        const LEN        = 1 << 13,
        const REN        = 1 << 12,
        const ADCS       = 1 << 11,
        const INTR       = 1 << 10,
        const INTD       = 1 << 9,
        const DMAEN      = 1 << 8,
        const TA         = 1 << 7,
        const CSPOL      = 1 << 6,
        const CLEAR_RX   = 1 << 5,
        const CLEAR_TX   = 1 << 4,
        const CPOL       = 1 << 3,
        const CPHA       = 1 << 2,
        const RESERVED    = 0b11 << 0,
        const CS2         = 0b10 << 0,
        const CS1         = 0b01 << 0,
        const CS0         = 0b00 << 0,
    }
}

#[repr(C,packed)]
pub struct SPI  {
   pub control_status : volatile::Volatile<ControlStatusFlags>,
   pub fifo : volatile::Volatile<u32>,
   pub clock_div : volatile::Volatile<u32>,
   pub dlen : volatile::Volatile<u32>,
   pub ltoh : volatile::Volatile<u32>,
   pub dc : volatile::Volatile<u32>,
}
/*
fn pic_register(&mut self, inthandle : &IntrHandle<'a>) Attachment<'a> {

}
*/
/*
impl Driver for SPI {
    fn attach(&mut self, dh : DriverHandle){
        let intrnum : usize;
        self.attachment = pic.register(intrnum, dh);
        self.attachment = fsnode.register(dh);
    }
}
*/

impl SPI { 
    pub unsafe fn new() -> &'static mut Self {
        &mut *(SPI0_ADDR.0 as *mut SPI)
    }

    pub fn confiure(&mut self, clock_pol : ClockPolarity, clock_phase : ClockPhase, speed : Hz) -> Result<(),()>{
        let mut cs = ControlStatusFlags::empty();
        cs |= INTD | INTR;
        cs |= match clock_pol {
            ClockPolarity::ResetIsLow => ControlStatusFlags::empty(),
            ClockPolarity::ResetIsHigh => CPOL,
        };
        cs |= match clock_phase {
            ClockPhase::Middle => ControlStatusFlags::empty(),
            ClockPhase::Begin => CPHA,
        };

        const APB_CLOCK : u32 = 250_000_000;

        // only lower 16 bits count for clk
        let divider = (((APB_CLOCK / speed.0) >> 1) << 1) as u16;

        self.clock_div.write(divider as u32);
        self.control_status.write(cs);

        Ok(())
    }
}

impl io::WriteFifo for SPI {
    fn can_write(&self) -> bool {
        self.control_status.read().contains(TXD)
    }

    fn write_one(&mut self, b : u8) {
        self.fifo.write(b as u32);
    }
}

impl io::ReadFifo for SPI {

    fn can_read(&self) -> bool {
        self.control_status.read().contains(RXD)
    }

    fn read_one(&mut self) -> u8 {
        self.fifo.read() as u8
    }
}
