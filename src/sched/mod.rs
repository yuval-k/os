
use kernel_alloc;
use collections::Vec;
use collections::boxed::Box;

// TODO remove this for something generic..
use ::arch::arm::vector;

// TODO make generic
type C = ::arch::arm::vector::Context;


pub struct Thread{
    pub ctx: C,
    // TODO:
    /*
    wake_on: u32,
    id: u32,
    owns: Vec<u32>,
    blocks_on: u32,
    */
}

pub struct Sched {
    threads: Vec<Box<Thread>>,
    curr_thread_index: usize,
}

impl Sched {

    pub fn new(cur : Box<Thread>) -> Sched {
        Sched {
            threads : vec![cur],
            curr_thread_index : 0
        }
    }

    pub fn spawn_thread(&mut self, f: fn()) {
        // TODO thread safety and SMP Support

        // find an eligble thread
        // threads.map()
    }

    pub fn schedule(&mut self) -> C {
        // find an eligble thread
        // threads.map()
        return self.threads[0].ctx;
    }

    pub fn yield_thread(&mut self) {
        // TODO: disable interrupts + mutex
        let newContext = self.schedule();
        // switch active thread and save context.
        // current thread <- thread
        // enable interrupts + unmutex
        
        // save the context, and go go go
        // pc needs to be after save context
        vector::switchContext(&mut self.threads[self.curr_thread_index].ctx, &newContext);
        // can't use curr_thread.ctx from here on, as it might died during context switch

        // we don't get here :)
    }

    // TODO
    pub fn lock(&mut self) {
        

    }
    pub fn unlock(&mut self) {
        

    }

}