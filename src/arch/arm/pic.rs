use core::iter::Iterator;
use collections::Vec;
use core::ops::Range;
use platform;
use collections::boxed::Box;
use core::borrow::Borrow;


pub trait InterruptSource {
    fn len(&self) -> usize;
    fn enable( &self, interrupt : usize);
    fn disable(&self, interrupt : usize);
    fn is_interrupted(&self, interrupt : usize) -> bool;
}

pub struct PIC<InterruptSourceT: Borrow<InterruptSource> >{
    sources : Vec<InterruptSourceT>,
    callbacks : Vec<Option<Box<platform::Interruptable>>>,
}

pub struct InterruptSourceHandle(usize);


impl<InterruptSourceT: Borrow<InterruptSource> >  PIC<InterruptSourceT> {
    pub fn new() -> PIC<InterruptSourceT> {
        PIC { 
            sources : vec![],
            callbacks : vec![],
        }
    }

    pub fn add_source(&mut self, is : InterruptSourceT) -> InterruptSourceHandle {
        {   let is : &InterruptSource = is.borrow();
            for _ in 0..is.len() {
                self.callbacks.push(None);
            }
        }
        self.sources.push(is);
        InterruptSourceHandle(self.sources.len() - 1)
    }

    pub fn register_callback_on_intr(&mut self, h : InterruptSourceHandle, interrupt : usize, handler : Box<platform::Interruptable>) {
        // find callback index:
        let ci = self.get_callback_index(h.0, interrupt);
        self.callbacks[ci] = Some(handler);
    }

    fn get_callback_index(&self, source_index : usize, interrupt : usize) -> usize {
        let mut ci = 0;
        for is in &self.sources[..source_index] {
            let is : &InterruptSource = is.borrow();
            ci += is.len();
        }
        ci += interrupt;
        ci
    }
}

impl<InterruptSourceT: Borrow<InterruptSource> >  platform::Interruptable for PIC<InterruptSourceT> {
    fn interrupted(&self, ctx: &mut platform::Context) {
        let mut source_index : usize = 0;
        for is in &self.sources {
            let is : &InterruptSource = is.borrow();
            for intr in 0..(is.len()) {
                if is.is_interrupted(intr) {
                    let idx = self.get_callback_index(source_index, intr);
                    if let Some(ref cb) = self.callbacks[idx] {
                        cb.interrupted(ctx);
                    } else {
                        panic!("unexpected interrupt")
                    }
                }
            }
            source_index += 1;
        }
    }
}
