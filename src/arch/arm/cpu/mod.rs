#[cfg(feature = "armv6")]
pub mod armv6;
#[cfg(feature = "armv6")]
pub use self::armv6::*;

#[cfg(feature = "armv7")]
pub mod armv7;
#[cfg(feature = "armv7")]
pub use self::armv7::*;


pub const USER_MODE: u32 = 0b10000;
pub const FIQ_MODE: u32 = 0b10001;
pub const IRQ_MODE: u32 = 0b10010;
pub const SUPER_MODE: u32 = 0b10011;
pub const ABRT_MODE: u32 = 0b10111;
pub const UNDEF_MODE: u32 = 0b11011;
pub const SYS_MODE: u32 = 0b11111;

pub const MODE_MASK: u32 = 0b11111;

pub const DISABLE_FIQ: u32 = 1 << 6;
pub const DISABLE_IRQ: u32 = 1 << 7;


// #[inline(always)] -> cause these might be used in the stub (the rest of program code will be mapped later)

#[inline(always)]
pub fn memory_write_barrier() {
    // take cafre o memory ordering 
    data_memory_barrier();
    // make sure data was written to memory
    data_synchronization_barrier();
}
#[inline(always)]
pub fn memory_read_barrier() {
    data_memory_barrier();
    data_synchronization_barrier();
}

