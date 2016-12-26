pub mod intr;
pub mod syscalls;

use collections::boxed::Box;
use alloc::rc::Rc;
use core::cell::UnsafeCell;
use collections::Vec;

#[cfg(target_arch = "arm")]
mod arm;
#[cfg(target_arch = "arm")]
pub use self::arm::*;

pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ThreadId(pub usize);

pub trait InterruptSource {
    // must be safe for concurrent calls.
    fn interrupted(&self, &mut Context);
}


pub struct PlatformServices {
    pub scheduler: super::sched::Sched,
    pub mem_manager: Box<::mem::MemoryManagaer>, 
    pub frame_alloc: Rc<::mem::FrameAllocator>,
    pub arch_services: Option<ArchPlatformServices>,
    pub cpus : Vec<::cpu::CPU>,
}

static mut platform_services: Option<UnsafeCell<PlatformServices>> = None;

pub unsafe fn set_platform_services(p: PlatformServices) {
      platform_services = Some(UnsafeCell::new(p));
}

pub fn get_platform_services() -> &'static PlatformServices {
    unsafe {
        match platform_services {
            Some(ref x) => &*x.get(),
            None => panic!(),
        }
    }
}

pub fn get_mut_platform_services() -> &'static mut PlatformServices {
    unsafe {
        match platform_services {
            Some(ref x) => &mut *x.get(),
            None => panic!(),
        }
    }
}

impl PlatformServices {
    pub fn get_scheduler(&self) -> &super::sched::Sched {
        &get_platform_services().scheduler
    }

    pub fn get_current_cpu(&self) -> &::cpu::CPU {
        & self.cpus[get_current_cpu_id()]
    }
}
