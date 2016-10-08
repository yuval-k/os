use core::slice;
use core::mem;
use super::cpu;

// processor jumps to address 0 so must be ID mapper here for now, till (if?) if will relocate the vectors.
// pub const VECTORS_ADDR : ::mem::VirtualAddress  = ::mem::VirtualAddress(0xea00_0000);
pub const VECTORS_ADDR : ::mem::VirtualAddress  = ::mem::VirtualAddress(0x0);

// NOTE: DO NOT change struct without changing the inline assembly in vector_entry
#[repr(C, packed)]
#[derive(Copy, Clone)]
struct InterruptContext{
    cpsr: u32,
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

// NOTE: DO NOT change struct without changing the inline assembly in switchContext
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Context {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
    pub sp: u32,
    pub pc: u32,
    pub lr: u32,
    pub cpsr: u32,
}

macro_rules! inthandler {
    ( $handler:ident ) => {{ 

extern "C" fn vector_with_context(ctx : &mut InterruptContext) {
    let (r13, r14) = cpu::get_r13r14(ctx.cpsr);
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
    cpsr: ctx.cpsr,
    };
    if let Some(newCtx) = $handler(&c) {
        // context switch!!
        cpu::set_r13r14(c.cpsr, newCtx.sp, newCtx.lr);
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
        ctx.cpsr = newCtx.cpsr;
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
    // sp will be restored automatically when we exit and restore cpsr
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

// TODO: sync access to this or even better, to it lock free :)
static mut vecTable : VectorTable = VectorTable{
    irq_callbacks : [None;10],
    index : 0,
};

// TODO: make thread safe !!
pub fn get_vec_table() -> &'static mut VectorTable { unsafe { &mut vecTable } }

pub struct VectorTable {
    // TODO re-write when we have a heap
    irq_callbacks : [Option<fn(ctx : & Context) -> Option<Context> >;10],
    index : usize,
}

impl VectorTable {
    pub fn register_irq(&mut self, callback : fn(ctx : & Context) -> Option<Context>) {
        self.irq_callbacks[self.index] = Some(callback);
        self.index += 1;
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
    unsafe {
        for i in 0..vecTable.index {
            if let Some(func) = vecTable.irq_callbacks[i] {
                let ret = func(ctx);
                match ret {
                    Some(_) => {
                        return ret;
                    },
                    _ => {},
                }
            }
        }
    }

    None
}

fn vector_fiq_handler(ctx : & Context) -> Option<Context> {
    loop{};
    None
}

/*
0:  r1
4:  r2
8:  r3
12: r4
16: r5
20: r6
24: r7
28: r8
32: r9
36: r10
40: r11
44: r12
48: sp
52: pc
56: lr
60: cpsr
couldn't find an easy way to calc offsets using compiler :(
*/
const PC_OFFSET : u32 =  52;
const LR_OFFSET : u32 = 56;
const CPSR_OFFSET : u32 = 60;
const SP_OFFSET : u32 = 48;

// called from kernel yeilding functions
pub extern "C" fn switchContext(saveContext : &mut Context, newContext  : &Context) {
    // save the non-scratch registers, as caller shouldn't care about the
    // scratch registers or cpsr
    unsafe{
    asm!("mov r0, $0
          mov r1, $1 
          /* save to r1, restore from r0 */
          stmfd sp!, {r4-r12,r14}
          /* place leavefunc as pc and sp and cspr in save context */
          adr r2, leavefunc
          str r2, [r1, $2]
          mrs r3, cpsr
          str r3, [r1, $3]
          /* store sp */
          str sp, [r1, $4]


          /* restore cspr to spcr */
          ldr r3, [r0, $3]
          msr spsr, r3

          /* restore lr */
          ldr lr, [r0, $4]
          /* restore regs and context switch */
          /* can't have LR here, see docs: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0204j/Cihcadda.html */
          ldm r0, {r0-r13,r15}^

          /* context switched back; sp and pc should be correctly set for us, restore all the rest from the stack. */
          leavefunc:

          ldmfd sp!, {r4-r12,r14}

    ":: "r"(newContext), "r"(saveContext) , 
        "i"( PC_OFFSET ) , 
        "i"( CPSR_OFFSET ), 
        "i"( LR_OFFSET ),
        "i"( SP_OFFSET ) :  : "volatile");
    }

}
