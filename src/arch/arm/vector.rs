use core::slice;
use super::cpu;
use super::thread::Context;
use platform;

use collections::boxed::Box;

// processor jumps to address 0 so must be ID mapper here for now, till (if?) if will relocate the vectors.
// pub const VECTORS_ADDR : ::mem::VirtualAddress  = ::mem::VirtualAddress(0xea00_0000);
pub const VECTORS_ADDR: ::mem::VirtualAddress = ::mem::VirtualAddress(0x0);

// NOTE: DO NOT change struct without changing the inline assembly in vector_entry
#[repr(C, packed)]
#[derive(Copy, Clone)]
struct InterruptContext {
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


macro_rules! inthandler {
    ( $handler:ident ) => {{ 

extern "C" fn vector_with_context(ctx : &mut InterruptContext) {
    let (r13, r14) = cpu::get_r13r14(ctx.cpsr);
    let mut c = Context {
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
    
    $handler(&mut c);

    // potentially context switch - so restore registers incase handler changed them!!
    cpu::set_r13r14(c.cpsr, c.sp, c.lr);
    ctx.r0 = c.r0;
    ctx.r1 = c.r1;
    ctx.r2 = c.r2;
    ctx.r3 = c.r3;
    ctx.r4 = c.r4;
    ctx.r5 = c.r5;
    ctx.r6 = c.r6;
    ctx.r7 = c.r7;
    ctx.r8 = c.r8;
    ctx.r9 = c.r9;
    ctx.r10 = c.r10;
    ctx.r11 = c.r11;
    ctx.r12 = c.r12;
    ctx.pc = c.pc;
    ctx.cpsr = c.cpsr;

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


#[naked]
fn vector_table_asm() {
    unsafe {
        asm!("ldr pc, [pc, #24]" : : : : "volatile");
    };
}

// TODO change vec_table to point to the right place in memory
pub fn init_interrupts() {
    unsafe {
        let mut vec_table: &'static mut [u32] =
            slice::from_raw_parts_mut(VECTORS_ADDR.0 as *mut u32, 4 * 8 * 2);

        let asmjump: *const u32 = vector_table_asm as *const u32;
        vec_table[0] = *asmjump;
        vec_table[1] = *asmjump;
        vec_table[2] = *asmjump;
        vec_table[3] = *asmjump;
        vec_table[4] = *asmjump;
        vec_table[5] = 0;
        vec_table[6] = *asmjump;
        vec_table[7] = *asmjump;

        // default implementations
        vec_table[8 + 0] = inthandler!(vector_reset_handler) as u32;
        vec_table[8 + 1] = inthandler!(vector_undefined_handler) as u32;
        vec_table[8 + 2] = inthandler!(vector_softint_handler) as u32;
        vec_table[8 + 3] = inthandler!(vector_prefetch_abort_handler) as u32;
        vec_table[8 + 4] = inthandler!(vector_data_abort_handler) as u32;
        vec_table[8 + 5] = 0;
        vec_table[8 + 6] = inthandler!(vector_irq_handler) as u32;
        vec_table[8 + 7] = inthandler!(vector_fiq_handler) as u32;
    }
}

pub struct VectorTable {
    // TODO re-write when we have a heap
    irq_callback: Option<Box<platform::InterruptSource>>,
}

// TODO: sync access to this or even better, to it lock free :)
static mut vecTable: VectorTable = VectorTable { irq_callback: None };

// TODO: make thread safe if multi processing !!
pub fn get_vec_table() -> &'static mut VectorTable {
    unsafe { &mut vecTable }
}

impl VectorTable {
    pub fn set_irq_callback(&mut self, callback: Box<platform::InterruptSource>) {
        self.irq_callback = Some(callback);
    }
}

fn vector_reset_handler(_: &mut Context) {

    // TODO : call scheduler
    loop {}

}

fn vector_undefined_handler(_: &mut Context) {
    loop {}
}

fn vector_softint_handler(_: &mut Context) {
    loop {}
}

fn vector_prefetch_abort_handler(_: &mut Context) {
    loop {}
}

fn vector_data_abort_handler(_: &mut Context) {
    loop {}
}

fn vector_irq_handler(ctx: &mut Context) {
    unsafe {
        if let Some(ref mut func) = vecTable.irq_callback {
            func.interrupted(ctx);
        }
    }
}

fn vector_fiq_handler(_: &Context) -> Option<Context> {
    loop {}
}
