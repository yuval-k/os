use collections::Vec;
use collections::boxed::Box;
use core::cell::RefCell;
use core::cell::Cell;
use super::platform;
use alloc::boxed::FnBox;

use platform::ThreadId;


type C = super::platform::Context;


const WAKE_NEVER: u64 = 0xFFFFFFFF_FFFFFFFF;

struct Thread {
    ctx: C,
    ready: bool,
    id: ThreadId,
    wake_on: u64, /* TODO:
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
    sched_impl : RefCell<SchedImpl>
}

struct SchedImpl {
    threads: Vec<Box<Thread>>,
    idle_thread: Thread,
    curr_thread_index: Option<usize>,
    thread_id_counter: usize,
    time_since_boot_millies: u64,
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
        sched_impl : RefCell::new(SchedImpl {
            // fake thread as this main thread..
            threads: vec![Box::new(
                Thread{
                    ctx : platform::new_thread(::mem::VirtualAddress(0),::mem::VirtualAddress(0),0),
                    ready: true,
                    id : MAIN_THREAD_ID,
                    wake_on: 0,
                })
                ],
            idle_thread: Thread{
                ctx : platform::new_thread(::mem::VirtualAddress(0), 
                    ::mem::VirtualAddress(platform::wait_for_interrupts as usize), 0),
                ready: true,
                id : IDLE_THREAD_ID,
                wake_on: 0,
            },
            curr_thread_index : Some(0),
            thread_id_counter : 10,
            time_since_boot_millies : 0,
        })
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
        let mut simpl = self.sched_impl.borrow_mut();
        simpl.thread_id_counter +=1;

        let t = Box::new(Thread {
            ctx: platform::new_thread(stack, start, arg),
            ready: true,
            id: ThreadId(simpl.thread_id_counter),
            wake_on: 0,
        });

        let ig = platform::intr::no_interrupts();
        simpl.threads.push(t);
        // find an eligble thread
        // threads.map()
    }

    // no interrupts here..
    pub fn schedule_no_intr(&self, old_ctx: &C) -> C {
        {
            let mut simpl = self.sched_impl.borrow_mut();
            if let Some(cur_th_i) = simpl.curr_thread_index {
                simpl.threads[cur_th_i].ctx = *old_ctx;
            }
        }
        // find an eligble thread
        // threads.map()
        return self.schedule_new();
    }

    fn schedule_new(&self) -> C {
        // find an eligble thread
        // threads.map()
        let mut simpl = self.sched_impl.borrow_mut();

        let mut cur_th = if let Some(cur_th_i) = simpl.curr_thread_index { cur_th_i} else { 0 };

        let num_threads = simpl.threads.len();
        for _ in 0..num_threads {
            cur_th += 1;
            // TODO linker with libgcc/compiler_rt so we can have division and mod
            if cur_th == num_threads {
                cur_th = 0;
            }

            {
                let time_since_boot_millies = simpl.time_since_boot_millies;
                let cur_thread = &mut simpl.threads[cur_th];
                if (cur_thread.wake_on != WAKE_NEVER) &&
                   (cur_thread.wake_on <= time_since_boot_millies) {
                    cur_thread.wake_on = 0;
                    cur_thread.ready = true;
                }
            }

            let cur_thread_ready = simpl.threads[cur_th].ready;
            if cur_thread_ready {
                simpl.curr_thread_index = Some(cur_th);
                let ctx = simpl.threads[cur_th].ctx;
                return ctx;
            }
        }
        // no thread is ready.. time to sleep sleep...
        // return to the idle thread.
        // don't wait for interrupts here, as we might already be in an interrupt..
        simpl.curr_thread_index = None;
        simpl.idle_thread.ctx
    }

    pub fn exit_thread(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
     
        {
            let mut simpl = self.sched_impl.borrow_mut();
            // remove the current thread
            let cur_th = simpl.curr_thread_index.unwrap();
            simpl.threads.remove(cur_th);
        }
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

        let curr_thread = { self.sched_impl.borrow().curr_thread_index };

        // TODO: should we add a mutex for smp?

        let new_context = self.schedule_new();

        let new_thread = { self.sched_impl.borrow().curr_thread_index };


        if curr_thread != new_thread {
            // save the context, and go go go
            // pc needs to be after save context
            // use unsafe cell as the we have a context switch.
            let ctx = {
                let t = &mut self.sched_impl.borrow_mut().threads[curr_thread.unwrap()];
                unsafe{ &mut *(&mut t.ctx as *mut C) }
                // make sure borrow ends before call to switch_context. 
            };
            // context includes whether or not interrupts are enabled.
            //TODO: perhaps forbid contex switch with interrupts disabled?
            platform::switch_context(ctx, &new_context);
            // we don't get here :)
        }
    }

    pub fn block(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        self.block_no_intr()
    }

    pub fn sleep(&self, millis: u32) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        {
            let mut simpl = self.sched_impl.borrow_mut();
            let time_since_boot_millies = simpl.time_since_boot_millies;
            let cur_th = simpl.curr_thread_index.unwrap();
            let ref mut cur_thread = simpl.threads[cur_th];
            cur_thread.wake_on = time_since_boot_millies + (millis as u64);
        }

        self.block_no_intr()
    }

    // assume interrupts are blocked
    pub fn block_no_intr(&self) {
        {
            let mut simpl = self.sched_impl.borrow_mut();
            let cur_th = simpl.curr_thread_index.unwrap();

            let ref mut t = simpl.threads[cur_th];
            t.ready = false;
            if t.wake_on == 0 {
                t.wake_on = WAKE_NEVER;
            }
        }
        self.yeild_thread_no_intr();
    }

    // assume interrupts are blocked
    pub fn wakeup_no_intr(&self, tid: ThreadId) {
        let mut simpl = self.sched_impl.borrow_mut();

        for t in simpl.threads.iter_mut().filter(|x| x.id == tid) {
            // there can only be one..
            t.wake_on = 0;
            t.ready = true;
            break;
        }
    }

    pub fn get_current_thread(&self) -> ThreadId {
        let simpl = self.sched_impl.borrow();

        return simpl.threads[simpl.curr_thread_index.unwrap()].id;
    }

    // TODO
    pub fn lock(&mut self) {}

    pub fn unlock(&mut self) {}
}

// for the timer interrupt..
impl platform::InterruptSource for Sched {
    // this method is called platform::ticks_in_second times a second
    fn interrupted(&self, ctx: &mut platform::Context) {
        const DELTA_MILLIS: u64= (1000 / platform::ticks_in_second) as u64;
        {
            let mut simpl = self.sched_impl.borrow_mut();
            simpl.time_since_boot_millies +=  DELTA_MILLIS;
        }
        *ctx = self.schedule_no_intr(ctx);
    }
}
