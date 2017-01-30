use core::intrinsics::{volatile_load, volatile_store};

use super::super::super::cpu;
use ::platform;

const CORE0_TIMER_IRQCNTL : ::mem::PhysicalAddress = ::mem::PhysicalAddress(super::ARM_LOCAL_PSTART.0 + 0x40);
const CORE0_IRQ_SOURCE : ::mem::PhysicalAddress = ::mem::PhysicalAddress(super::ARM_LOCAL_PSTART.0 + 0x60);

const TIMER_CONTROL_ENABLE  : u32 = 1 << 0;
const TIMER_CONTROL_IMASK   : u32 = 1 << 1;
const TIMER_CONTROL_ISTATUS : u32 = 1 << 2;

pub struct GlobalTimer {
	time : u32
}

impl GlobalTimer {

	pub fn start_timer(&self) {
		cpu::write_cntv_tval(self.time);

		cpu::write_cntv_ctl(TIMER_CONTROL_ENABLE);
	//	cpu::write_cntp_ctl(TIMER_CONTROL_ENABLE);
	}
}

impl platform::Interruptable for GlobalTimer {
    fn interrupted(&self, ctx: &mut platform::Context) {
		cpu::write_cntv_tval(self.time);
		::platform::get_platform_services().clock();
	}
}