use collections::VecDeque;
use core::cell::Cell;
use core::cell::RefCell;
use platform;

pub struct Semaphore {
    sema: platform::intr::InterruptGuard<SemaphoreImpl>,
}

pub struct SemaphoreGuard<'a> {
    sem: &'a Semaphore,
}

struct SemaphoreImpl {
    waiting: RefCell<VecDeque<super::ThreadId>>,
    counter: Cell<usize>,
}

impl Semaphore {
    pub fn new(count: usize) -> Semaphore {
        Semaphore {
            sema: platform::intr::InterruptGuard::new(SemaphoreImpl {
                waiting: RefCell::new(VecDeque::new()),
                counter: Cell::new(count),
            }),
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

    pub fn access(&self) -> SemaphoreGuard {
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
    fn acquire(&self) {
        // add to counter
        // protect with spin lock:
        // call scheduler to wake up potential sleeping threads
        // http://www.mpi-sws.org/~druschel/courses/os/lectures/proc4.pdf
        if self.counter.get() > 0 {
            self.counter.set(self.counter.get() - 1);
        } else {
            let cur_th = platform::get_platform_services().get_scheduler().get_current_thread();
            self.waiting
                .borrow_mut()
                .push_back(cur_th);
            platform::get_platform_services().get_scheduler().block_no_intr();
        }
    }

    fn release(&self) {
        // protect with spin lock:
        // if n is smaller than counter, just reduce the counter.
        // if n is bigger than counter,
        // tell the scheduler that the current thread is waking
        // for (n-counter) units to arrive.
        if self.waiting.borrow_mut().is_empty() {
            self.counter.set(self.counter.get() + 1);
        } else {
            let thread = self.waiting.borrow_mut().pop_front().unwrap();
            platform::get_platform_services().get_scheduler().wakeup_no_intr(thread); /* put thread on the ready queue */
        }
    }
}
