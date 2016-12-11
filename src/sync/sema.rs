use collections::VecDeque;
use core::cell::Cell;
use core::cell::RefCell;
use platform;

pub struct Semaphore {
    sema: super::CpuMutex<platform::intr::InterruptGuard<SemaphoreImpl>>,
}

unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}



pub struct SemaphoreGuard<'a> {
    sem: &'a Semaphore,
}

// TODO make SMP SAFE; i.e.    cpu mutex.
struct SemaphoreImpl {
    waiting: RefCell<VecDeque<platform::ThreadId>>,
    counter: Cell<usize>,
}

impl Semaphore {
    pub fn new(count: usize) -> Semaphore {
        Semaphore {
            sema: super::CpuMutex::new(platform::intr::InterruptGuard::new(SemaphoreImpl {
                waiting: RefCell::new(VecDeque::new()),
                counter: Cell::new(count),
            })),
        }
    }

    pub fn acquire(&self) {
        // add to counter
        // protect with spin lock:
        // call scheduler to wake up potential sleeping threads
        let mut ret : bool;
        {
        // make the cpu lock as short as possible.
        // we can't place a cpu mutex on block 
            let locked = self.sema.lock();
            ret = locked.no_interrupts().acquire();
        }

        if ret {
            platform::get_platform_services().get_scheduler().yield_thread();
            // TODO: add time out feature and check if we timed out.
        }

    }

    pub fn release(&self) {
        // protect with spin lock:
        // if n is smaller than counter, just reduce the counter.
        // if n is bigger than counter,
        // tell the scheduler that the current thread is waking
        // for (n-counter) units to arrive.
        let maybeThread : Option<platform::ThreadId>;
        {
            let locked = self.sema.lock();
            locked.no_interrupts().release();
        }
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
    fn acquire(&self) -> bool {
        // add to counter
        // protect with spin lock:
        // call scheduler to wake up potential sleeping threads
        // http://www.mpi-sws.org/~druschel/courses/os/lectures/proc4.pdf

        // TODO add memory barriers / sync barriers
        // see : http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.faqs/ka14041.html

        if self.counter.get() > 0 {
            self.counter.set(self.counter.get() - 1);
            return false;
        }

        platform::get_platform_services().get_scheduler().unschedule_no_intr();
        let cur_th = platform::get_platform_services().get_scheduler().get_current_thread();
        self.waiting
            .borrow_mut()
            .push_back(cur_th);
        
        return true;
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
            platform::get_platform_services().get_scheduler().wakeup_no_intr(thread);
        }
    }
}
