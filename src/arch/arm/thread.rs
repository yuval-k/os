use core::intrinsics::volatile_store;


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct ThreadMachineContext {
    pub sp: u32,
    
}

const SP_OFFSET: u32 = 0;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Context {
    sp : u32,
}

// switch context without an interrupt.
// called from kernel yeilding functions in system mode.
pub extern "C" fn switch_context(current_context: &mut Context, new_context: &Context)  {
    // save the non-scratch registers, as caller shouldn't care about the
    // scratch registers or cpsr
    unsafe {
        asm!("
          /* store all regs in the stack - cause we can! we store
           scratch registers as well for new threads */
          push {r0-r12,r14}
          mov r1, $0
          mov r0, $1
          /* save to r0, restore from r1 */
          /* old context saved! */

          /* store sp */
          str sp, [r0, $2]
          /* load new sp */
          ldr sp, [r1, $2]

          /* restore old regisers */
          pop {r0-r12,r14}

          /* changing threads so time to clear exclusive loads */
          clrex

          /* return old context (that's already in r0) */
          bx lr

          ":: "r"(new_context), "r"(current_context) ,
              "i"( SP_OFFSET )
        :  : "volatile")
    };

}

#[naked]
extern "C" fn new_thread_trampoline(arg: u32, f : u32) {
    /* enable interrupts for new thread, as cspr is at unknown state..*/
    super::cpu::enable_interrupts();

    unsafe {
        asm!("
          mov r0, $0
          bx $1
          ":: "r"(arg), "r"(f) 
        :  : "volatile")
    };

}


// cspr in system mode with interrupts enabled and no flags.
const NEW_CSPR: u32 = super::cpu::SUPER_MODE;

pub fn new_thread(stack: ::mem::VirtualAddress,
                  start: ::mem::VirtualAddress,
                  arg: usize)
                  -> Context {

    if start.0 == 0 {
        // this is the current thread, so no need to init anything
        return Context {
           sp: 0,
        };
    }

    // fill in the stack so that context_switch will work..
    // basically need to construct stack, as if context switch as called

    // store r14
    let mut stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, new_thread_trampoline as u32); }
    // store r12
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r11
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r10
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r9
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r8
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r7
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r6
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r5
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r4
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r3
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r2
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r1
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, start.0 as u32); }
    // store r0
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, arg as u32); }

    Context {
        sp: stack.0 as u32,
    }
}

// pub struct Thread {
// context : thread::Context,
// sp : ::mem::VirtualAddress,
// TODO make sure we support clousures
// func : fn()
// }
//
