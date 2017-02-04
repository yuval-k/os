
#[macro_export]
macro_rules! read_reg {
    ($line:expr) => {
        {
            let mut ret: u32;
            unsafe {
                asm!(concat!("mrc ", $line) :  "=r" (ret));
            }
            ret
        }
    };
}

#[macro_export]
macro_rules! write_reg {
    ($line:expr, $value:expr) => {
        {
            unsafe {
                asm!(concat!("mcr ", $line) : : "r" ($value):: "volatile");
            }
        }
    };
}

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
pub fn instruction_synchronization_barrier() {
    unsafe {
        asm!("isb":::"memory":"volatile")
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
        asm!("wfi"::::"volatile");
    }
}

pub fn read_cnt_frq() -> u32 { read_reg!("p15,0,$0,c14,c0,0") }
pub fn write_cnt_frq(i : u32){write_reg!("p15,0,$0,c14,c0,0", i)}

pub fn read_cntk_ctl() -> u32 { read_reg!("p15,0,$0,c14,c1,0") }
pub fn write_cntk_ctl(i : u32){write_reg!("p15,0,$0,c14,c1,0", i)}

pub fn read_cntp_tval() -> u32 { read_reg!("p15,0,$0,c14,c2,0") }
pub fn write_cntp_tval(i : u32){write_reg!("p15,0,$0,c14,c2,0", i)}

pub fn read_cntp_ctl() -> u32 { read_reg!("p15,0,$0,c14,c2,1") }
pub fn write_cntp_ctl(i : u32){write_reg!("p15,0,$0,c14,c2,1", i)}

pub fn read_cntv_tval() -> u32 { read_reg!("p15,0,$0,c14,c3,0") }
pub fn write_cntv_tval(i : u32){write_reg!("p15,0,$0,c14,c3,0", i)}

pub fn read_cntv_ctl() -> u32 { read_reg!("p15,0,$0,c14,c3,1") }
pub fn write_cntv_ctl(i : u32){write_reg!("p15,0,$0,c14,c3,1", i)}
