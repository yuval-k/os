
// NOTE: DO NOT change struct without changing the inline assembly in switch_context
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

// 0:  r0
// 4:  r1
// 8:  r2
// 12: r3
// 16: r4
// 20: r5
// 24: r6
// 28: r7
// 32: r8
// 36: r9
// 40: r10
// 44: r11
// 48: r12
// 52: sp
// 56: pc
// 60: lr
// 64: cpsr
// couldn't find an easy way to calc offsets using compiler :(
//

const PC_OFFSET: u32 = 56;
const LR_OFFSET: u32 = 60;
const CPSR_OFFSET: u32 = 64;
const SP_OFFSET: u32 = 52;

// switch context without an interrupt.
// called from kernel yeilding functions in system mode.
pub extern "C" fn switch_context(current_context: &mut Context, new_context: &Context) {
    // save the non-scratch registers, as caller shouldn't care about the
    // scratch registers or cpsr
    unsafe {
        asm!("mov r0, $0
          mov r1, $1
          /* save to r1, restore from r0 */
          /* store non scratch regs in the stack - cause we can! */
          stmfd sp!, {r4-r12,r14}
          /* place leavefunc as pc and sp and cspr in save context */
          adr r2, leavefunc
          str r2, [r1, $2]
          /* save cpsr */
          mrs r3, cpsr
          str r3, [r1, $3]

          /* store sp */
          str sp, [r1, $5]

          /* restore cspr to spcr */
          ldr r3, [r0, $3]
          msr spsr, r3

          /* restore lr */
          ldr lr, [r0, $4]
          /* restore regs and context switch */
          /* can't have LR here, see docs: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0204j/Cihcadda.html */
          ldm r0, {r0-r13,r15}^

          /* context switched back; sp, pc and cspr should be correctly set for us, restore all the rest from the stack. */
          leavefunc:

          ldmfd sp!, {r4-r12,r14}

    ":: "r"(new_context), "r"(current_context) ,
        "i"( PC_OFFSET ) ,
        "i"( CPSR_OFFSET ),
        "i"( LR_OFFSET ),
        "i"( SP_OFFSET )
        :  : "volatile");
    }

}

// cspr in system mode with interrupts enabled and no flags.
const NEW_CSPR: u32 = super::cpu::SUPER_MODE;
pub fn new_thread(stack: ::mem::VirtualAddress,
                  start: ::mem::VirtualAddress,
                  arg: usize)
                  -> Context {
    Context {
        // TODO make this cross platform
        r0: arg as u32,
        r1: 0,
        r2: 0,
        r3: 0,
        r4: 0,
        r5: 0,
        r6: 0,
        r7: 0,
        r8: 0,
        r9: 0,
        r10: 0,
        r11: 0,
        r12: 0,
        sp: stack.0 as u32,
        lr: 0,
        pc: start.0 as u32,
        cpsr: NEW_CSPR,
    }
}

// pub struct Thread {
// context : thread::Context,
// sp : ::mem::VirtualAddress,
// TODO make sure we support clousures
// func : fn()
// }
//
