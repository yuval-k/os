use core::intrinsics::{volatile_load, volatile_store};
use collections::boxed::Box;
use platform;

// section 3.6 in: http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf
pub const PIC_BASE_PADDR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1400_0000);

pub const PIC_IRQ_STATUS_OFFSET : usize = 0x00;
pub const PIC_IRQ_RAWSTAT_OFFSET : usize = 0x04;
pub const PIC_IRQ_ENABLESET_OFFSET : usize = 0x08;
pub const PIC_IRQ_ENABLECLR_OFFSET : usize = 0x0C;
pub const PIC_INT_SOFTSET_OFFSET : usize = 0x10;
pub const PIC_INT_SOFTCLR_OFFSET : usize = 0x14;
pub const PIC_FIQ_STATUS_OFFSET : usize = 0x20;
pub const PIC_FIQ_RAWSTAT_OFFSET : usize = 0x24;
pub const PIC_FIQ_ENABLESET_OFFSET : usize = 0x28;
pub const PIC_FIQ_ENABLECLR_OFFSET : usize = 0x2C;

bitflags! {
    pub flags PicFlags: u32 {
        const SOFTINT       =  1 << 0,
        const UARTINT0       = 1 << 1,
        const UARTINT1       = 1 << 2,
        const KBDINT         = 1 << 3,
        const MOUSEINT       = 1 << 4,
        const TIMERINT0      = 1 << 5,
        const TIMERINT1      = 1 << 6,
        const TIMERINT2      = 1 << 7,
        const RTCINT         = 1 << 8,
        const LM_LLINT0      = 1 << 9,
        const LM_LLINT1      = 1 << 10,
        const CLCDCINT       = 1 << 22,
        const MMCIINT0       = 1 << 23,
        const MMCIINT1       = 1 << 24,
        const AACIINT        = 1 << 25,
        const CPPLDINT       = 1 << 26,
        const ETH_INT        = 1 << 27,
        const TS_PENINT      = 1 << 28,
    }
}

pub struct PIC {
    vbase : ::mem::VirtualAddress,
    callback : Option<Box<platform::InterruptSource>>
}

impl PIC {

    pub fn new(vbase : ::mem::VirtualAddress) -> PIC {
        PIC {
            vbase : vbase,
            callback : None,
        }
    }

    pub fn add_timer_callback(&mut self, callback : Box<platform::InterruptSource> ) {
        self.callback = Some(callback);
    }

    pub fn enable_interrupts(&mut self, flags : PicFlags) {
        let ptr : *mut u32 = self.vbase.uoffset(PIC_IRQ_ENABLESET_OFFSET).0 as *mut u32;
        unsafe {volatile_store(ptr, flags.bits);}
    }

    pub fn clear_interrupts(&mut self, flags : PicFlags) {
        let ptr : *mut u32 = self.vbase.uoffset(PIC_IRQ_ENABLECLR_OFFSET).0 as *mut u32;
        unsafe {volatile_store(ptr, flags.bits);}
    }

    pub fn interrupt_status(&self) ->  PicFlags {
        let mut flags : PicFlags = PicFlags::empty();
        let ptr : *mut u32 = self.vbase.uoffset(PIC_IRQ_STATUS_OFFSET).0 as *mut u32;
        flags.bits = unsafe {volatile_load(ptr)};
        
        flags
    }

}

impl platform::InterruptSource for PIC {
    fn interrupted(& self, ctx : &mut platform::Context) {
        let status = self.interrupt_status();

        if status.contains(TIMERINT0) {
            if let Some(ref callback) = self.callback {
                callback.interrupted(ctx);
            }
        }
         
        // TODO switch back to main thread to deal with this...
        // let it know what interrupt happened
        // once we have semaphores or some other way of sync objects.


    }
}
