use core::cell::RefCell;
use core::ops::Range;
use collections::boxed::Box;
use platform;
use volatile;
use super::super::super::pic;

// section 3.6 in: http://infocenter.arm.com/help/topic/com.arm.doc.dui0159b/DUI0159B_integratorcp_1_0_ug.pdf
pub const PIC_BASE_VADDR: ::mem::VirtualAddress = super::MMIO_VSTART.uoffset(0xB000);
pub const PIC_IRQ_BASE_VADDR: ::mem::VirtualAddress = PIC_BASE_VADDR.uoffset(0x200);


pub enum Interrupts {
    AUX =            29,
    I2C_SPI_SLV =    43,
    PWA0 =           45,
    PWA1 =           46,
    SMI =            48,
    GPIO_INT0 =      49,
    GPIO_INT1 =      50,
    GPIO_INT2 =      51,
    GPIO_INT3 =      52,
    I2C =            53,
    SPI =            54,
    PCM =            55,
    UART =           57,
}

bitflags! {
    #[repr(C,packed)] flags PicFlags1: u32 {
        const UNKNOWN0 =        1 << 0,
        const UNKNOWN1 =        1 << 1,
        const UNKNOWN2 =        1 << 2,
        const UNKNOWN3 =        1 << 3,
        const UNKNOWN4 =        1 << 4,
        const UNKNOWN5 =        1 << 5,
        const UNKNOWN6 =        1 << 6,
        const UNKNOWN7 =        1 << 7,
        const UNKNOWN8 =        1 << 8,
        const UNKNOWN9 =        1 << 9,
        const UNKNOWN10 =        1 << 10,
        const UNKNOWN11 =        1 << 11,
        const UNKNOWN12 =        1 << 12,
        const UNKNOWN13 =        1 << 13,
        const UNKNOWN14 =        1 << 14,
        const UNKNOWN15 =        1 << 15,
        const UNKNOWN16 =        1 << 16,
        const UNKNOWN17 =        1 << 17,
        const UNKNOWN18 =        1 << 18,
        const UNKNOWN19 =        1 << 19,
        const UNKNOWN20 =        1 << 20,
        const UNKNOWN21 =        1 << 21,
        const UNKNOWN22 =        1 << 22,
        const UNKNOWN23 =        1 << 23,
        const UNKNOWN24 =        1 << 24,
        const UNKNOWN25 =        1 << 25,
        const UNKNOWN26 =        1 << 26,
        const UNKNOWN27 =        1 << 27,
        const UNKNOWN28 =        1 << 28,
        const AUX =            1 << (Interrupts::AUX as usize),
        const UNKNOWN30 =        1 << 30,
        const UNKNOWN31 =        1 << 31,
    }
}
bitflags! {
    #[repr(C,packed)] flags PicFlags2: u32 {
    const UNKNOWN32 =        1 << (32 - 32),
    const UNKNOWN33 =        1 << (33 - 32),
    const UNKNOWN34 =        1 << (34 - 32),
    const UNKNOWN35 =        1 << (35 - 32),
    const UNKNOWN36 =        1 << (36 - 32),
    const UNKNOWN37 =        1 << (37 - 32),
    const UNKNOWN38 =        1 << (38 - 32),
    const UNKNOWN39 =        1 << (39 - 32),
    const UNKNOWN40 =        1 << (40 - 32),
    const UNKNOWN41 =        1 << (41 - 32),
    const UNKNOWN42 =        1 << (42 - 32),
    const I2C_SPI_SLV =    1 << ((Interrupts::I2C_SPI_SLV as usize) - 32),
    const UNKNOWN44 =        1 << (44 - 32),
    const PWA0 =           1 << ((Interrupts::PWA0 as usize)- 32),
    const PWA1 =           1 << ((Interrupts::PWA1 as usize)- 32),
    const UNKNOWN47 =        1 << (47 - 32),
    const SMI =            1 << ((Interrupts::SMI as usize)- 32),
    const GPIO_INT0 =      1 << ((Interrupts::GPIO_INT0 as usize)- 32),
    const GPIO_INT1 =      1 << ((Interrupts::GPIO_INT1 as usize)- 32),
    const GPIO_INT2 =      1 << ((Interrupts::GPIO_INT2 as usize)- 32),
    const GPIO_INT3 =      1 << ((Interrupts::GPIO_INT3 as usize)- 32),
    const I2C =            1 << ((Interrupts::I2C as usize)- 32),
    const SPI =            1 << ((Interrupts::SPI as usize)- 32),
    const PCM =            1 << ((Interrupts::PCM as usize)- 32),
    const UNKNOWN56 =        1 << (56 - 32),
    const UART =           1 << ((Interrupts::UART as usize)- 32),
    const UNKNOWN58 =        1 << (58 - 32),
    const UNKNOWN59 =        1 << (59 - 32),
    const UNKNOWN60 =        1 << (60 - 32),
    const UNKNOWN61 =        1 << (61 - 32),
    const UNKNOWN62 =        1 << (62 - 32),
    const UNKNOWN63 =        1 << (63 - 32),
    }
}


