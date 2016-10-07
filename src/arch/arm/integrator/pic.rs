use core::intrinsics::{volatile_load, volatile_store};

// section 3.6 in: http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf
pub const PIC_BASE_VADDR : ::mem::VirtualAddress  = ::mem::VirtualAddress(0x1400_0000);
pub const PIC_BASE_PADDR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1400_0000);

pub const PIC_IRQ_STATUS : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x14000000);
pub const PIC_IRQ_RAWSTAT : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x14000004);
pub const PIC_IRQ_ENABLESET : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x14000008);
pub const PIC_IRQ_ENABLECLR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1400000C);
pub const PIC_INT_SOFTSET : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x14000010);
pub const PIC_INT_SOFTCLR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x14000014);

pub const PIC_FIQ_STATUS : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x14000020);
pub const PIC_FIQ_RAWSTAT : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x14000024);
pub const PIC_FIQ_ENABLESET : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x14000028);
pub const PIC_FIQ_ENABLECLR : ::mem::PhysicalAddress  = ::mem::PhysicalAddress(0x1400002C);

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


pub fn enable_interrupts(flags : PicFlags) {
    let ptr : *mut u32 = ((PIC_IRQ_ENABLESET.0 - PIC_BASE_PADDR.0) + PIC_BASE_VADDR.0) as *mut u32;
    unsafe {volatile_store(ptr, flags.bits);}
}

pub fn clear_interrupts(flags : PicFlags) {
    let ptr : *mut u32 = ((PIC_IRQ_ENABLECLR.0 - PIC_BASE_PADDR.0) + PIC_BASE_VADDR.0) as *mut u32;
    unsafe {volatile_store(ptr, flags.bits);}
}

pub fn interrupt_status() ->  PicFlags{
    let mut flags : PicFlags = PicFlags::empty();
    let ptr : *mut u32 = ((PIC_IRQ_STATUS.0 - PIC_BASE_PADDR.0) + PIC_BASE_VADDR.0) as *mut u32;
    flags.bits = unsafe {volatile_load(ptr)};
    
    flags
}
