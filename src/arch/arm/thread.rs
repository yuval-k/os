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
// TODO change 
pub extern "C" fn switch_context<'a,'b>(current_context: Option< &'a  ::thread::Thread>, new_context: &'b ::thread::Thread) -> Option<&'a ::thread::Thread>  {
    // no need to save the non-scratch registers, as caller shouldn't care about the
    // scratch registers or cpsr
    let current_context_ref : u32 = if let Some(t) = current_context {
        t as *const ::thread::Thread as u32
    } else {
        0u32
    };

    unsafe {
        asm!("
            mov r0, $1
            mov r1, $0
            cmp r0, #0
            beq 1f
            /* store all regs in the stack - cause we can!  */
            push {r4-r12,r14}
            /* save to r0, restore from r1 */
            /* old context saved! */

            /* store sp */
            str sp, [r0, $2]
            1:
            /* load new sp */
            ldr sp, [r1, $2]

            /* restore old regisers */
            pop {r4-r12,r14}

            /* changing threads so time to clear exclusive loads */
            clrex

            /* TODO: add MemBar incase thread goes to other cpu */
            
          ":: "r"(new_context), "r"(current_context_ref) ,
              "i"( SP_OFFSET )
           :"sp","r0","r1","r4","r5","r6","r7","r8","r9","r10","r11","r12","r14" : "volatile")
    };

    return current_context;
}
    /* enable interrupts for new thread, as cspr is at unknown state..*/
#[no_mangle]
extern "C" fn new_thread_trampoline2(arg: u32, f : u32) {

    // TODO: make sure that f is a extern "C" function 
    // and then delete assembly..
    super::cpu::enable_interrupts();

        unsafe {
        asm!("
          mov r0, $0
          bx $1
          ":: "r"(arg), "r"(f) 
        :  : "volatile")
    };

}

// gets stack arg from non scratch regs
#[naked]
extern "C" fn new_thread_trampoline1() {

    unsafe {
        asm!("
          mov r0, r4
          mov r1, r5
          b new_thread_trampoline2
          "::
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
    unsafe { volatile_store(stack.0 as *mut u32, new_thread_trampoline1 as u32); }
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
    unsafe { volatile_store(stack.0 as *mut u32, start.0 as u32); }
    // store r4
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, arg as u32); }
    // store r3
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r2
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r1
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }
    // store r0
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0); }

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
