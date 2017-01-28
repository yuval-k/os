use collections::Vec;
use collections::boxed::Box;
use  core::sync::atomic;
use alloc::boxed::FnBox;
use core::cell::RefCell;
use core::cell::Cell;
use super::platform;
use super::thread;

use platform::ThreadId;
use sync;


const WAKE_NEVER: u64 = 0xFFFFFFFF_FFFFFFFF;

// TODO: make this Thread and SMP safe.
// TODO this is the one mega unsafe class, so it needs to take care of it's on safety.
pub struct Sched {
    threads: sync::CpuMutex<Vec<Box<thread::Thread>>>,
    thread_id_counter: atomic::AtomicUsize,
    time_since_boot_millies: RefCell<u64>,
}

const IDLE_THREAD_ID: ThreadId = ThreadId(0);
pub const MAIN_THREAD_ID: ThreadId = ThreadId(1);


impl Sched {
    pub fn new() -> Sched {
        Sched {
            // fake thread as this main thread..
            threads: sync::CpuMutex::new(vec![]),
            thread_id_counter : atomic::AtomicUsize::new(10),
            time_since_boot_millies : RefCell::new(0),
        }
    }

    pub fn add_idle_thread_for_cpu(&mut self) {
        let mut idle = Self::new_thread_obj(IDLE_THREAD_ID, platform::wait_for_interrupts );
        idle.cpu_affinity = Some(platform::get_current_cpu_id());
        idle.priority = 0;
        self.threads.lock().push(idle);
    }

// TODO move start to thread object.
    pub fn thread_start(old_thread: Option<Box<::thread::Thread>>, new_thread: Box<::thread::Thread>) {

        // release_old_thread();
        // acquire_new_thread();
        // TODO assert that running thread is None.
        let newthreadfun = new_thread.func.borrow_mut().take().unwrap();
        ::platform::get_platform_services().get_current_cpu().set_running_thread(new_thread);

        if let Some(old) = old_thread {
            let mut threads = platform::get_platform_services().get_scheduler().threads.lock();
            threads.push(old);
        }

        // all plumbing set! we can enable interrupts
        ::platform::intr::enable_interrupts();

        (newthreadfun)();
        
        unsafe {
            platform::get_platform_services().get_scheduler().exit_thread();
        }
    }

    fn new_thread_obj<F>(tid: ThreadId, f: F) -> Box<thread::Thread>
        where F: FnOnce(),
              F: Send + 'static {

