use collections::VecDeque;
use alloc::rc::Rc;
use core::cell::Cell;
use core::cell::RefCell;
use super::Sched;
use platform;

use spin;

pub struct Semaphore {
    sema : platform::intr::InterruptGuard<SemaphoreImpl>,
}

pub struct SemaphoreGuard<'a> {
    sem: &'a Semaphore,
}

struct SemaphoreImpl {
    waiting : RefCell<VecDeque<super::ThreadId>>,
    counter : Cell<usize>,
    sched : &'static super::Sched,
}

impl Semaphore {

    pub fn new() -> Semaphore {
        Semaphore{
            sema : platform::intr::InterruptGuard::new(
                    SemaphoreImpl {
                    waiting : RefCell::new(VecDeque::new()),
                    counter : Cell::new(0),
                    sched : platform::get_platform_services().get_scheduler(),
                }
            )
        }
    }

    pub fn acquire(&self) {
        // add to counter 
        // protect with spin lock:
        // call scheduler to wake up potential sleeping threads
        self.sema.no_interrupts().acquire();
        
    }

    pub fn release(&self) {
        // protect with spin lock:
        // if n is smaller than counter, just reduce the counter.
        // if n is bigger than counter, 
        // tell the scheduler that the current thread is waking 
        // for (n-counter) units to arrive.
        self.sema.no_interrupts().release();
    }

    pub fn access(& self) -> SemaphoreGuard {
        self.acquire();
        SemaphoreGuard { sem: self }
    }
}

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        self.sem.release();
    }
}

impl SemaphoreImpl {

    fn acquire(& self) {
        // add to counter 
        // protect with spin lock:
        // call scheduler to wake up potential sleeping threads
        // http://www.mpi-sws.org/~druschel/courses/os/lectures/proc4.pdf
        if self.counter.get() > 0 {
            self.counter.set(self.counter.get() + 1);
        } else {
            self.waiting.borrow_mut().push_back(self.sched.get_current_thread());
            self.sched.block();
        }

    }

    fn release(& self) {
        // protect with spin lock:
        // if n is smaller than counter, just reduce the counter.
        // if n is bigger than counter, 
        // tell the scheduler that the current thread is waking 
        // for (n-counter) units to arrive.
        if self.waiting.borrow_mut().is_empty() {
            self.counter.set(self.counter.get() + 1);
        } else {
            let thread = self.waiting.borrow_mut().pop_front().unwrap();
            self.sched.wakeup(thread); /* put thread on the ready queue */
        }
    }

}