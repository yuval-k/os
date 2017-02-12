mod spi;

use platform;
use sync;
use core::mem;
use collections::vec::Vec;
use collections::boxed::Box;
use alloc::boxed::FnBox;


use io::WriteFifo;
use io::ReadFifo;

struct Transfer {
    buf : Vec<u8>,
    slave : usize,
    bytes_read  : usize,
    bytes_written : usize,
    callback : Option<Box<FnBox(Vec<u8>)>>,
}

impl Transfer {
    fn new<F>(buf : Vec<u8>, slave : usize, callback : F) -> Self 
    where F: FnOnce(Vec<u8>),
          F: Send + 'static {
        Transfer {
            buf           : buf,
            slave         : slave,
            bytes_read    : 0,
            bytes_written : 0,
            callback      : Some(Box::new(callback))
        }
    }
}

impl Drop for Transfer {
    fn drop(&mut self) {
        let buf =  mem::replace(&mut self.buf, vec![]);
        (self.callback.take().unwrap())(buf);
    }
}


struct SPIDev {
    dev_impl : sync::CpuMutex<SPIDevImpl>
}
struct SPIDevImpl {
    spi : &'static mut spi::SPI,
    cur_transfer : Option<Transfer>
}

impl SPIDev {
    unsafe fn new() -> Self {
        
        SPIDev{
           dev_impl : sync::CpuMutex::new(
                SPIDevImpl {
                    spi : spi::SPI::new(),
                    cur_transfer : None
                }
                )
        }
    }

    fn start_transfer(&mut self, t : Transfer) -> Result<(),()> {
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
       
        spidev.cur_transfer = Some(t);
        platform::memory_write_barrier();

        spidev.spi.control_status.update(|cs|{
            cs.remove(spi::CS0);
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
    fn interrupted(&self, c : &mut platform::Context) {

        let mut spidev = self.dev_impl.lock();

        if spidev.cur_transfer.is_none() {
            panic!("SPI transfer is none during interrupt!")
        }
        
        let mut transfer = spidev.cur_transfer.take().unwrap();

        {
            let mut spidevspi = &mut spidev.spi;

            let flags = spidevspi.control_status.read();
            if flags.contains(spi::DONE) {
                if transfer.bytes_written < transfer.buf.len() {
                    if let Ok(written) = spidevspi.write(&transfer.buf[transfer.bytes_written..]) {
                        transfer.bytes_written += written;
                    }
                } else {
                    spidevspi.control_status.update(|cs| {
                        cs.remove(spi::TA);
                    });
                    if let Ok(read) =  spidevspi.read(&mut transfer.buf[transfer.bytes_read..]) {
                        transfer.bytes_read += read;
                    }
                    // we are done, no need to place the transfer back.
                    return;
                }

            }
            if flags.contains(spi::RXR) {
                    if let Ok(read) =  spidevspi.read(&mut transfer.buf[transfer.bytes_read..]) {
                        transfer.bytes_read += read;
                    }
                    if let Ok(written) = spidevspi.write(&transfer.buf[transfer.bytes_written..]) {
                        transfer.bytes_written += written;
                    }
            }
        }

        spidev.cur_transfer = Some(transfer);

    }
}