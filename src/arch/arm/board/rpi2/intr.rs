use core::intrinsics::{volatile_load, volatile_store};
use core::ops::Range;
use collections::boxed::Box;
use platform;
use super::super::super::pic;

// section 3.6 in: http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf
pub const PIC_BASE_PADDR: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x40000040);

pub enum Interrupts {
    CNTPSIRQ,
    CNTPNSIRQ,
    CNTHPIRQ,
    CNTVIRQ,
    Mailbox0,
    Mailbox1,
    Mailbox2,
    Mailbox3,
    GPU,
    PMU,
    AXI,
    LocalTimer,
}

bitflags! {
    flags CorePicFlags: u32 {
        const CNTPSIRQ      = 1 << (Interrupts::CNTPSIRQ as usize),
        const CNTPNSIRQ     = 1 << (Interrupts::CNTPNSIRQ as usize),
        const CNTHPIRQ      = 1 << (Interrupts::CNTHPIRQ as usize),
        const CNTVIRQ       = 1 << (Interrupts::CNTVIRQ as usize),
        const Mailbox0      = 1 << (Interrupts::Mailbox0 as usize),
        const Mailbox1      = 1 << (Interrupts::Mailbox1 as usize),
        const Mailbox2      = 1 << (Interrupts::Mailbox2 as usize),
        const Mailbox3      = 1 << (Interrupts::Mailbox3 as usize),
        const GPU           = 1 << (Interrupts::GPU as usize),
        const PMU           = 1 << (Interrupts::PMU as usize),
        const AXI           = 1 << (Interrupts::AXI as usize),
        const LocalTimer    = 1 << (Interrupts::LocalTimer as usize),
    }
}

// see here:
// https://www.raspberrypi.org/documentation/hardware/raspberrypi/bcm2836/QA7_rev3.4.pdf
// 4.10 Core interrupt sources
// offsets from PIC_BASE_PADDR
const TIMER_CONTROL_OFFSET : usize = 0;
const MAILBOX_CONTROL_OFFSET : usize = 0x10;
const INTR_SOURCE_OFFSET : usize = 0x20;

// vbase is on of the trimer control address, like 0x40000040 + cpu offset
pub struct CorePIC {
    vbase: ::mem::VirtualAddress,
}

impl pic::InterruptSource for CorePIC {
    fn len(&self) -> usize {
        // each core has 4 timers and 4 mailboxes
         8 
    }

    fn enable(&self, interrupt : usize) {
        // handle timers
        let intr : CorePicFlags = CorePicFlags::from_bits_truncate(1<<interrupt);
        let timers = intr & (CNTPSIRQ | CNTPNSIRQ | CNTHPIRQ | CNTVIRQ);
        if ! timers.is_empty() {
            let ptr: *mut u32 = self.vbase.uoffset(TIMER_CONTROL_OFFSET).0 as *mut u32;
            unsafe {
                let mut curstatus : u32 = unsafe { volatile_load(ptr) };
                curstatus |= timers.bits;
                volatile_store(ptr, curstatus);
            }
        }
        // handle mailboxes
        let msgboxs = intr & (Mailbox0 | Mailbox1 | Mailbox2 | Mailbox3);
        if ! msgboxs.is_empty() {
            let ptr: *mut u32 = self.vbase.uoffset(MAILBOX_CONTROL_OFFSET).0 as *mut u32;
            unsafe {
                let mut curstatus : u32 = unsafe { volatile_load(ptr) };
                curstatus |= msgboxs.bits >> 4;
                volatile_store(ptr, curstatus);
            }
        }
    }

    fn disable(&self, interrupt : usize) {
        let intr : CorePicFlags = CorePicFlags::from_bits_truncate(1<<interrupt);
        let timers = intr & (CNTPSIRQ | CNTPNSIRQ | CNTHPIRQ | CNTVIRQ);
        if ! timers.is_empty() {
            let ptr: *mut u32 = self.vbase.uoffset(TIMER_CONTROL_OFFSET).0 as *mut u32;
            unsafe {     
                let mut curstatus : u32 = unsafe { volatile_load(ptr) };
                curstatus &= !timers.bits;
                volatile_store(ptr, curstatus);
            }
        }
      let msgboxs = intr & (Mailbox0 | Mailbox1 | Mailbox2 | Mailbox3);
        if ! msgboxs.is_empty() {
            let ptr: *mut u32 = self.vbase.uoffset(MAILBOX_CONTROL_OFFSET).0 as *mut u32;
            unsafe {
                let mut curstatus : u32 = unsafe { volatile_load(ptr) };
                curstatus &= !(msgboxs.bits >> 4);
                volatile_store(ptr, curstatus);
            }
        }
    }
    
    fn is_interrupted(&self, interrupt : usize) -> bool {
        let intr : CorePicFlags = CorePicFlags::from_bits_truncate(1<<interrupt);
        
        let ptr: *mut u32 = self.vbase.uoffset(INTR_SOURCE_OFFSET).0 as *mut u32;
        let is : u32 = unsafe { volatile_load(ptr)};
        let intr_source =  CorePicFlags::from_bits_truncate(is);

        intr_source.contains(intr)
    }
    
}

impl CorePIC {
    pub fn new() -> Self {
        let cpuid = ::platform::get_current_cpu_id();
        Self::new_for_cpu(cpuid)
    }
    pub fn new_for_cpu(cpuid : usize) -> Self {
        let vbase = ::platform::get_platform_services().mem_manager.p2v(PIC_BASE_PADDR).unwrap();

        CorePIC {
            vbase: vbase.uoffset(4*cpuid),
        }
    }

}
