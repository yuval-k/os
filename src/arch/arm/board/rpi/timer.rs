use ::platform;
use volatile;

use arch::arm::Driver;
use arch::arm::DriverHandle;
use arch::arm::InterruptableDriver;
use core::cell::RefCell;

const SYS_TIMER_BASE_VADDR: ::mem::VirtualAddress = super::MMIO_VSTART.uoffset(0x3000);

const TIMER_HZ : u32 = 1000_000;

pub enum Matches {
// can't use timers 0 and 2 as they are reservered for GPU
//	Match0,
	Match1 = 1,
//	Match2,
	Match3 = 3,
}

bitflags! {
    #[repr(C,packed)] pub flags ControlStatusFlags: u32 {
        const M0 = 1 << (0),
        const M1 = 1 << (Matches::Match1 as u32),
        const M2 = 1 << (2),
        const M3 = 1 << (Matches::Match3 as u32),
    }
}

fn matchToFlag(m : Matches) -> ControlStatusFlags {
    match m {
   //     Matches::Match0 => M0,
        Matches::Match1 => M1,
   //     Matches::Match2 => M2,
        Matches::Match3 => M3,
    }
} 

// see here: http://infocenter.arm.com/help/topic/com.arm.doc.ddi0183f/DDI0183.pdf section 3.2
#[repr(C,packed)]
pub struct SystemTimer {
    pub control_status: volatile::Volatile<ControlStatusFlags>,
    pub counter_low: volatile::Volatile<u32>,
    pub counter_high: volatile::Volatile<u32>,
    pub compares: [volatile::Volatile<u32>; 4]
}

impl SystemTimer {
    pub unsafe fn new() -> &'static mut Self {
 		&mut *(SYS_TIMER_BASE_VADDR.0 as *mut SystemTimer)
    }

	pub fn clear_match(&mut self, m : Matches) {
		self.control_status.update(|cs|{cs.insert(matchToFlag(m)) })
	}

	pub fn add_to_match(&mut self, m : Matches, v : u32) {
		self.compares[m as usize].write(self.counter_low.read().wrapping_add(v) & 0xF);
	}
	
	pub fn set_match(&mut self, m : Matches, v : u32) {
		self.compares[m as usize].write(v);
	}
}


pub struct SystemTimerDriver {
    timer : RefCell<&'static mut SystemTimer>,
}

impl SystemTimerDriver {
    pub fn new() -> Self {
        SystemTimerDriver {
            timer : RefCell::new(unsafe{SystemTimer::new()})
        }
    }

	pub fn clear(&self) {
        self.timer.borrow_mut().clear_match(Matches::Match3)
	}

	pub fn add_to_match(&self, v : u32) {
        self.timer.borrow_mut().add_to_match(Matches::Match3, v)
	}
	
	pub fn set_match(&self, v : u32) {
        self.timer.borrow_mut().set_match(Matches::Match3, v)
	}
}

impl InterruptableDriver for SystemTimerDriver {}
impl Driver for SystemTimerDriver {
    fn attach(&mut self, dh : DriverHandle) {
        platform::get_platform_services().arch_services.interrupt_service.register_callback_on_intr(super::intr::Interrupts::TIMER3 as usize, dh);

        let curcounter = {self.timer.borrow().counter_low.read()};
        self.set_match(curcounter+100_000);
    }
}
impl platform::Interruptable for SystemTimerDriver {
    fn interrupted(&self) {
        // 100ms
        self.add_to_match(100_000);
        self.clear();
    }
}