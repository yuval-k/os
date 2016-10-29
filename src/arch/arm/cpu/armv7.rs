
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