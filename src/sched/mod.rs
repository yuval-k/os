use collections::Vec;
use collections::boxed::Box;
use  core::sync::atomic;
use super::platform;
use super::thread;

use platform::ThreadId;
use sync;

// TODO: make this Thread and SMP safe.
// TODO this is the one mega unsafe class, so it needs to take care of it's on safety.
pub struct Sched {
    threads: sync::CpuMutex<Vec<Box<thread::Thread>>>,
    dying_threads: sync::CpuMutex<Vec<Box<thread::Thread>>>,
    thread_id_counter: atomic::AtomicUsize,
    time_since_boot_millies: atomic::AtomicUsize,
}

pub const MAIN_THREAD_ID: ThreadId = ThreadId(0);


impl Sched {
    pub fn new() -> Sched {
        Sched {
            // fake thread as this main thread..
            threads: sync::CpuMutex::new(vec![]),
            dying_threads: sync::CpuMutex::new(vec![]),
            thread_id_counter : atomic::AtomicUsize::new(1000),
            time_since_boot_millies :  atomic::AtomicUsize::new(0),
        }
    }

// TODO move start to thread object.
    pub fn thread_start(old_thread: Option<Box<thread::Thread>>, new_thread: Box<thread::Thread>) {

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
        
        platform::get_platform_services().get_scheduler().exit_thread();
        
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


    fn schedule_new(&self, run_thread : Option<&Box<thread::Thread>>) -> Option<Box<thread::Thread>> {
        // find an eligble thread
        // threads.map()
        let mut threads = self.threads.lock();

        let time_since_boot_millies = self.time_since_boot_millies.load(atomic::Ordering::Acquire);

        let curcpuid = platform::get_current_cpu_id();

        let mut chosen : Option<usize> = None;

        for i in 0.. threads.len() {
            {
                let mut cur_thread = &mut threads[i];
                    if let thread::RunState::WakeOn(wake_on) = cur_thread.run_state {
                        if wake_on >= time_since_boot_millies {
                            cur_thread.run_state = ::thread::RunState::Ready;
                        }
                    }
            }
            let  cur_thread = &threads[i];

            if let Some(affinity) = cur_thread.cpu_affinity {
                if affinity != curcpuid {
                    continue
                }
            }
            if ! cur_thread.is_ready()  {
                    continue
            }

            if let Some(index) = chosen {

                if cur_thread.priority > threads[index].priority {
                    chosen = Some(i);
                }
            } else if let Some(run_thread) = run_thread {
                if (run_thread.is_ready()) && (cur_thread.priority > run_thread.priority)  {
                 chosen = Some(i);
                }
            } 

            if chosen.is_none() {
                chosen = Some(i);
            }

        }
            
        if let Some(index) = chosen {
            return Some(threads.swap_remove(index));
        }
        if let Some(run_thread) = run_thread {
            if run_thread.is_ready() {
                return None
            }
        }
        // no thread is ready.. panic
        panic!("No thread to run!")
    }

    pub fn exit_thread(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();

        let mut curr_thread = ::platform::get_platform_services().get_current_cpu().take_running_thread();
        // we need to delete the stack, and we can't do it right now, we can only do it after the ctx switch,
        // so instead of delete the thread here, place it in a list.
        {
            let mut threads = self.dying_threads.lock();
            curr_thread.cpu_affinity = Some(platform::get_current_cpu_id());
            threads.push(curr_thread);
        }
        let new_thread = self.schedule_new(None);

        platform::switch_context(None, new_thread.expect("No thread to run"));
        // never gonna get here..

    }

    pub fn yield_thread(&self) {
        // disable interrupts
        let ig = platform::intr::no_interrupts();
        self.yeild_thread_no_intr()

    }

    pub fn yeild_thread_no_intr(&self) {

        let curr_thread = ::platform::get_platform_services().get_current_cpu().take_running_thread();

        // get new thread to run
        let new_thread = self.schedule_new(Some(&curr_thread));

        if new_thread.is_none() {
            // short path - thread has not changed..
            ::platform::get_platform_services().get_current_cpu().set_running_thread(curr_thread);
            return
        }

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

        /* MemBar incase thread goes to other cpu */

        platform::memory_write_barrier();

        let (old, current) = platform::switch_context(Some(curr_thread), new_thread.unwrap());

        platform::memory_read_barrier();

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

            cur_thread.run_state = thread::RunState::WakeOn(
                self.time_since_boot_millies.load(atomic::Ordering::Acquire) + (millis as usize)
            );

        }

        self.yeild_thread_no_intr()
    }

    pub fn unschedule_no_intr(&self) {

        let mut curthread_cell = platform::get_platform_services().get_current_cpu().get_running_thread().borrow_mut();
        let mut t = curthread_cell.as_mut().unwrap();
        if t.is_ready() {
            t.run_state = thread::RunState::Never;
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
            t.run_state = thread::RunState::Ready;
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

    // this method is called about platform::ticks_in_second times a second
    pub fn clock(&self) {
        const DELTA_MILLIS: usize = (1000 / platform::ticks_in_second) as usize;
        // TODO fix time_since_boot_millies to be in cell?!
        self.time_since_boot_millies.fetch_add(DELTA_MILLIS, atomic::Ordering::Release); 
    }
}
