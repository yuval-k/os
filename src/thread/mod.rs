use super::mem;
use core::sync::atomic;
use core::ops::Drop;
use platform;
use platform::ThreadId;
use collections::boxed::Box;
use alloc::boxed::FnBox;
use core::cell::RefCell;

pub enum RunState {
    Ready,
    WakeOn(usize),
    Never,
}

pub struct Thread {
    pub ctx: super::platform::ThreadContext,
    pub run_state: RunState,
    pub id: ThreadId,
    pub func : RefCell<Option<Box<FnBox()>>>,
    pub cpu_affinity: Option<usize>,
    pub priority: usize,
}

static STACK_BASE_COUNTER: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;
const STACK_PAGES: usize = 4;
const STACK_SIZE: usize = STACK_PAGES << platform::PAGE_SHIFT;
const STACK_BASE: ::mem::VirtualAddress = ::mem::VirtualAddress(0x100_0000);

impl Thread {


    fn free_stack(_ : ::mem::VirtualAddress) {
        // TODO: we leak the stack :-(
    }

    pub fn allocate_stack() -> ::mem::VirtualAddress {
        let oldcounter = STACK_BASE_COUNTER.fetch_add(STACK_SIZE, atomic::Ordering::SeqCst);
        let stack_start = STACK_BASE.uoffset(oldcounter);
        let stack_end   = stack_start.uoffset(STACK_SIZE);
        // allocate to pages
        let pv = platform::get_platform_services().frame_alloc.allocate(STACK_PAGES).unwrap();
        platform::get_platform_services().mem_manager.map(
           pv,
           stack_start,
           mem::MemorySize::PageSizes(STACK_PAGES)).expect("Can't map stack");
        
        stack_end 

    }

// TODO: remove the start address
    pub fn new(id : ThreadId, f: Box<FnBox()>) -> Self {
        Thread {
            ctx: platform::new_thread(Thread::allocate_stack()),
            run_state: RunState::Ready,
            id: id,
            func : RefCell::new(Some(f)),
            cpu_affinity: None,
            priority: 1,
        }
    }

    pub fn is_ready(&self) -> bool {
        if let RunState::Ready = self.run_state {
            return true;
        };
        false
    }

    pub fn new_cur_thread(id : ThreadId) -> Self {
        Thread{
                    ctx : platform::new_thread(::mem::VirtualAddress(0)),
                    run_state: RunState::Ready,
                    id : id,
                    func : RefCell::new(None),
                    cpu_affinity: None,
                    priority: 1,
        }
    }
}


impl Drop for Thread {
    fn drop(&mut self) {
        // TODO: free the stacks.
    }
}