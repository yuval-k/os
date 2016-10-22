pub use ::arch::arm::mem::PAGE_SHIFT;

pub use ::arch::arm::cpu::set_interrupts;
pub use ::arch::arm::cpu::get_interrupts;
pub use ::arch::arm::cpu::wait_for_interrupts;

pub type Context = ::arch::arm::thread::Context;
pub use ::arch::arm::thread::newThread;
pub use ::arch::arm::thread::switchContext;

pub type ArchPlatformServices = ::arch::arm::PlatformServices;