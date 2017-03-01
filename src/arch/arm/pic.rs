use collections::Vec;
use collections::boxed::Box;
use platform;
use sync::CpuMutex;


pub trait InterruptSource {
    fn range(&self) -> (usize,usize);
    fn enable( &self, interrupt : usize);
    fn disable(&self, interrupt : usize);
    fn is_interrupted(&self, interrupt : usize) -> bool;
}

#[derive(Clone, Copy)]
struct InterruptState {
    driver_handle : Option<super::DriverHandle>,
    is_enabled : bool,
}

impl InterruptState {
    fn new() -> Self {
        InterruptState{
            driver_handle : None,
            is_enabled : false,
        }
    }
}

pub struct PIC {
    sources : CpuMutex<Vec<Box<InterruptSource>>>,
    callbacks :  CpuMutex<Vec<InterruptState>>,
}

pub struct InterruptAttachment {
    num : usize,
}

impl InterruptAttachment {
    pub fn new(num : usize ,driver : super::DriverHandle) -> Self {
        platform::get_platform_services().arch_services.interrupt_service.register_callback_on_intr(num, driver);
        InterruptAttachment {
            num : num
        }
    }
}
/*

impl Drop for InterruptAttachment {
    fn drop(&mut self) {
        platform::get_platform_services().arch_services.interrupt_service.unregister_callback_on_intr(self.num)
    }
}
*/

/*
pub fn foo() {
    borrow pic
    ?? static
    // BM stuff: finish board with: points for rgb, programming header for isp / ftdi; pro mini shield mode?
    // goal is to test communication with 10w LED under various conditions    
}
*/

// TODO: maybe change to iterators?!

impl PIC {
    pub fn new() -> PIC {
        PIC { 
            sources :  CpuMutex::new(vec![]),
            callbacks : CpuMutex::new(vec![]),
        }
    }

    pub fn add_source<T : InterruptSource + 'static> (&self, is : T) {
        {
            let mut v = self.callbacks.lock();
            let (_, end) = is.range();
            let cursize = v.len();
            for _ in cursize..end {
                v.push(InterruptState::new());
            }
        }
        let mut sources = self.sources.lock();
        sources.push(Box::new(is));
    }

    pub fn register_callback_on_intr(&self, interrupt : usize, driver : super::DriverHandle) {
        // find callback index:
        let mut v = self.callbacks.lock();
        v[interrupt].driver_handle = Some(driver);
    }

    pub fn enable_registered(&self) {
        // find callback index:
        let mut callbacks = self.callbacks.lock();
        let mut callbacks : &mut Vec<InterruptState> = &mut callbacks;
        
        for (i,is) in callbacks.iter_mut().enumerate().filter(|&(_, ref is)|is.driver_handle.is_some()) {
            is.is_enabled = true;
            self.enable_interrupt(i);
        }
    }

    pub fn enable_interrupt(&self, interrupt : usize) {
        // find callback index:
        let sources = self.sources.lock();
        for source in sources.iter() { 
            let (start,end) = source.range();
            if (interrupt >= start) && (interrupt < end) {
                source.enable(interrupt);
                return;
            }
        }
    }

}

impl platform::Interruptable for PIC {
    fn interrupted(&self) {
        // TODO remove cpu mutex ASAP! this makes interrupt handling not happen in 
        // parallel which is shit!
        let sourceslock = self.sources.lock();
        let sources : &Vec<Box<InterruptSource>> = sourceslock.as_ref();
        for is in sources {
            let (start, end) = is.range();
            for intr in start..end {

                if is.is_interrupted(intr)  {
                    let intrstate = {self.callbacks.lock()[intr]};
                    if intrstate.is_enabled {
                        if let Some(cb) = intrstate.driver_handle {
                            platform::get_platform_services().arch_services.driver_manager.driver_interrupted(cb);
                        } else {
                            panic!("unexpected interrupt")
                        }
                    }
                }
            }
        }
    }
}
