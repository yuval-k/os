pub mod intr;
pub mod syscalls;

use collections::boxed::Box;
use alloc::rc::Rc;
use core::cell::UnsafeCell;
use collections::Vec;
use core::sync::atomic;

#[cfg(target_arch = "arm")]
mod arm;
#[cfg(target_arch = "arm")]
pub use self::arm::*;


pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ThreadId(pub usize);

pub trait InterruptableWithContext {
    // must be safe for concurrent calls.
    fn interrupted_ctx(&self, &mut Context);
}


pub trait Interruptable {
    // must be safe for concurrent calls.
    fn interrupted(&self);
}

impl<T: Interruptable>  InterruptableWithContext for T {
    fn interrupted_ctx(&self, _ : &mut Context){
        self.interrupted()
  }
}

pub struct MemoryServices {
// TODO: remove box and rc from here
    pub mem_manager: Box<::mem::MemoryManagaer>, 
    pub frame_alloc: Rc<::mem::FrameAllocator>,
}

pub struct PlatformServices {
    pub scheduler: super::sched::Sched,
    pub cpus : Vec<::cpu::CPU>,
    pub arch_services: ArchPlatformServices,
}

static mut PLATFORM_SERVICES: Option<UnsafeCell<PlatformServices>> = None;
static mut MEMORY_SERVICES: Option<UnsafeCell<MemoryServices>> = None;

pub unsafe fn set_memory_services(p: MemoryServices) {
      MEMORY_SERVICES = Some(UnsafeCell::new(p));
}

pub fn get_memory_services() -> &'static MemoryServices {
    unsafe {
        match MEMORY_SERVICES {
            Some(ref x) => &*x.get(),
            None => panic!("platform services are note INITIALIZED!"),
        }
    }
}
pub unsafe fn set_platform_services(p: PlatformServices) {
      PLATFORM_SERVICES = Some(UnsafeCell::new(p));
}

pub fn get_platform_services() -> &'static PlatformServices {
    unsafe {
        match PLATFORM_SERVICES {
            Some(ref x) => &*x.get(),
            None => panic!("platform services are note INITIALIZED!"),
        }
    }
}

pub unsafe fn get_mut_platform_services() -> &'static mut PlatformServices {
    match PLATFORM_SERVICES {
        Some(ref x) => &mut *x.get(),
        None => panic!(),
    }
}

impl PlatformServices {
    pub fn get_scheduler(&self) -> &super::sched::Sched {
        &get_platform_services().scheduler
    }

    pub fn get_current_cpu(&self) -> &::cpu::CPU {
        & self.cpus[get_current_cpu_id()]
    }

    pub fn clock(&self) {
        // TODO: move to scheduler to decide
        if get_current_cpu_id() == 0 {
            self.scheduler.clock();
        }
        self.get_current_cpu().should_resched.set(true);
    }

    // called with interrupts disabled..
    pub fn post_interrupted(&self, _ : &mut Context) {
        if self.get_current_cpu().should_resched.get() {
            self.get_current_cpu().should_resched.set(false);
            self.scheduler.yeild_thread_no_intr();
        }
    }
    
}


static IS_SYS_READY : atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;


pub fn set_system_ready() {

    IS_SYS_READY.store(true, atomic::Ordering::Release)
}

pub fn is_system_ready() -> bool {
    IS_SYS_READY.load(atomic::Ordering::Acquire)
}

