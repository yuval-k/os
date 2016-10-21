use collections::VecDeque;
use alloc::rc::Rc;
use platform;

use spin;

pub struct Semaphore {
    sema : platform::intr::InterruptGuard<SemaphoreImpl>,
}

impl Semaphore {

    fn add(&mut self) {
        // add to counter 
        // protect with spin lock:
        // call scheduler to wake up potential sleeping threads
        self.sema.no_interrupts().add();
        
    }

    fn take(&mut self) {
        // protect with spin lock:
        // if n is smaller than counter, just reduce the counter.
        // if n is bigger than counter, 
        // tell the scheduler that the current thread is waking 
        // for (n-counter) units to arrive.
        self.sema.no_interrupts().take();
    }

}

struct SemaphoreImpl {
    waiting : VecDeque<super::ThreadId>,
    counter : usize,
    sched : &'static mut super::Sched,
}


impl SemaphoreImpl {

    fn add(&mut self) {
        // add to counter 
        // protect with spin lock:
        // call scheduler to wake up potential sleeping threads
        // http://www.mpi-sws.org/~druschel/courses/os/lectures/proc4.pdf
        if self.counter > 0 {
            self.counter -= 1;
        } else {
            self.waiting.push_back(self.sched.get_current_thread());
            self.sched.block();
        }

    }

    fn take(&mut self) {
        // protect with spin lock:
        // if n is smaller than counter, just reduce the counter.
        // if n is bigger than counter, 
        // tell the scheduler that the current thread is waking 
        // for (n-counter) units to arrive.
        if self.waiting.is_empty() {
            self.counter += 1;
        } else {
            let thread = self.waiting.pop_front().unwrap();
            self.sched.wakeup(thread); /* put thread on the ready queue */
        }
    }

}