bitflags! {
    #[repr(C,packed)] flags PicFlagsBasic: u32 {
        const ARM_TIMER =        1 << 0,
        const ARM_MAILBOX =        1 << 1,
        const ARM_DOORBELL0 =        1 << 2,
        const ARM_DOORBELL1 =        1 << 3,
        const GPU0_HALT =        1 << 4,
        const GPU1_HALT =        1 << 5,
        const ILLEGAL0 =        1 << 6,
        const ILLEGAL1 =        1 << 7,
        const REG1_SET =        1 << 8,
        const REG2_SET =        1 << 9,
        const GPU_IRQ7 =        1 << 10,
        const GPU_IRQ9 =        1 << 11,
        const GPU_IRQ10 =        1 << 12,
        const GPU_IRQ18 =        1 << 13,
        const GPU_IRQ19 =        1 << 14,
        const GPU_IRQ53 =        1 << 15,
        const GPU_IRQ54 =        1 << 16,
        const GPU_IRQ55 =        1 << 17,
        const GPU_IRQ56 =        1 << 18,
        const GPU_IRQ57 =        1 << 19,
    }
}

#[repr(C,packed)]
pub struct PIC  {
    irq_basicpending    : volatile::ReadOnly<PicFlagsBasic>,
    irq_pending1        : volatile::ReadOnly<PicFlags1>,
    irq_pending2        : volatile::ReadOnly<PicFlags2>,
    fiq_control         : volatile::Volatile<u32>,
    enable_irqs1        : volatile::WriteOnly<PicFlags1>,
    enable_irqs2        : volatile::WriteOnly<PicFlags2>,
    enable_basic_irqs   : volatile::WriteOnly<PicFlagsBasic>,
    disable_irqs1       : volatile::WriteOnly<PicFlags1>,
    disable_irqs2       : volatile::WriteOnly<PicFlags2>,
    disable_basic_irqs  : volatile::WriteOnly<PicFlagsBasic>,
}


impl PIC { 
    pub unsafe fn new() -> &'static mut Self {
        &mut *(PIC_IRQ_BASE_VADDR.0 as *mut PIC)
    }
}

pub struct PICDev {
    pic : RefCell<&'static mut PIC>
}

impl PICDev {
    pub fn new() -> Self {
        unsafe {
            PICDev {
                // this will only be called from one cpu, so no need for cpu mutex. refcell is enough.
                pic : RefCell::new(PIC::new())
            }
        }
    }

    fn split_intr(intr_num : usize) -> (PicFlagsBasic, PicFlags1, PicFlags2) {
        if intr_num >= 64 {
            (PicFlagsBasic::empty(), PicFlags1::empty(), PicFlags2::from_bits_truncate(1 << (intr_num - 64)))
       } else if intr_num >= 32 {
            ( PicFlagsBasic::empty(), PicFlags1::empty(), PicFlags2::from_bits_truncate(1 << (intr_num - 32)))
        } else {
            (PicFlagsBasic::from_bits_truncate(1 << intr_num), PicFlags1::empty(),  PicFlags2::empty())            
        }
    }
}

impl pic::InterruptSource for PICDev {

    fn len(&self) -> usize {
         32*3
    }

    fn enable(&self, interrupt : usize) {
        let (flags_basic, flags1, flags2)  = Self::split_intr(interrupt);
        let mut pic = self.pic.borrow_mut();
        pic.enable_basic_irqs.write(flags_basic);
        pic.enable_irqs1.write(flags1);
        pic.enable_irqs2.write(flags2);

    }

    fn disable(&self, interrupt : usize) {
        let (flags_basic, flags1, flags2)  = Self::split_intr(interrupt);
        let mut pic = self.pic.borrow_mut();
        pic.disable_basic_irqs.write(flags_basic);
        pic.disable_irqs1.write(flags1);
        pic.disable_irqs2.write(flags2);
    }
    
    fn is_interrupted(&self, interrupt : usize) -> bool {
        let (flags_basic, flags1, flags2)  = Self::split_intr(interrupt);
        let pic = self.pic.borrow();
        let interrupts_basic = pic.irq_basicpending.read();
        let interrupts1 = pic.irq_pending1.read();
        let interrupts2 = pic.irq_pending2.read();

        ! ( 
            (interrupts_basic & flags_basic).is_empty() && 
            (interrupts1 & flags1).is_empty() && 
            (interrupts2 & flags2).is_empty())

    }
    
}
