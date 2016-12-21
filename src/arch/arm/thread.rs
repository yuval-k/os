use core::intrinsics::volatile_store;
use collections::boxed::Box;
use alloc::boxed::FnBox;

const SP_OFFSET: u32 = 0;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Context {
    sp : u32,
}


extern {
     fn switch_context3(current_context: *const Context, new_context: *const Context, old_thread : *const ::thread::Thread, new_thread : *const ::thread::Thread);
}

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


// TODO: This has to be an assembly naked function so we can control the stack :(
// Note the hack - the actual function is switch_context3
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
            /* TODO : might not need to save cspr, as this should always happen from the same mode */

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
            /* move the thread objects to r0 and r0 */
            mov r0, r2
            mov r1, r3
            // TODO: restore cspr
            bx lr
          ":: "i"( SP_OFFSET ) :: "volatile")
    };


    unsafe {
        ::core::intrinsics::unreachable();
    }
}
    /* enable interrupts for new thread, as cspr is at unknown state..*/
#[no_mangle]
extern "C" fn new_thread_trampoline2(old_thread : *const ::thread::Thread, new_thread : *const ::thread::Thread) {

    super::cpu::enable_interrupts();

    let oldthreaed_ref = 
    if old_thread == (0 as *const ::thread::Thread) {
        None
    } else {
        unsafe{ Some(&*old_thread) }
    };

    let new_thread_ref = unsafe {
        &*new_thread
    };

    ::sched::Sched::thread_start(oldthreaed_ref, new_thread_ref);
    unsafe {
        ::core::intrinsics::unreachable();
    }
}

// gets stack arg from non scratch regs
#[naked]
extern "C" fn new_thread_trampoline1() {

    unsafe {
        asm!("
        /* r0 and r1 contain new and old thread respectivly*/
          b new_thread_trampoline2
          "::
        :  : "volatile")
    };

}


// cspr in system mode with interrupts enabled and no flags.
const NEW_CSPR: u32 = super::cpu::SUPER_MODE;

pub fn new_thread(stack: ::mem::VirtualAddress)
                  -> Context {

    if stack.0 == 0 {
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
    unsafe { volatile_store(stack.0 as *mut u32, 0 as u32); }
    // store r4
    stack = stack.offset(-4);
    unsafe { volatile_store(stack.0 as *mut u32, 0 as u32); }

    Context {
        sp: stack.0 as u32,
    }
}