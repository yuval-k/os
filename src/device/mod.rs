pub mod serial;
pub mod spi;

use collections::vec::Vec;
use core::cmp;
use io;
use sync;
use platform;
use core::ops::DerefMut;
use collections::boxed::Box;

pub trait Device {
    fn new() -> Self;
    fn attach(&mut self);
}


trait ReadWriteFifo:  io::WriteFifo + io::ReadFifo + io::Read + io::Write {}
impl<T> ReadWriteFifo for T where T: io::WriteFifo + io::ReadFifo {}

pub struct IoDevice {
    io_dev : sync::CpuMutex<IoDeviceInner>
}

pub struct IoDeviceInner {
    dev : Box<ReadWriteFifo>,
    rx_buffer : Vec<u8>,
    tx_buffer : Vec<u8>,
}


impl IoDevice {

        pub fn new(device : Box<ReadWriteFifo>) -> Self {
            IoDevice {
                io_dev : sync::CpuMutex::new(
                    IoDeviceInner{
                    dev       : device,
                    rx_buffer : vec![],
                    tx_buffer : vec![],
            }
            )
            }
        }

        pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let ig  = platform::intr::no_interrupts();
            self.try_read();

            let mut dev = self.io_dev.lock();
            
            let rx_len = dev.rx_buffer.len();
            let amt = cmp::min(buf.len(), rx_len);
            buf[..amt].copy_from_slice(&dev.rx_buffer);

            dev.rx_buffer.reverse();
            dev.rx_buffer.truncate(rx_len - amt);
            dev.rx_buffer.reverse();


            return Ok(amt)

        }

    fn try_write(&self) {
        let mut iodev_m = self.io_dev.lock();
        let mut iodev = iodev_m.deref_mut(); 
        let res = iodev.dev.write(&mut iodev.tx_buffer);
        if let Ok(written) =  res {
            let txbuflen = iodev.tx_buffer.len();
            iodev.tx_buffer.reverse();
            iodev.tx_buffer.truncate(txbuflen - written);
            iodev.tx_buffer.reverse();
            
        }
    }
    fn try_read(&self) {
        let mut buff = [0u8; 10];
        
        let mut iodev_m = self.io_dev.lock();
        let mut iodev = iodev_m.deref_mut(); 
        loop {
            match iodev.dev.read(&mut buff) {
                Ok(written) => iodev.rx_buffer.extend(&buff[0..written]),
                Err(_) => break,
            }
        }

    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ig  = platform::intr::no_interrupts();
        {
            let mut dev = self.io_dev.lock();
            dev.tx_buffer.extend(buf);
        
        }
        self.try_write();

        Ok(buf.len())
    }

}

impl platform::Interruptable for IoDevice {
    fn interrupted(&self) {
        self.try_write();
        self.try_read();
    }
}

/*
impl<T : io::WriteFifo + io::ReadFifo + platform::Interruptable > platform::Interruptable for IoDevice<T> {
    fn interrupted(&self, c : &mut platform::Context) {
        let mut iodev_m = self.io_dev.lock();
        iodev_m.interrupted(c)
    }
}
*/
