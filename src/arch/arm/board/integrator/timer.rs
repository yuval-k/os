use core::intrinsics::volatile_store;
use platform;
use alloc::rc::Rc;
use collections::boxed::Box;

// section 4.9.2 in: http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf

pub const TIMERS_BASE: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1300_0000);
pub const TIMER_BASE_OFFSET: usize = 0x100;

pub const TIMER_LOAD_OFFSET: usize = 0x00;
pub const TIMER_VALUE_OFFSET: usize = 0x04;
pub const TIMER_CNTRL_OFFSET: usize = 0x08;
pub const TIMER_INTCLR_OFFSET: usize = 0x0C;
pub const TIMER_RIS_OFFSET: usize = 0x10;
pub const TIMER_MIS_OFFSET: usize = 0x14;
pub const TIMER_BG_LOAD_OFFSET: usize = 0x18;

bitflags! {
    pub flags TimerControlFlags: u32 {
        const ONE_SHOT_COUNTER = 1 << 0,
        const TIMER_SIZE_32    = 1 << 1,
        const PRESCALE1        = 1 << 2,
        const PRESCALE2        = 1 << 3,
        const R                = 1 << 4,
        const INT_EN           = 1 << 5,
        const PERIODIC         = 1 << 6,
        const ENABLE           = 1 << 7,
    }
}

pub struct Timer {
    base: ::mem::VirtualAddress, // this should be mapped to TIMERS_BASE
    callback:  Box<Fn()>,
}


impl Timer {
    pub fn new( index: usize,
                timerbase: ::mem::VirtualAddress,
                callback: Box<Fn()>)
               -> Timer {
        Timer {
            base: timerbase.uoffset(index * TIMER_BASE_OFFSET),
            callback: callback,
        }
    }

    pub fn start_timer(&mut self, counter: u32, intr: bool) {
        set_value(self.base.uoffset(TIMER_LOAD_OFFSET), counter);
        set_value(self.base.uoffset(TIMER_BG_LOAD_OFFSET), counter);
        let mut flags = ENABLE | PERIODIC | TIMER_SIZE_32;
        if intr {
            flags = flags | INT_EN;
        }
        set_value(self.base.uoffset(TIMER_CNTRL_OFFSET), flags.bits);
        self.clear_interrupt();
    }

    pub fn clear_interrupt(&self) {
        set_value(self.base.uoffset(TIMER_INTCLR_OFFSET), 1);
    }
}

impl platform::Interruptable for Timer {
    fn interrupted(&self) {
        self.clear_interrupt();
        (self.callback)();
    }
}

fn set_value(va: ::mem::VirtualAddress, v: u32) {
    let ptr: *mut u32 = va.0 as *mut u32;
    unsafe {
        volatile_store(ptr, v);
    }
}

// register
//

// fn timer_isr(ctx : & thread::Context) -> Option<thread::Context> {
// clear the interrupt
// clear_interrupt0();
//
// return service routine
//
// None
// }
//
