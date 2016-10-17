#[cfg(target_arch = "arm")]
pub use ::arch::arm::cpu::set_interrupts;

#[cfg(target_arch = "arm")]
pub use ::arch::arm::cpu::get_interrupts;

#[cfg(target_arch = "arm")]
pub type Context = ::arch::arm::thread::Context;

#[cfg(target_arch = "arm")]
pub use ::arch::arm::thread::switchContext;

#[cfg(target_arch = "arm")]
pub use ::arch::arm::mem::PAGE_SHIFT;

#[cfg(target_arch = "arm")]
pub type ArchPlatformServices = ::arch::arm::PlatformServices;

pub const PAGE_SIZE : usize = 1<<PAGE_SHIFT;
pub const PAGE_MASK : usize = PAGE_SIZE - 1;

mod intr;

pub trait InterruptSource {
    fn interrupted(&mut self, &mut Context);
}