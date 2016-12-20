use core::intrinsics::volatile_store;

const SP_OFFSET: u32 = 0;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Context {
    sp : u32,
}


extern {
     fn switch_context3(current_context: *const Context, new_context: *const Context, old_thread : *const ::thread::Thread, new_thread : *const ::thread::Thread);
}

// switch context without an interrupt.
// called from kernel yeilding functions in system mode.
// TODO change 
// TODO: This has to be an assembly naked function so we can control the stack :(
pub fn switch_context<'a,'b>(current_context: Option< &'a  ::thread::Thread>, new_context: &'b ::thread::Thread) -> Option<&'a ::thread::Thread>  {
    // no need to save the non-scratch registers, as caller shouldn't care about the
    // scratch registers or cpsr
    let (current_context_ref, current_thread_ref) = if let Some(t) = current_context {
        (&t.ctx as *const Context,t as *const ::thread::Thread)
    } else {
        (0 as *const Context, 0 as *const ::thread::Thread)
    };
    
    unsafe {
        switch_context3(current_context_ref, &new_context.ctx, current_thread_ref, new_context);
    }
    return current_context;
}


#[naked]
extern "C" fn switch_context2(current_context: *const Context, new_context: *const Context, old_thread : *const ::thread::Thread, new_thread : *const ::thread::Thread) {

    unsafe {
        asm!("
            switch_context3:
            cmp r0, #0
            beq 1f
            /* store all regs in the stack - cause we can!  */
            push {r4-r12,r14}
            /* save to r0, restore from r1 */
            /* old context saved! */

            /* store sp */
            str sp, [r0, $0]
            1:
            /* load new sp */
            ldr sp, [r1, $0]

            /* restore old regisers */
            pop {r4-r12,r14}

            /* changing threads so time to clear exclusive loads */
            clrex

            /* TODO: add MemBar incase thread goes to other cpu */
            /*
            move the thread objects to r0 and r0

            */
            mov r0, r2
            mov r1, r3
            
            bx lr
          ":: "i"( SP_OFFSET ) :: "volatile")
    };


    unsafe {
        ::core::intrinsics::unreachable();
    }
}
    /* enable interrupts for new thread, as cspr is at unknown state..*/
#[no_mangle]
extern "C" fn new_thread_trampoline2(old_thread : *const ::thread::Thread, new_thread : *const ::thread::Thread, arg: u32, f : u32) {

    // TODO:
    // release_old_thread();
    // acquire_new_thread();

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
          mov r2, r4
          mov r3, r5
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
