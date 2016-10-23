pub mod intr;

#[cfg(target_arch = "arm")]
mod arm;
#[cfg(target_arch = "arm")]
pub use self::arm::*;

use alloc::rc::Rc;
use core::cell::UnsafeCell;

pub const PAGE_SIZE : usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK : usize = PAGE_SIZE - 1;

pub trait InterruptSource {
    // must be safe for concurrent calls.
    fn interrupted(& self, &mut Context);
}

pub struct PlatformServices {
    pub scheduler : Rc<super::sched::Sched>,
    pub arch_services : ArchPlatformServices
}

static mut platform_services : Option<UnsafeCell<PlatformServices>> = None;

pub fn set_platform_services(p : PlatformServices) {
    unsafe {
        platform_services = Some(UnsafeCell::new(p))
    }
}

pub fn get_platform_services() -> &'static PlatformServices {
    unsafe {
        match platform_services {
            Some(ref x) => &*x.get(),
            None => panic!(),
        }
    }
}

impl PlatformServices {

    pub fn get_scheduler(&self) -> &super::sched::Sched { 
        &get_platform_services().scheduler 
    }

}
