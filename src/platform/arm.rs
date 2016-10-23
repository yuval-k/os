pub use ::arch::arm::mem::PAGE_SHIFT;

pub use ::arch::arm::cpu::set_interrupts;
pub use ::arch::arm::cpu::get_interrupts;
pub use ::arch::arm::cpu::wait_for_interrupts;

pub type Context = ::arch::arm::thread::Context;
pub use ::arch::arm::thread::new_thread;
pub use ::arch::arm::thread::switch_context;

pub type ArchPlatformServices = ::arch::arm::PlatformServices;

pub use ::arch::arm::write_to_console;
