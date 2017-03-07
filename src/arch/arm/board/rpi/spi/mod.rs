mod spi;

use platform;
use sync;
use device;

use io::Read;
use io::Write;

use arch::arm::Driver;
use arch::arm::DriverHandle;
use arch::arm::InterruptableDriver;

use device::spi::ClockPhase;
use device::spi::ClockPolarity;
use device::spi::Hz;
use device::spi::Transfer;

struct TransferState {
    transfer : device::spi::Transfer,
    bytes_read  : usize,
    bytes_written : usize,
}

impl TransferState {
    pub fn done(self) {
        self.transfer.done();
    }
}

pub struct SPIDev {
    dev_impl : sync::CpuMutex<SPIDevImpl>
}
struct SPIDevImpl {
    spi : &'static mut spi::SPI,
    cur_transfer : Option<TransferState>
}


impl SPIDev {

    pub fn new() -> Self {
    unsafe {    
        SPIDev{
           dev_impl : sync::CpuMutex::new(
                SPIDevImpl {
                    spi : spi::SPI::new(),
                    cur_transfer : None
                }
                )
        }
    }
    }
}

impl InterruptableDriver for SPIDev {}

impl Driver for SPIDev {
    fn attach(&mut self, dh : DriverHandle) {
        let interrupt_service = &platform::get_platform_services().arch_services.interrupt_service;
        interrupt_service.register_callback_on_intr(super::intr::Interrupts::SPI as usize, dh);

        // attach to fs node / spi node
    }
}


impl device::spi::SPIMaster for SPIDev {

    fn confiure(&self, c : device::spi::Configuration) -> Result<(),()>{
        let ig = platform::intr::no_interrupts();
        let mut spidev = self.dev_impl.lock();
        if spidev.cur_transfer.is_some() {
            return Err(())
        }
        spidev.spi.confiure(c)

    }

    fn start_transfer(&self, t : device::spi::Transfer) -> Result<(),()> {
        // enable TA and clear pipes.
        let mut cs_updates = spi::ControlStatusFlags::empty();

        cs_updates |= match t.slave {
            s @ 0 ... 2 => spi::ControlStatusFlags::from_bits_truncate(s as u32),
            _ => return Err(()),
        };

        cs_updates |= spi::TA | spi::CLEAR_TX | spi::CLEAR_RX;

        let ig = platform::intr::no_interrupts();
        let mut spidev = self.dev_impl.lock();
        if spidev.cur_transfer.is_some() {
            return Err(())
        }

        let t = TransferState {
            transfer : t,
            bytes_read  : 0,
            bytes_written : 0,
        };
       
        spidev.cur_transfer = Some(t);
        platform::memory_write_barrier();

        // TODO : add chip select at  some point

        spidev.spi.control_status.update(|cs|{
            cs.remove(spi::CS1);
            cs.remove(spi::CS2);
            *cs |= cs_updates;
        });

        Ok(())
    }

}


// see section 10.6.2 in https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2835/BCM2835-ARM-Peripherals.pdf
/*
10.6.2 Interrupt 
e)
Set INTR and INTD. These can be left set over multip
le operations. 
f)
Set CS, CPOL, CPHA as required and set TA = 1. This wil
l immediately trigger a 
first interrupt with DONE == 1. 
g)
On interrupt: 
h)
If DONE is set and data to write (this means it is th
e first interrupt), write up to 16 
bytes to SPI_FIFO. If DONE is set and no more data, set 
TA = 0. Read trailing data 
from SPI_FIFO until RXD is 0. 
i)
If RXR is set read 12 bytes data from SPI_FIFO and if mor
e data to write, write up to 
12 bytes to SPIFIFO. 
*/
impl platform::Interruptable for SPIDev {
    fn interrupted(&self) {

        let mut spidev = self.dev_impl.lock();

        if spidev.cur_transfer.is_none() {
            panic!("SPI transfer is none during interrupt!")
        }
        
        let mut transfer = spidev.cur_transfer.take().unwrap();

        {
            let mut spidevspi = &mut spidev.spi;

            let flags = spidevspi.control_status.read();
            if flags.contains(spi::DONE) {
                if transfer.bytes_written < transfer.transfer.buf.len() {
                    if let Ok(written) = spidevspi.write(&transfer.transfer.buf[transfer.bytes_written..]) {
                        transfer.bytes_written += written;
                    }
                } else {
                    spidevspi.control_status.update(|cs| {
                        cs.remove(spi::TA);
                    });
                    if let Ok(read) =  spidevspi.read(&mut transfer.transfer.buf[transfer.bytes_read..]) {
                        transfer.bytes_read += read;
                    }
                    platform::memory_write_barrier();  // <- probably not needed for non SMP
                    // we are done, no need to place the transfer back.
                    transfer.done();
                    return;
                }

            }
            // read flags again
            let flags = spidevspi.control_status.read();
            if flags.contains(spi::RXR) {
                    if let Ok(read) =  spidevspi.read(&mut transfer.transfer.buf[transfer.bytes_read..]) {
                        transfer.bytes_read += read;
                    }
                    if let Ok(written) = spidevspi.write(&transfer.transfer.buf[transfer.bytes_written..]) {
                        transfer.bytes_written += written;
                    }
            }
        }

        spidev.cur_transfer = Some(transfer);
    }
}


impl ::arch::arm::SPIDriver for SPIDev {}