#[inline(always)]
pub fn flush_caches() {

    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0360e/I1014942.html
    // clean and Invalidate Both Caches. Also flushes the branch target cache
    // first instruction cleansd the cache, the second one invalidates it.
    unsafe {
        asm!("
        mov r0, #0
        mcr	p15, 0, r0, c7, c5, 0
        mcr	p15, 0, r0, c7, c14, 0
        mcr	p15, 0, r0, c7, c10, 4
        mcr p15, 0, $0, c7, c7, 0"  ::"r"(0)::"volatile")
    }
}

#[inline(always)]
pub fn invalidate_tlb() {

    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0344k/I1001599.html
    // Invalidate Inst-TLB and Data-TLB
    unsafe {
        asm!("mcr p15, 0, $0, c8, c7, 0"  ::"r"(0)::"volatile")
    }
}

#[inline(always)]
pub fn set_ttb0(page_table: *const ()) {
    // Set Translation Table Base 0 (TTB0)
    unsafe {
        asm!("mcr p15, 0, $0, c2, c0, 0"
          :: "r"(page_table as u32) :: "volatile");
    }
}

#[inline(always)]
pub fn get_ttb0() -> *const () {
    let mut ttb0: u32;
    unsafe {
        asm!("mrc p15, 0, $0, c2, c0, 0":  "=r"(ttb0));
    }
    return ttb0 as *const ();
}

#[inline(always)]
pub fn set_ttb1(page_table: *const ()) {
    // Set Translation Table Base 0 (TTB0)
    unsafe {
        asm!("mcr p15, 0, $0, c2, c0, 1"
          :: "r"(page_table as u32) :: "volatile");

    }
}

#[inline(always)]
pub fn set_ttbcr(ttbcr: u32) {
    unsafe {
        asm!("mcr p15, 0, $0, c2, c0, 2" : : "r" (ttbcr):: "volatile");
    }
}

#[inline(always)]
#[allow(unused_mut)]
pub fn get_ttbcr() -> u32 {
    let mut ttbcr: u32;
    unsafe {
        asm!("mrc p15, 0, $0, c2, c0, 2" :  "=r" (ttbcr));
    }
    return ttbcr;
}


#[inline(always)]
pub fn write_domain_access_control_register(dcr: u32) {
    unsafe {
        asm!("mcr p15, 0, $0, c3, c0, 0" :: "r"(dcr) :: "volatile");
    }
}


// c1 register controls the mmu
#[inline(always)]
#[allow(unused_mut)]
fn get_p15_c1() -> u32 {
    let mut cr: u32;
    unsafe {
        asm!("mcr p15, 0, $0, c1, c0, 0" : "=r"(cr));
    }
    return cr;
}

#[inline(always)]

fn set_system_control_register(cr: u32) {
    unsafe {
        asm!("mcr p15, 0, $0, c1, c0, 0" :: "r"(cr) :: "volatile");
    }
}

const MMU_BIT: u32 = 1;
const DCACHE_BIT: u32 = 1 << 2;
const ICACHE_BIT: u32 = 1 << 12;
const XP_BIT: u32 = 1 << 23;

#[inline(always)]
pub fn enable_mmu() {
    let mut cr: u32;
    cr = get_p15_c1();

    cr |= MMU_BIT;
    cr |= DCACHE_BIT;
    cr |= ICACHE_BIT;
    // extended page tables
    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0290g/Babhejba.html
    // and
    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0360f/BGEIHGIF.html
    cr |= XP_BIT;

    set_system_control_register(cr);
}


// not called from stub goes here:


pub fn set_stack_for_mode(mode: u32, stack_base: ::mem::VirtualAddress) {

    unsafe {
        asm!("
            /* change cpu mode */
            mov r0, $0
            mov r2, $2
            mrs r1, cpsr
            bic r1, r1, $1
	        orr r1, r1, r2
            msr cpsr_c, r1
            /* set stack */
            mov sp, r0
            /* back to supervisor mode */
            bic r1, r1, $1
            orr r1, r1, $3
            msr cpsr_c, r1
            "
            :
            :
            "r"(stack_base.0),
            "i"(MODE_MASK),
            "r"(mode),
            "i"(SUPER_MODE)
            : "sp","r0","r1","cpsr" : "volatile"
            )
    };
}

pub fn disable_interrupts() {
    unsafe {
        asm!("mrs r0, cpsr
            orr r0, r0, $0
            msr cpsr_c, r0            
            "
            :: 
            "i"(DISABLE_FIQ | DISABLE_IRQ)
            : "r0", "cpsr" : "volatile"
        )
    }
}

pub fn enable_interrupts() {
    unsafe {
        asm!("mrs r0, cpsr
            bic r0, r0, $0
            msr cpsr_c, r0            
            "
            :: 
            "i"(DISABLE_FIQ | DISABLE_IRQ)
            : "r0", "cpsr" : "volatile"
        )
    }
}

pub fn set_interrupts(b: bool) {
    if b {
        enable_interrupts();
    } else {
        disable_interrupts();
    }
}

pub fn get_interrupts() -> bool {
    (get_cpsr() & (DISABLE_IRQ | DISABLE_IRQ)) == 0
}


#[allow(unused_mut)]
pub fn get_cpsr() -> u32 {
    let mut cpsr: u32;
    unsafe {
        asm!("mrs $0, cpsr" : "=r"(cpsr));
    }
    return cpsr;
}

pub fn set_cpsr(cpsr: u32) {
    unsafe {
        asm!("msr cpsr, $0" :: "r"(cpsr) :: "volatile");
    }
}

#[allow(unused_mut)]
pub fn get_spsr() -> u32 {
    let mut spsr: u32;
    unsafe {
        asm!("mrs $0, spsr" : "=r"(spsr));
    }
    return spsr;
}

pub fn set_spsr(spsr: u32) {
    unsafe {
        asm!("msr spsr, $0" :: "r"(spsr) :: "volatile");
    }
}

#[allow(unused_mut)]
pub fn get_r13r14(spsr: u32) -> (u32, u32) {
    let cpsr = get_cpsr();
    // get the mode
    let frommode = cpsr & MODE_MASK;
    let mut tomode = MODE_MASK & spsr; // not needed..get_spsr() & MODE_MASK;

    if frommode == tomode {
        panic!("This should only be used to get regs from different mode.")
    }


    // if to mode is user mode, change to system mode
    if tomode == USER_MODE {
        tomode = SYS_MODE;
    }

    let tocpsr = (cpsr & !(MODE_MASK)) | tomode;

    let mut r13: u32;
    let mut r14: u32;

    unsafe {
        asm!("
        mov r0, $2
        mov r1, $3
        msr cpsr, r0
        mov r3, sp
        mov r4, lr
        msr cpsr, r1
        mov $0, r3
        mov $1, r4
        "
        : "=r"(r13), "=r"(r14): "r"(tocpsr) , "r"(cpsr) : "r0", "r1","r3", "r4" : "volatile"
        );
    }

    return (r13, r14);
}

pub fn set_r13r14(spsr: u32, r13: u32, r14: u32) {
    let cpsr = get_cpsr();
    // get the mode
    let frommode = cpsr & MODE_MASK;
    let mut tomode = MODE_MASK & spsr;

    if frommode == tomode {
        panic!("This should only be used to get regs from different mode.")
    }

    // if to mode is user mode, change to system mode
    if tomode == USER_MODE {
        tomode = SYS_MODE;
    }

    let tocpsr = (cpsr & !(MODE_MASK)) | tomode;

    unsafe {
        asm!("
        mov r3, $0
        mov r4, $1
        mov r0, $2
        mov r1, $3
        msr cpsr, r0
        mov sp, r3
        mov lr, r4
        msr cpsr, r1
        "
        :: "r"(r13), "r"(r14), "r"(tocpsr) , "r"(cpsr) : "r0", "r1","r3", "r4": "volatile"
        );
    }
}

pub fn set_vector_table(vector_table: u32) {
    unsafe {
        // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0204h/Cihfifej.html
        asm!("mcr p15, 0, $0, c12, c0, 0 "
          :: "r"(vector_table) :: "volatile"
          );
    }
}
