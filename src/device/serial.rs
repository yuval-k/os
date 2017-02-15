
use collections::boxed::Box;
use alloc::rc::Rc;
use device::IoDevice;
use io;

pub trait SerialMMIO {
    fn write_byte_async(&mut self, ch: u8);
    fn is_done(&self) -> bool;

    fn write_byte(&mut self, ch: u8) {
        while !self.is_done() {}
        self.write_byte_async(ch);
    }

    fn write(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_byte(ch as u8);
        }
    }
    fn writeln(&mut self, s: &str) {
        self.write(s);
        self.write_byte('\n' as u8);
    }
}


pub fn get_serial() -> Option<&'static IoDevice> {
    unsafe {
        match SERIAL {
            None => None,
            Some(ref ser) => Some(ser.as_ref())
        }
    }
}

pub unsafe  fn set_serial( s : Rc<IoDevice>) {
        SERIAL = Some(s);
}


static mut SERIAL : Option<Rc<IoDevice>> = None;
