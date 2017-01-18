use core::iter::Iterator;
use collections::Vec;
use core::ops::Range;
use platform;
use collections::boxed::Box;

pub trait InterruptSource {
    fn iter(&self) -> Range<usize>;
    fn enable(&self, interrupt : usize);
    fn disable(&self, interrupt : usize);
    fn is_interrupted(&self, interrupt : usize) -> bool;
    fn interrupted(&self, interrupt : usize, ctx: &mut platform::Context);
}

pub struct PIC {
    sources : Vec<Box<InterruptSource>>
}

impl PIC {
    pub fn new() -> PIC {
        PIC { sources : vec![] }
    }

    pub fn add_source(&mut self, is : Box<InterruptSource>) {
        self.sources.push(is)
    }
}



impl platform::Interruptable for PIC {
    fn interrupted(&self, ctx: &mut platform::Context) {

        for is in &self.sources {
            for intr in is.iter() {
                if is.is_interrupted(intr) {
                    is.interrupted(intr, ctx)
                }
            }
        }
    }
}
