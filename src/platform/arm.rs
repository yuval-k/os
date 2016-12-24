pub use ::arch::arm::mem::PAGE_SHIFT;

pub use ::arch::arm::cpu::set_interrupts;
pub use ::arch::arm::cpu::get_interrupts;
pub use ::arch::arm::cpu::wait_for_interrupts;
pub use ::arch::arm::cpu::memory_write_barrier;
pub use ::arch::arm::cpu::memory_read_barrier;

pub type Context = ::arch::arm::vector::InterruptContext;
pub type ThreadContext = ::arch::arm::thread::Context;
pub use ::arch::arm::thread::new_thread;
pub use ::arch::arm::thread::switch_context;
pub use ::arch::arm::ticks_in_second;
pub use ::arch::arm::cpu::get_current_cpu_id;

pub type ArchPlatformServices = ::arch::arm::PlatformServices;

pub use ::arch::arm::write_to_console;
