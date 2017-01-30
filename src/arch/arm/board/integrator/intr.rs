use core::intrinsics::{volatile_load, volatile_store};
use collections::boxed::Box;
use platform;
use super::super::super::pic;

// section 3.6 in: http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf
pub const PIC_BASE_PADDR: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1400_0000);

pub const PIC_IRQ_STATUS_OFFSET: usize = 0x00;
pub const PIC_IRQ_RAWSTAT_OFFSET: usize = 0x04;
pub const PIC_IRQ_ENABLESET_OFFSET: usize = 0x08;
pub const PIC_IRQ_ENABLECLR_OFFSET: usize = 0x0C;
pub const PIC_INT_SOFTSET_OFFSET: usize = 0x10;
pub const PIC_INT_SOFTCLR_OFFSET: usize = 0x14;
pub const PIC_FIQ_STATUS_OFFSET: usize = 0x20;
pub const PIC_FIQ_RAWSTAT_OFFSET: usize = 0x24;
pub const PIC_FIQ_ENABLESET_OFFSET: usize = 0x28;
pub const PIC_FIQ_ENABLECLR_OFFSET: usize = 0x2C;

pub enum Interrupts {
    SOFTINT,
    UARTINT0,
    UARTINT1,
    KBDINT,
    MOUSEINT,
    TIMERINT0,
    TIMERINT1,
    TIMERINT2,
    RTCINT,
    LM_LLINT0,
    LM_LLINT1,
    CLCDCINT = 22,
    MMCIINT0,
    MMCIINT1,
    AACIINT,
    CPPLDINT,
    ETH_INT,
    TS_PENINT,
}

bitflags! {
    flags PicFlags: u32 {
        const SOFTINT       =  1 << (Interrupts::SOFTINT as usize),
        const UARTINT0       = 1 << (Interrupts::UARTINT0 as usize),
        const UARTINT1       = 1 << (Interrupts::UARTINT1 as usize),
        const KBDINT         = 1 << (Interrupts::KBDINT as usize),
        const MOUSEINT       = 1 << (Interrupts::MOUSEINT as usize),
        const TIMERINT0      = 1 << (Interrupts::TIMERINT0 as usize),
        const TIMERINT1      = 1 << (Interrupts::TIMERINT1 as usize),
        const TIMERINT2      = 1 << (Interrupts::TIMERINT2 as usize),
        const RTCINT         = 1 << (Interrupts::RTCINT as usize),
        const LM_LLINT0      = 1 << (Interrupts::LM_LLINT0 as usize),
        const LM_LLINT1      = 1 << (Interrupts::LM_LLINT1 as usize),
        const CLCDCINT       = 1 << (Interrupts::CLCDCINT as usize),
        const MMCIINT0       = 1 << (Interrupts::MMCIINT0 as usize),
        const MMCIINT1       = 1 << (Interrupts::MMCIINT1 as usize),
        const AACIINT        = 1 << (Interrupts::AACIINT as usize),
        const CPPLDINT       = 1 << (Interrupts::CPPLDINT as usize),
        const ETH_INT        = 1 << (Interrupts::ETH_INT as usize),
        const TS_PENINT      = 1 << (Interrupts::TS_PENINT as usize),
    }
}

pub struct PIC {
    vbase: ::mem::VirtualAddress,
    callback: Option<Box<platform::Interruptable>>,
}

impl pic::InterruptSource for PIC {
    fn len(&self) -> usize {
        29
    }

    fn enable(&self, interrupt: usize) {
        // TODO: change to from_bits.unwrap to panic on errors?
        let flags: PicFlags = PicFlags::from_bits_truncate(1 << interrupt);
        self.enable_interrupts(flags);
    }

    fn disable(&self, interrupt: usize) {
        let flags: PicFlags = PicFlags::from_bits_truncate(1 << interrupt);
        self.clear_interrupts(flags);
    }

    fn is_interrupted(&self, interrupt: usize) -> bool {
        let flags: PicFlags = PicFlags::from_bits_truncate(1 << interrupt);

        // interrupt is not really an interrupt...  i should really fix this at some point..
        if flags.bits == 0 {
            return false;
        }

        let status = self.interrupt_status();

        status.contains(flags)
    }
}

impl PIC {
    pub fn new(vbase: ::mem::VirtualAddress) -> PIC {
        PIC {
            vbase: vbase,
            callback: None,
        }
    }

    fn enable_interrupts(&self, flags: PicFlags) {
        let ptr: *mut u32 = self.vbase.uoffset(PIC_IRQ_ENABLESET_OFFSET).0 as *mut u32;
        unsafe {
            volatile_store(ptr, flags.bits);
        }
    }

    fn clear_interrupts(&self, flags: PicFlags) {
        let ptr: *mut u32 = self.vbase.uoffset(PIC_IRQ_ENABLECLR_OFFSET).0 as *mut u32;
        unsafe {
            volatile_store(ptr, flags.bits);
        }
    }

    fn interrupt_status(&self) -> PicFlags {
        let mut flags: PicFlags = PicFlags::empty();
        let ptr: *mut u32 = self.vbase.uoffset(PIC_IRQ_STATUS_OFFSET).0 as *mut u32;
        flags.bits = unsafe { volatile_load(ptr) };

        flags
    }
}
