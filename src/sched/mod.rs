
use kernel_alloc;
use collections::Vec;
use collections::boxed::Box;
use super::platform;

type C = super::platform::Context;

struct Thread{
    ctx: C,
    ready: bool,
    // TODO:
    /*
    wake_on: u32,
    id: u32,
    owns: Vec<u32>,
    blocks_on: u32,
    */
}
 // TODO: make this Thread and SMP safe.
 // TODO this is the one mega unsafe class, so it needs to take care of it's on safety.
pub struct Sched {
    threads: Vec<Box<Thread>>,
    curr_thread_index: usize,
}

impl Sched {

    pub fn new() -> Sched {
        Sched {
            // fake thread as this main thread..
            threads : vec![Box::new(Thread{
                ctx:platform::newThread(::mem::VirtualAddress(0),::mem::VirtualAddress(0),0),
                ready: true,
                }
                )],
            curr_thread_index : 0
        }
    }

    pub fn spawn_thread(&mut self, stack : ::mem::VirtualAddress, start : ::mem::VirtualAddress, arg : usize) {
        // TODO thread safety and SMP Support
        

        let t = Box::new(Thread{
                ctx:platform::newThread(stack, start, arg),
                ready: true,
        });
        self.threads.push(t);
        // find an eligble thread
        // threads.map()
    }

    pub fn schedule(&mut self, ctx : & C) -> C {
        self.threads[self.curr_thread_index].ctx = *ctx;
        // find an eligble thread
        // threads.map()
        return self.schedule_new();
    }

    fn schedule_new(&mut self) -> C {
        // find an eligble thread
        // threads.map()
        loop {
            for i in 0 .. self.threads.len() {
                self.curr_thread_index += 1;
                // TODO linker with libgcc/compiler_rt so we can have division and mod
                if self.curr_thread_index == self.threads.len() {
                    self.curr_thread_index = 0;
                }

                if self.threads[self.curr_thread_index].ready {
                    return self.threads[self.curr_thread_index].ctx;
                }
            }
            // no thread is ready.. time to sleep sleep...
            // return to the main thread.
            // don't wait for interrupts here, as we might already be in an interrupt..
        }
    }

    pub fn yield_thread(&mut self) {
        let curr_thread = self.curr_thread_index;
        // TODO: disable interrupts + mutex
        let newContext = self.schedule_new();
        // switch active thread and save context.
        // current thread <- thread
        // enable interrupts + unmutex
        
        if curr_thread != self.curr_thread_index {
            // save the context, and go go go
            // pc needs to be after save context
            platform::switchContext(&mut self.threads[curr_thread].ctx, &newContext);
            // can't use curr_thread.ctx from here on, as it might died during context switch

            // we don't get here :)

        }
    }

    // TODO
    pub fn lock(&mut self) {
        

    }
    pub fn unlock(&mut self) {
        

    }

}

impl platform::InterruptSource for Sched {
    fn interrupted(&mut self, ctx : &mut platform::Context)  {

        unsafe{
            // TODO make this thread safe; or later in the init and remove altogether...
            *ctx = self.schedule(ctx);
        }

    }
}