use core::intrinsics::{volatile_load, volatile_store};

pub const MBOX0_BASE_PADDR: ::mem::PhysicalAddress = ::mem::PhysicalAddress(super::MMIO_PSTART.0 + 0xb880);

pub const MBOX_READ_OFFSET: usize = 0x0;
pub const MBOX_WRITE_OFFSET:usize = 0x20;
pub const MBOX_STATUS_OFFSET :usize = 0x18;
pub const MAILBOX_FULL: u32 = 0x80000000;
pub const MAILBOX_EMPTY: u32 = 0x80000000;

/* VC core mailbox tocome..
pub struct Mailbox {
    base: *mut u8,
}


impl Mailbox {
    pub fn new(base: ::mem::VirtualAddress) -> Self {
        Mailbox { base: base.0 as *mut u8 }
    }
    pub fn new_bare() -> Self {
        Mailbox { base: MBOX0_BASE_PADDR.0 as *mut u8 }
    }

    fn write(&mut self, channel: u32, data: u32) {
        let ptr = (self.base as usize + MBOX_WRITE_OFFSET) as *mut u32;
        unsafe {
            volatile_store(ptr, data | channel);
        }
    }
    fn read(&mut self) -> (u32,  u32){
        let ptr: *const u32 = (self.base as usize + MBOX_READ_OFFSET) as *const u32;
        let val = unsafe { volatile_load(ptr) } ;

        return (val & 0xF, (val & (!0xF)))

    }

    fn is_empty(&self) -> bool {
        let ptr: *const u32 = (self.base as usize + MBOX_STATUS_OFFSET) as *const u32;
        return (unsafe { volatile_load(ptr) } & MAILBOX_EMPTY) != 0;
    }
    fn is_full(&self) -> bool {
        let ptr: *const u32 = (self.base as usize + MBOX_STATUS_OFFSET) as *const u32;
        return (unsafe { volatile_load(ptr) } & MAILBOX_FULL) != 0;
    }
}

*/

pub struct LocalMailbox {
    pub mailboxes: [CpuLocalMailbox; 4],
}

const LOCAL_MBOX_ADDR : ::mem::PhysicalAddress = ::mem::PhysicalAddress(super::ARM_LOCAL_PSTART.0 + 0x80);

impl LocalMailbox {
    pub fn new() -> Self {
        let base = ::platform::get_platform_services().mem_manager.p2v(LOCAL_MBOX_ADDR).unwrap();
        LocalMailbox { 
            mailboxes: [CpuLocalMailbox::new(base.uoffset(0x10*0)),
                        CpuLocalMailbox::new(base.uoffset(0x10*1)),
                        CpuLocalMailbox::new(base.uoffset(0x10*2)),
                        CpuLocalMailbox::new(base.uoffset(0x10*3))],
        }
    }
}

pub enum MailboxIndex {
    MailboxZero,
    MailboxOne,
    MailboxTwo,
    MailboxThree,
}

pub struct CpuLocalMailbox {
    write : ::mem::VirtualAddress,
    read  : ::mem::VirtualAddress,
}

impl CpuLocalMailbox {

    pub fn new(base: ::mem::VirtualAddress) -> Self {
        CpuLocalMailbox {
            write :  base,
            read : base.uoffset(0x40),
        }
    }

    pub fn set_high(&self, num : MailboxIndex, data : u32) {
        let num = num as usize;
        let ptr: *mut u32 = self.write.uoffset(num*4).0 as *mut u32;
        unsafe { volatile_store(ptr, data) };
    }

    pub fn read(&self, num : MailboxIndex) -> u32{
        let num = num as usize;
        let ptr: *mut u32 = self.read.uoffset(num*4).0  as *mut u32;
        unsafe { volatile_load(ptr) }
    }

    pub fn set_low(&self, num : MailboxIndex, data : u32) {
        let num = num as usize;
        let ptr: *mut u32 = self.read.uoffset(num*4).0  as *mut u32;
        unsafe { volatile_store(ptr, data) };
    }
}