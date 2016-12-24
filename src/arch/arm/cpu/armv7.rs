
#[inline(always)]
pub fn data_memory_barrier() {
    unsafe {
        asm!("dmb":::"memory":"volatile")
    }
}
/*
macro_rules! data_memory_barrier {
    () => {{
        unsafe{asm!("dmb");}
    }}
}
*/
#[inline(always)]
pub fn data_synchronization_barrier() {
    unsafe {
        asm!("dsb":::"memory":"volatile")
    }
}

#[inline(always)]
pub fn get_current_cpu_id() -> usize {
    let mut mpidr: u32;
    unsafe {
        asm!("mrc p15, 0, $0, c0, c0, 5" :  "=r" (mpidr));
    }
    return (mpidr & 0b111) as usize;
}

// thanks https://www.raspberrypi.org/forums/viewtopic.php?f=72&t=11183
// see http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0464d/BABIEBAC.html
// http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0438g/CDEDBHDD.html
#[inline(always)]
pub fn enable_fpu() {
    unsafe {
        asm!("
            mrc p15, 0, r0, c1, c0, 2
            orr r0, r0, #0xF00000       /* Enable access to CP10 and CP11 */
            mcr p15, 0, r0, c1, c0, 2
            isb
            mov r0, #0x40000000
            fmxr fpexc, r0
            " :::"r0":"volatile");
    }
}

#[inline(never)]
#[naked]
pub fn wait_for_interrupts() {
    unsafe {
        asm!("loop:
            wfi
            b loop
            "::::"volatile"
            )
    }
}