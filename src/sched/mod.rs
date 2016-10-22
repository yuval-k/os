mod sema;

use kernel_alloc;
use collections::Vec;
use collections::boxed::Box;
use core::cell::RefCell;
use core::cell::Cell;
use super::platform;

type C = super::platform::Context;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ThreadId(pub usize);

struct Thread{
    ctx: C,
    ready: bool,
    id : ThreadId,
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
    threads: RefCell<Vec<Box<Thread>>>,
    idle_thread: Thread,
    curr_thread_index: Cell<usize>,
    thread_id_counter: Cell<usize>,
}

const IDLE_THREAD_ID :  ThreadId = ThreadId(0);
const MAIN_THREAD_ID :  ThreadId = ThreadId(1);

impl Sched {

    pub fn new() -> Sched {
        Sched {
            // fake thread as this main thread..
            threads : RefCell::new(vec![Box::new(Thread{
                                    ctx : platform::newThread(::mem::VirtualAddress(0),::mem::VirtualAddress(0),0),
                                    ready: true,
                                    id : MAIN_THREAD_ID,
                },
                )]),
            idle_thread: Thread{
                ctx : platform::newThread(::mem::VirtualAddress(0), ::mem::VirtualAddress(platform::wait_for_interrupts as usize), 0),
                ready: true,
                id : IDLE_THREAD_ID,
            },
            curr_thread_index : Cell::new(0),
            thread_id_counter : Cell::new(10),
        }
    }

    pub fn spawn_thread(& self, stack : ::mem::VirtualAddress, start : ::mem::VirtualAddress, arg : usize) {
        // TODO thread safety and SMP Support
        self.thread_id_counter.set(self.thread_id_counter.get() + 1);

        let t = Box::new(Thread{
                ctx:platform::newThread(stack, start, arg),
                ready: true,
                id : ThreadId(self.thread_id_counter.get()),
        });
        self.threads.borrow_mut().push(t);
        // find an eligble thread
        // threads.map()
    }

    pub fn schedule(& self, ctx : & C) -> C {
        self.threads.borrow_mut()[self.curr_thread_index.get()].ctx = *ctx;
        // find an eligble thread
        // threads.map()
        return self.schedule_new();
    }

    fn schedule_new(& self) -> C {
        // find an eligble thread
        // threads.map()
        let num_threads = self.threads.borrow().len();
        for i in 0 .. num_threads {
            self.curr_thread_index.set(self.curr_thread_index.get() + 1);
            // TODO linker with libgcc/compiler_rt so we can have division and mod
            if self.curr_thread_index.get() == num_threads {
                self.curr_thread_index.set(0);
            }

            if self.threads.borrow()[self.curr_thread_index.get()].ready {
                return self.threads.borrow()[self.curr_thread_index.get()].ctx;
            }
        }
        // no thread is ready.. time to sleep sleep...
        // return to the idle thread.
        // don't wait for interrupts here, as we might already be in an interrupt..
        self.idle_thread.ctx
    }

    pub fn yield_thread(& self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        self.yeild_thread_internal()
    }

    fn yeild_thread_internal(& self) {
        let newContext : platform::Context;
        let curr_thread = self.curr_thread_index.get();

        // TODO: should we add a mutex for smp?
        newContext = self.schedule_new();
        
        if curr_thread != self.curr_thread_index.get() {
            // save the context, and go go go
            // pc needs to be after save context
            platform::switchContext(&mut self.threads.borrow_mut()[curr_thread].ctx, &newContext);
            // we don't get here :)
        }
    }

    // assume interrupts are blocked
    pub fn block(& self) {
        self.threads.borrow_mut()[self.curr_thread_index.get()].ready = false;
        self.yeild_thread_internal();
    }

    // assume interrupts are blocked
    pub fn wakeup(& self, tid : ThreadId) {
        self.threads.borrow_mut().iter_mut().filter(|x| x.id == tid).map(|x| x.ready = true);
    }

    pub fn get_current_thread(&self) -> ThreadId {
        return self.threads.borrow()[self.curr_thread_index.get()].id;
    }

    // TODO
    pub fn lock(&mut self) {
        
    }

    pub fn unlock(&mut self) {
        
    }

}

// for the timer interrupt..
impl platform::InterruptSource for Sched {
    fn interrupted(& self, ctx : &mut platform::Context) {
        unsafe {
            // TODO make this thread safe; or later in the init and remove altogether...
            *ctx = self.schedule(ctx);
        }
    }
}