        Box::new(thread::Thread::new(tid,  Box::new(f)))
    }

    pub fn spawn<F>(&self, f: F)
        where F: FnOnce(),
              F: Send + 'static {
        // TODO thread safety and SMP Support
        let tid = self.thread_id_counter.fetch_add(1, atomic::Ordering::SeqCst);

        let t = Self::new_thread_obj(ThreadId(tid), f);

        let ig = platform::intr::no_interrupts();
        self.threads.lock().push(t);
    }

    fn can_current_continue(&self) -> bool {

        let curthread_cell = platform::get_platform_services().get_current_cpu().get_running_thread().borrow();
        let curthread = curthread_cell.as_ref().unwrap();
        if ! curthread.ready {
            return false;
        }
                
        let threads = self.threads.lock();
        
        // is there one other thread that can run?
        threads.iter().filter(|&t| t.ready == true).filter(|&t| (t.cpu_affinity == None) || (t.cpu_affinity == Some(platform::get_current_cpu_id()))).
        filter(|&t| t.priority >= curthread.priority).next().is_some()
    }

    fn schedule_new(&self) -> Box<::thread::Thread> {
        // find an eligble thread
        // threads.map()
        let mut threads = self.threads.lock();
/*
        // wake up a sleeping threads
        for sleepingt in threads
        .filter(|&t| t.ready == false)
        .filter(|&t| t.wake_on != WAKE_NEVER)
        .filter(|&t| t.wake_on <= time_since_boot_millies) {
            sleepingt.wake_on = 0;
            sleepingt.ready = true;
        }
*/
        let num_threads = threads.len();
        let time_since_boot_millies = *self.time_since_boot_millies.borrow();

        for i in 0..num_threads {
            let chosen = {
                let mut cur_thread = &mut threads[i];
                if  cur_thread.priority == 0 {
                    continue;
                }
                let affinity =  cur_thread.cpu_affinity;
                if (affinity != None) && (affinity != Some(platform::get_current_cpu_id())) {
                    continue;
                }

                {
                    if (cur_thread.wake_on != WAKE_NEVER) &&
                    (cur_thread.wake_on >= time_since_boot_millies) {
                        cur_thread.wake_on = 0;
                        cur_thread.ready = true;
                    }
                }

                cur_thread.ready
            };

            if chosen {
                return threads.swap_remove(i);
            }
        }

        // no thread is ready.. time to sleep sleep... find the idle thread:

        for i in 0..num_threads {

            if  threads[i].priority != 0 {
                continue;
            }
            let affinity =  threads[i].cpu_affinity;
            if affinity == Some(platform::get_current_cpu_id()) {
                return threads.swap_remove(i);
            }
        }
        // return to the idle thread.
        panic!("No thread to run!")
    }

    pub fn exit_thread(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();

        let curr_thread = ::platform::get_platform_services().get_current_cpu().take_running_thread();
        // TODO place in deleted threads list..
        // instead of leaking the thread..
        ::core::mem::forget(curr_thread);

        let new_thread = self.schedule_new();

        platform::switch_context(None, new_thread);
        // never gonna get here..

    }

    pub fn yield_thread(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        self.yeild_thread_no_intr()

    }

    pub fn yeild_thread_no_intr(&self) {

        // this can't be the idle thread...

        if self.can_current_continue() {
            return
        }

        // current thread can't continue to run..
        // take the current thread from CPU
        let curr_thread = ::platform::get_platform_services().get_current_cpu().take_running_thread();

        let new_thread = self.schedule_new();

        // take new out from the thread list, and switch to it
        // current <- cpu.current
        // new thread = schedule(){threads.remove(tid)}
        // if scedule turns none, do nothing..
        // if it returns a thread of lower priority
        // old = switch(current, new)
        // after switch, current shoud go on cpu
        // // cpu.current should be == old 
        // // and the old new is now current
        // cpu.current = current
        // threads.insert(old)

        let (old, current) = platform::switch_context(Some(curr_thread), new_thread);

        ::platform::get_platform_services().get_current_cpu().set_running_thread(current);

        if let Some(old) = old {
            let mut threads = self.threads.lock();
            threads.push(old);
        }

        // cur_thread thread is now running!

        // we get here when context is switch back to us
        // TODO: SMP: mark that the old thread is no longer running on CPU
        // oldT is the thing that was just running on the cpu
        // re insert it to the thread list
        // insert new to cpu current

        
    }

    pub fn block(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        self.block_no_intr()
    }

    pub fn sleep(&self, millis: u32) {
        // disable interrupts
        //TODO how to release cpu guard after the context was saved?!
        let ig = platform::intr::no_interrupts();

        {
            let mut curthread_cell = platform::get_platform_services().get_current_cpu().get_running_thread().borrow_mut();
            let mut cur_thread = curthread_cell.as_mut().unwrap();

            cur_thread.wake_on = {
                *self.time_since_boot_millies.borrow() + (millis as u64)
            };
            cur_thread.ready = false;
        }

        self.yeild_thread_no_intr()
    }

    pub fn unschedule_no_intr(&self) {

        let mut curthread_cell = platform::get_platform_services().get_current_cpu().get_running_thread().borrow_mut();
        let mut t = curthread_cell.as_mut().unwrap();
        t.ready = false;
        if t.wake_on == 0 {
            t.wake_on = WAKE_NEVER;
        }
    }

    // assume interrupts are blocked
    pub fn block_no_intr(&self) {
        self.unschedule_no_intr();
        self.yeild_thread_no_intr();
    }

    pub fn wakeup(&self, tid: ThreadId) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        self.wakeup_no_intr(tid)
    }

    // assume interrupts are blocked
    pub fn wakeup_no_intr(&self, tid: ThreadId) {
        let mut threads = self.threads.lock();
        for t in threads.iter_mut().filter(|x| x.id == tid) {
            // there can only be one..
            t.wake_on = 0;
            t.ready = true;
            break;
        }
        // TODO: if we have other CPUs sleeping wake them up with an IPI...
    }

    pub fn get_current_thread(&self) -> ThreadId {

        let curthread_cell = platform::get_platform_services().get_current_cpu().get_running_thread().borrow();
        let cur_thread = curthread_cell.as_ref().unwrap();
        cur_thread.id
    }

    // TODO
    pub fn lock(&mut self) {}

    pub fn unlock(&mut self) {}

}

// this function runs in the context of whatever thread was interrupted.
fn handle_interrupts() {
    let ig = platform::intr::no_interrupts();
    
    // copy context to stack;
  //  let ctx = *get_cur_cpu().get_context();
    // switch!
   // yield();


}


// for the timer interrupt..
// TODO: delete..
impl platform::Interruptable for Sched {
    // this method is called platform::ticks_in_second times a second
    fn interrupted(&self, ctx: &mut platform::Context) {
        const DELTA_MILLIS: u64= (1000 / platform::ticks_in_second) as u64;
        {
            // TODO fix time_since_boot_millies to be in cell?!
            *self.time_since_boot_millies.borrow_mut() +=  DELTA_MILLIS;
        }
        // TODO: change to yeild? - we need yield to mark the unscheduled 
        // thread as unscheduled.. so it can continue to run on other cpus.. 
        // and release cpu mutex..
        // TODO(YES!): switch to CPU SCHEDULER THREAD (per cpu) not in interrupt mode..
        self.yeild_thread_no_intr();
        // TODO: need to notify that context was switched
        // set pc to handle_interrupts
        // set r0 to lr
    }
}
