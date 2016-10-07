use core::intrinsics::{volatile_load, volatile_store};

pub const TIMER_BASE_VADDR : ::mem::VirtualAddress  = ::mem::VirtualAddress(0x1300_0000);
pub const TIMER_BASE_PADDR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1300_0000);

// section 4.9.2 in: http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf
pub const TIMER0_LOAD : ::mem::PhysicalAddress    = ::mem::PhysicalAddress(0x1300_0000);
pub const TIMER0_VALUE : ::mem::PhysicalAddress   = ::mem::PhysicalAddress(0x1300_0004);
pub const TIMER0_CNTRL : ::mem::PhysicalAddress   = ::mem::PhysicalAddress(0x1300_0008);
pub const TIMER0_INTCLR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1300_000C);
pub const TIMER0_RIS : ::mem::PhysicalAddress     = ::mem::PhysicalAddress(0x1300_0010);
pub const TIMER0_MIS : ::mem::PhysicalAddress     = ::mem::PhysicalAddress(0x1300_0014);
pub const TIMER0_BG_LOAD : ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1300_0018);

pub const TIMER1_LOAD : ::mem::PhysicalAddress    = ::mem::PhysicalAddress(0x1300_0100);
pub const TIMER1_VALUE : ::mem::PhysicalAddress   = ::mem::PhysicalAddress(0x1300_0104);
pub const TIMER1_CNTRL : ::mem::PhysicalAddress   = ::mem::PhysicalAddress(0x1300_0108);
pub const TIMER1_INTCLR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1300_010C);
pub const TIMER1_RIS : ::mem::PhysicalAddress     = ::mem::PhysicalAddress(0x1300_0110);
pub const TIMER1_MIS : ::mem::PhysicalAddress     = ::mem::PhysicalAddress(0x1300_0114);
pub const TIMER1_BG_LOAD : ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1300_0118);

pub const TIMER2_LOAD : ::mem::PhysicalAddress    = ::mem::PhysicalAddress(0x1300_0200);
pub const TIMER2_VALUE : ::mem::PhysicalAddress   = ::mem::PhysicalAddress(0x1300_0204);
pub const TIMER2_CNTRL : ::mem::PhysicalAddress   = ::mem::PhysicalAddress(0x1300_0208);
pub const TIMER2_INTCLR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1300_020C);
pub const TIMER2_RIS : ::mem::PhysicalAddress     = ::mem::PhysicalAddress(0x1300_0210);
pub const TIMER2_MIS : ::mem::PhysicalAddress     = ::mem::PhysicalAddress(0x1300_0214);
pub const TIMER2_BG_LOAD : ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1300_0218);

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


pub fn start_timer0() {
    set_value(TIMER0_LOAD, 0xffffff);
    set_value(TIMER0_BG_LOAD, 0xffffff);
    set_value(TIMER0_CNTRL, (ENABLE | INT_EN | PERIODIC | TIMER_SIZE_32).bits);
    clear_interrupt0(); // pancakes did it (http://wiki.osdev.org/ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch) and i love precautions. OS goals are educational not production.
}


pub fn clear_interrupt0() {
    set_value(TIMER0_INTCLR, 1);
}

fn set_value(p  : ::mem::PhysicalAddress, v : u32) {
    let ptr : *mut u32 = ((p.0 - TIMER_BASE_PADDR.0) + TIMER_BASE_VADDR.0) as *mut u32;
    unsafe {volatile_store(ptr, v);}
}
