use alloc::rc::Rc;
use collections::vec::Vec;
use alloc::boxed::FnBox;
use alloc::boxed::Box;
use core::mem;

pub enum ClockPolarity {
    ResetIsLow,
    ResetIsHigh,
}

pub enum ClockPhase {
    Middle,
    Begin,
}

pub struct Hz(pub u32);

pub struct Transfer {
    pub buf : Vec<u8>,
    pub slave : usize,
    callback : Option<Box<FnBox(Vec<u8>)>>,
}

impl Transfer {
    pub fn new<F>(buf : Vec<u8>, slave : usize, callback : F) -> Self 
    where F: FnOnce(Vec<u8>),
          F: Send + 'static {
        Transfer {
            buf           : buf,
            slave         : slave,
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

pub trait SPIMaster {

    fn confiure(&mut self, clock_pol : ClockPolarity, clock_phase : ClockPhase, speed : Hz) -> Result<(),()>;
    fn start_transfer(&mut self, t : Transfer) -> Result<(),()>;
}

pub fn get_spi_master() -> Option<&'static SPIMaster> {
    unsafe{
        match SPI_MASTER {
            None => None,
            Some(ref spi) => Some(spi.as_ref())
        }
    }
}

pub unsafe fn set_spi_master( spi : Rc<SPIMaster>) {
    SPI_MASTER = Some(spi);
}

static mut SPI_MASTER: Option<Rc<SPIMaster>> = None;
