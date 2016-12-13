use super::mem;
use core::sync::atomic;
use core::ops::Drop;
use platform;
use platform::ThreadId;

pub struct Thread {
    pub ctx: super::platform::ThreadContext,
    pub ready: bool,
    pub id: ThreadId,
    pub wake_on: u64, /* TODO:
                   *
                   * wake_on: u32,
                   * id: u32,
                   * owns: Vec<u32>,
                   * blocks_on: u32,
                   * */
}

static STACK_BASE_COUNTER: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;
const STACK_PAGES: usize = 4;
const STACK_SIZE: usize = STACK_PAGES << platform::PAGE_SHIFT;
const STACK_BASE: ::mem::VirtualAddress = ::mem::VirtualAddress(0x100_0000);

impl Thread {


    fn free_stack(s : ::mem::VirtualAddress) {
        // TODO: we leak the stack :-(
    }

    fn allocate_stack() -> ::mem::VirtualAddress {
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

    pub fn new(id : ThreadId,start: ::mem::VirtualAddress, arg : usize) -> Self {
        Thread {
            ctx: platform::new_thread(Thread::allocate_stack(), start, arg),
            ready: true,
            id: id,
            wake_on: 0,
        }
    }

    pub fn new_cur_thread(id : ThreadId) -> Self {
        Thread{
                    ctx : platform::new_thread(::mem::VirtualAddress(0),::mem::VirtualAddress(0),0),
                    ready: true,
                    id : id,
                    wake_on: 0,
        }
    }

    fn exit(&self) {
        // free myself
    }
}


impl Drop for Thread {
    fn drop(&mut self) {
        // TODO: free the stacks.
    }
}