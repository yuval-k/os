#![link_section=".stub"]


use ::arch::arm::cpu;
use ::arch::arm::mem;
use core;

extern "C" {
    fn stub_begin_glue() -> *const usize;
    fn kernel_start_phy_glue() -> *const usize;
    fn kernel_start_virt_glue() -> *const usize;
    fn kernel_end_virt_glue() -> *const usize;
    fn l1pagetable_glue() -> *const usize;
    fn l2pagetable_glue() -> *const usize;
}
// setup virtual table and jump to rust main

#[no_mangle]
#[link_section=".stub"]
pub extern "C" fn stub_main() -> ! {
    // goal of this function is to setup correct virtual table for kernel and then jump to it.
    // the kernel should cleanup the stub functions afterwards.

    // We use a very simple to map just the bare minimum using just the l1 table and section mappings.

    // we map the stub, as we need it till we jump to virtual main
    // map some stack space
    // map the kernel it self to the right virtual place

    // then switch stack and go to rust main! in assembly.. :

    // rust main should take the page information (stub, kernel, stack) and remove the stub
    // call arm_main() that sets up diff stacks for diff modes
    // arm_main will call rust_main()

    // the glue c code shouldnt be needed in theory. but i couldn't get any other
    // way of getting linker variables to work
    let stub_begin: usize = unsafe { stub_begin_glue() as usize };
    let kernel_start_phy: usize = unsafe { kernel_start_phy_glue() as usize };
    let kernel_start_virt: usize = unsafe { kernel_start_virt_glue() as usize };
    let kernel_end_virt: usize = unsafe { kernel_end_virt_glue() as usize };

    const CLEAR_MB_MASK: usize = !mem::MB_MASK;

    // VIRTAL TABLE TIME!
    // find next available physical frame
    let l1table_unsafe: *const u32 = unsafe { l1pagetable_glue() as *mut _ };
    let l2table_unsafe: *const u32 = unsafe { l2pagetable_glue() as *mut _ };

    // 1mb aligned stack pointer. 0xD000000 can be more random
    const STACK_POINTER_BEGIN: usize = 0xD000_0000;
    // place the stack physical frame, 1mb aligned and after the page table
    let stack_pointer_phy: usize = ((l1table_unsafe as usize) + (4 * mem::L1TABLE_ENTRIES) +
                                    2 * (1 << 20)) & CLEAR_MB_MASK;
    const STACK_POINTER_END: usize = STACK_POINTER_BEGIN + (1 << 20);
    let stack_pointer_end_phy: usize = STACK_POINTER_END - STACK_POINTER_BEGIN + stack_pointer_phy;

    // This code here only uses the most basic rust.
    // That's because we want to make sure no rust library functions are called as they reside in unmapped memory
    // (where all the code lives)

    // Zero page table:

    // can't use iterator loop as the code is not mapped yet :(
    {
        let mut i = 0;
        while i < mem::L1TABLE_ENTRIES {
            // can't use offset cause it is not mapped yet :(
            let cur_entry: *mut u32 = ((l1table_unsafe as usize) + 4 * i) as *mut u32;
            unsafe {
                *cur_entry = 0;
            }
            i += 1;
        }
        i = 0;
        while i < mem::L2TABLE_ENTRIES {
            // can't use offset cause it is not mapped yet :(
            let cur_entry: *mut u32 = ((l2table_unsafe as usize) + 4 * i) as *mut u32;
            unsafe {
                *cur_entry = 0;
            }
            i += 1;
        }
    }
    // VERY CURDE PAGE TABLE
    // map the stub, kernel and stack in a very basic way
    // let the kernel create a normal page table once it is in virtual mode and can use
    // more language constructs

    // http://stackoverflow.com/questions/16383007/what-is-the-right-way-to-update-mmu-translation-table
    // see here: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0333h/I1029222.html
    // map a section to the stub.
    // map a section to the kernel.
    // map a section for the stack
    // give control to the kernel and let it sort this shit out in virtual mode.
    unsafe {
        {
            // TODO make sure not more space is needed here...
            let offset = (stub_begin >> 20) as usize;
            let cur_entry: *mut u32 = ((l1table_unsafe as usize) + 4 * offset) as *mut u32;
            *cur_entry = (0b10 | 0xc | (0b11 << 10) | (stub_begin & CLEAR_MB_MASK)) as u32;
        }
        {
            // this will not work cause kernel is not 1mb aligned in physical memory
            let enteries = ((kernel_end_virt - kernel_start_virt)  + mem::MB_MASK )  >> mem::MB_SHIFT;
            let mut i = 0;
            while i < enteries {
                let v = kernel_start_virt + (i << mem::MB_SHIFT);
                let p = kernel_start_phy  + (i << mem::MB_SHIFT);
                let offset = (v >> mem::MB_SHIFT) as usize;
                let cur_entry: *mut u32 = ((l1table_unsafe as usize) + 4 * offset) as *mut u32;
                *cur_entry = (0b10 | 0xc | (0b11 << 10) | (p & CLEAR_MB_MASK)) as u32;
                i += 1;
            }
        }
        // TODO: check number of CPUs, and allocate stack for each CPU. 
        {
            let offset = (STACK_POINTER_BEGIN >> 20) as usize;
            let cur_entry: *mut u32 = ((l1table_unsafe as usize) + 4 * offset) as *mut u32;
            *cur_entry = (0b10 | 0xc | (0b11 << 10) | (stack_pointer_phy & CLEAR_MB_MASK)) as u32;
        }
    }

    // write barrier probably not needed but just in case..
    cpu::memory_write_barrier();
    cpu::invalidate_caches();
    cpu::invalidate_tlb();

    cpu::enable_fpu();
    // disable access checks for domain 0
    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0344k/I1001599.html
    cpu::write_domain_access_control_register(3);
    cpu::set_ttb0(l1table_unsafe as *const ());
    cpu::set_ttb1(l1table_unsafe as *const ());
    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0433a/CIHHACFF.html
    cpu::set_ttbcr(0);
    // enable_mmu also turns on caches
    cpu::enable_mmu();

    // TODO: should we do a data sync barrier here?

    // now switch stack and call arm main:
    unsafe {
        asm!("mov sp, $1
            mov r0, $1
            mov r1, $2
            mov r2, $3
            mov r3, $4
            push {$5}
            push {$6}
            push {$7}
            b $0 "
            :: 
            "i"(super::rpi_main as extern "C" fn(_,_,_,_,_,_,_) -> !),
            "r"(STACK_POINTER_END),
            "r"(stack_pointer_end_phy),
            "r"(kernel_start_phy),
            "r"(kernel_start_virt),
            "r"(l2table_unsafe),
            "r"(l1table_unsafe),
            "r"(kernel_end_virt)
            : "sp","r0","r1","r2","r3" : "volatile"
      )
    }

    unsafe {
        ::core::intrinsics::unreachable();
    }
}

#[no_mangle]
#[link_section=".stub"]
pub extern "C" fn stub_secondary_core() -> ! {
    // TODO init page table from pre-existing pages
    // After we have a page table, setup stack (do all that in assembly?!) STACK_POINTER_BEGIN + 1mb*cpu_index
    // we can remap the stack to a page
    let l1table_unsafe: *const u32 = unsafe { l1pagetable_glue() as *mut _ };
    cpu::enable_fpu();
    cpu::write_domain_access_control_register(3);
    cpu::set_ttb0(l1table_unsafe as *const ());
    cpu::set_ttb1(l1table_unsafe as *const ());
    cpu::set_ttbcr(0);
    cpu::enable_mmu();

    // We should now have access to the stack!
unsafe {
        asm!("mov sp, $1
            b $0 "
            :: 
            "i"(super::rpi_multi_main as extern "C" fn() -> !),
            "r"(super::current_stack)
            : "sp" : "volatile"
      )
    }


    unsafe {
        ::core::intrinsics::unreachable();
    }
}