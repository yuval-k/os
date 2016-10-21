pub mod intr;

#[cfg(target_arch = "arm")]
mod arm;
#[cfg(target_arch = "arm")]
pub use self::arm::*;

pub const PAGE_SIZE : usize = 1<<PAGE_SHIFT;
pub const PAGE_MASK : usize = PAGE_SIZE - 1;

pub trait InterruptSource {
    fn interrupted(&mut self, &mut Context);
}