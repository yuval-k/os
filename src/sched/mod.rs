pub mod sema;

use collections::Vec;
use collections::boxed::Box;
use core::cell::RefCell;
use core::cell::Cell;
use super::platform;
use alloc::boxed::FnBox;


type C = super::platform::Context;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ThreadId(pub usize);

const WAKE_NEVER : u64 = 0xFFFFFFFF_FFFFFFFF;

struct Thread {
    ctx: C,
    ready: bool,
    id: ThreadId,
    wake_on: u64,
     /* TODO:
                   *
                   * wake_on: u32,
                   * id: u32,
                   * owns: Vec<u32>,
                   * blocks_on: u32,
                   * */
}
// TODO: make this Thread and SMP safe.
// TODO this is the one mega unsafe class, so it needs to take care of it's on safety.
pub struct Sched {
    threads: RefCell<Vec<Box<Thread>>>,
    idle_thread: Thread,
    curr_thread_index: Cell<usize>,
    thread_id_counter: Cell<usize>,
    time_since_boot_millies: Cell<u64>,
}

const IDLE_THREAD_ID: ThreadId = ThreadId(0);
const MAIN_THREAD_ID: ThreadId = ThreadId(1);

extern "C" fn thread_start(start: *mut Box<FnBox()>) {
    unsafe {
        let f: Box<Box<FnBox()>> = Box::from_raw(start);
        f();
        platform::get_platform_services().get_scheduler().exit_thread();
    }
}

impl Sched {
    pub fn new() -> Sched {
        Sched {
            // fake thread as this main thread..
            threads : RefCell::new(vec![Box::new(
                Thread{
                    ctx : platform::new_thread(::mem::VirtualAddress(0),::mem::VirtualAddress(0),0),
                    ready: true,
                    id : MAIN_THREAD_ID,
                    wake_on: 0,
                })
                ]),
            idle_thread: Thread{
                ctx : platform::new_thread(::mem::VirtualAddress(0), 
                    ::mem::VirtualAddress(platform::wait_for_interrupts as usize), 0),
                ready: true,
                id : IDLE_THREAD_ID,
                wake_on: 0,
            },
            curr_thread_index : Cell::new(0),
            thread_id_counter : Cell::new(10),
            time_since_boot_millies : Cell::new(10),
        }
    }

    pub fn spawn<F>(&self, stack: ::mem::VirtualAddress, f: F)
        where F: FnOnce(),
              F: Send + 'static
    {
        let p: Box<FnBox()> = Box::new(f);
        let ptr = Box::into_raw(Box::new(p)) as *const usize as usize; // some reson without another box ptr is 1
        self.spawn_thread(stack, ::mem::VirtualAddress(thread_start as usize), ptr);
    }

    pub fn spawn_thread(&self,
                        stack: ::mem::VirtualAddress,
                        start: ::mem::VirtualAddress,
                        arg: usize) {
        // TODO thread safety and SMP Support
        self.thread_id_counter.set(self.thread_id_counter.get() + 1);

        let t = Box::new(Thread {
            ctx: platform::new_thread(stack, start, arg),
            ready: true,
            id: ThreadId(self.thread_id_counter.get()),
            wake_on: 0,
        });

        let ig = platform::intr::no_interrupts();
        self.threads.borrow_mut().push(t);
        // find an eligble thread
        // threads.map()
    }

    // no interrupts here..
    pub fn schedule_no_intr(&self, old_ctx: &C) -> C {
        {
            let cur_th = self.curr_thread_index.get();
            self.threads.borrow_mut()[cur_th].ctx = *old_ctx;
        }
        // find an eligble thread
        // threads.map()
        return self.schedule_new();
    }

    fn schedule_new(&self) -> C {
        // find an eligble thread
        // threads.map()
        let mut cur_th = self.curr_thread_index.get();
        let num_threads = self.threads.borrow().len();
        for _ in 0..num_threads {
            cur_th += 1;
            // TODO linker with libgcc/compiler_rt so we can have division and mod
            if cur_th == num_threads {
                cur_th = 0;
            }

            {
                let ref mut cur_thread = self.threads.borrow_mut()[cur_th];
                if (cur_thread.wake_on != WAKE_NEVER) && (cur_thread.wake_on <= self.time_since_boot_millies.get()) {
                    cur_thread.wake_on = 0;
                    cur_thread.ready = true;
                }
            }

            let ref cur_thread = self.threads.borrow()[cur_th];
            if cur_thread.ready {
                self.curr_thread_index.set(cur_th);
                let ctx = cur_thread.ctx;
                return ctx;
            }
        }
        // no thread is ready.. time to sleep sleep...
        // return to the idle thread.
        // don't wait for interrupts here, as we might already be in an interrupt..
        self.idle_thread.ctx
    }

    pub fn exit_thread(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        // remove the current thread
        let cur_th = self.curr_thread_index.get();
        self.threads.borrow_mut().remove(cur_th);
        let new_context = self.schedule_new();
        // tmp ctx.. we don't really gonna use it...
        let mut c = platform::new_thread(::mem::VirtualAddress(0), ::mem::VirtualAddress(0), 0);
        platform::switch_context(&mut c, &new_context);

        // TODO - stack leaks here.. should we scheduler the schulder thread to clean it up.?

    }

    pub fn yield_thread(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        self.yeild_thread_no_intr()

    }

    fn yeild_thread_no_intr(&self) {
        let new_context: platform::Context;
        let curr_thread = self.curr_thread_index.get();

        // TODO: should we add a mutex for smp?
        new_context = self.schedule_new();

        if curr_thread != self.curr_thread_index.get() {
            // save the context, and go go go
            // pc needs to be after save context
            // use unsafe cell as the we have a context switch.
            let threads = unsafe { &mut *self.threads.as_ptr() };
            let ctx = &mut (threads[curr_thread].ctx);
            platform::switch_context(ctx, &new_context);
            // we don't get here :)
        }
    }

    pub fn block(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        self.block_no_intr()
    }

    pub fn sleep(&self, millis : u32) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        {
            let cur_th = self.curr_thread_index.get();
            let ref mut cur_thread = self.threads.borrow_mut()[cur_th];
            cur_thread.wake_on = self.time_since_boot_millies.get() + (millis as u64);
        }

        self.block_no_intr()
    }

    // assume interrupts are blocked
    pub fn block_no_intr(&self) {
        {
            let ref mut t = self.threads.borrow_mut()[self.curr_thread_index.get()];
            t.ready = false;
            if t.wake_on != 0 {
                t.wake_on = WAKE_NEVER;
            }
        }
        self.yeild_thread_no_intr();
    }

    // assume interrupts are blocked
    pub fn wakeup_no_intr(&self, tid: ThreadId) {
        for t in self.threads.borrow_mut().iter_mut().filter(|x| x.id == tid) {
            // there can only be one..
            t.wake_on = 0;
            t.ready = true;
            break;
        }
    }

    pub fn get_current_thread(&self) -> ThreadId {
        return self.threads.borrow()[self.curr_thread_index.get()].id;
    }

    // TODO
    pub fn lock(&mut self) {}

    pub fn unlock(&mut self) {}
}

// for the timer interrupt..
impl platform::InterruptSource for Sched {
    // this method is called platform::ticks_in_second times a second
    fn interrupted(&self, ctx: &mut platform::Context) {
        let delta_millis = (1000 / platform::ticks_in_second) as u64; 
        self.time_since_boot_millies.set(self.time_since_boot_millies.get() + delta_millis);
        *ctx = self.schedule_no_intr(ctx);
    }
}
