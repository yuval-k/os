use core::slice;
use core::mem;
use super::cpu;

// processor jumps to address 0 so must be ID mapper here for now, till (if?) if will relocate the vectors.
// pub const VECTORS_ADDR : ::mem::VirtualAddress  = ::mem::VirtualAddress(0xea00_0000);
pub const VECTORS_ADDR : ::mem::VirtualAddress  = ::mem::VirtualAddress(0x0);

#[repr(C, packed)]
struct InterruptContext{
    cspr: u32,
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r4: u32,
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
    r9: u32,
    r10: u32,
    r11: u32,
    r12: u32,
    pc: u32,
}

pub struct Context {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r4: u32,
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
    r9: u32,
    r10: u32,
    r11: u32,
    r12: u32,
    sp: u32,
    lr: u32,
    pc: u32,
    cspr: u32,
    // TODO add r13 and r14  (need to switch mode for that..)
}

macro_rules! inthandler {
    ( $handler:ident ) => {{ 

extern "C" fn vector_with_context(ctx : &mut InterruptContext) {
    let (r13, r14) = cpu::get_r13r14(ctx.cspr);
    let c = Context {
    r0: ctx.r0,
    r1: ctx.r1,
    r2: ctx.r2,
    r3: ctx.r3,
    r4: ctx.r4,
    r5: ctx.r5,
    r6: ctx.r6,
    r7: ctx.r7,
    r8: ctx.r8,
    r9: ctx.r9,
    r10: ctx.r10,
    r11: ctx.r11,
    r12: ctx.r12,
    sp: r13,
    lr: r14,
    pc: ctx.pc,
    cspr: ctx.cspr,
    };
    if let Some(newCtx) = $handler(&c) {
        // context switch!!
        cpu::set_r13r14(c.cspr, newCtx.sp, newCtx.lr);
        ctx.r0 = newCtx.r0;
        ctx.r1 = newCtx.r1;
        ctx.r2 = newCtx.r2;
        ctx.r3 = newCtx.r3;
        ctx.r4 = newCtx.r4;
        ctx.r5 = newCtx.r5;
        ctx.r6 = newCtx.r6;
        ctx.r7 = newCtx.r7;
        ctx.r8 = newCtx.r8;
        ctx.r9 = newCtx.r9;
        ctx.r10 = newCtx.r10;
        ctx.r11 = newCtx.r11;
        ctx.r12 = newCtx.r12;
        ctx.pc = newCtx.pc;
        ctx.cspr = newCtx.cspr;
    }
}

#[naked]
extern "C" fn vector_entry() {
    // lr - 4 points to where we need to return
    // original lr and sp are saved on diff mode
    // save all the registers (http://simplemachines.it/doc/arm_inst.pdf)
    // STMFD sp!,{r0-r12, lr}
    // save also spsr, just incase of context switch
    // implement sp, lr = get_orig_sp_lr();
    // then we can save the context!
    // save spsr incase we are gonna need it.
    // r13,r14 are saved in the cpu when we switch modes else; r15 will be the current r14
    unsafe{
    asm!("sub lr,lr, #4
          push {lr}
          push {r0-r12}
          mrs r0, spsr
          push {r0}
          mov r0, sp
          bl $0
    ":: "i"(vector_with_context as extern "C" fn(_) ):  : "volatile");
    }

    // we may want to restore the original sp and lr later on if we do a context switch
    // see: http://wiki.osdev.org/ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch
    // http://wiki.osdev.org/ARM_Integrator-CP_IRQTimerPICTasksMMAndMods

    // restore all registers with s bit on
    unsafe{
    asm!("pop {r0}
          msr spsr, r0
          pop {r0-r12}
          ldmfd sp!, {pc}^
    ": : : : "volatile")
    };
}

vector_entry
    }}
}


/*
based on:

.globl vector_start, vector_end
vector_start
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]
	nop
	ldr pc, [pc, #24]
	ldr pc, [pc, #24]

	.word	vector_reset
	.word	vector_undefined
	.word	vector_softint
	.word	vector_prefetch_abort
	.word	vector_data_abort
	.word	0
	.word	vector_irq
	.word	vector_fiq
vector_end:

*/ 

 
#[naked]
 fn vector_table_asm() {
    unsafe {
        asm!("ldr pc, [pc, #24]" : : : : "volatile");
    };
}

// TODO change vec_table to point to the right place in memory
pub fn build_vector_table() {
    unsafe {
        let mut vec_table : &'static mut [u32] = slice::from_raw_parts_mut(VECTORS_ADDR.0 as *mut u32, 4*8*2);

        let asmjump : *const u32 = vector_table_asm as *const u32;
        vec_table[0] = *asmjump;
        vec_table[1] = *asmjump;
        vec_table[2] = *asmjump;
        vec_table[3] = *asmjump;
        vec_table[4] = *asmjump;
        vec_table[5] = 0;
        vec_table[6] = *asmjump;
        vec_table[7] = *asmjump;

        // default implementations
        vec_table[8+0] = inthandler!(vector_reset_handler) as u32;
        vec_table[8+1] = inthandler!(vector_undefined_handler) as u32;
        vec_table[8+2] = inthandler!(vector_softint_handler) as u32;
        vec_table[8+3] = inthandler!(vector_prefetch_abort_handler) as u32;
        vec_table[8+4] = inthandler!(vector_data_abort_handler) as u32;
        vec_table[8+5] = 0;
        vec_table[8+6] = inthandler!(vector_irq_handler) as u32;
        vec_table[8+7] = inthandler!(vector_fiq_handler) as u32;
    }
}


fn vector_reset_handler(ctx : & Context) -> Option<Context> {

    // TODO : call scheduler
    loop{};
    None

}

fn vector_undefined_handler(ctx : & Context) -> Option<Context> {
    loop{};
    None
}

fn vector_softint_handler(ctx : & Context) -> Option<Context> {
    loop{};
    None
}

fn vector_prefetch_abort_handler(ctx : & Context) -> Option<Context> {
    loop{};
    None
}

fn vector_data_abort_handler(ctx : & Context) -> Option<Context> {
    loop{};
    None
}

fn vector_irq_handler(ctx : & Context) -> Option<Context> {
    loop{};
    None
}

fn vector_fiq_handler(ctx : & Context) -> Option<Context> {
    loop{};
    None
}
