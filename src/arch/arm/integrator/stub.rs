#![link_section=".stub"]

use ::arch::arm::mem;

extern "C" {
    fn stub_begin_glue() -> *const usize;
    fn kernel_start_phy_glue() -> *const usize;
    fn kernel_start_virt_glue() -> *const usize;
    fn kernel_end_virt_glue() -> *const usize;
    fn l1pagetable_glue() -> *const usize;
}

// setup virtual table and jump to rust main
#[no_mangle]
#[link_section=".stub"]
 pub  extern fn integrator_main() -> !{
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
    
    let stub_begin :  usize = unsafe{ stub_begin_glue() as usize};
    let kernel_start_phy :  usize = unsafe{ kernel_start_phy_glue() as usize};
    let kernel_start_virt :  usize = unsafe{ kernel_start_virt_glue() as usize};
    let kernel_end_virt :  usize = unsafe{ kernel_end_virt_glue() as usize};
    
    const MB_MASK : usize = !((1 << 20)-1);

    //VIRTAL TABLE TIME! 
    // find next available physical frame
    let l1table_unsafe : *const u32 = unsafe{ l1pagetable_glue() as *mut _};

    // 1mb aligned stack pointer. 0xD000000 can be more random
    const STACK_POINTER_BEGIN : usize = 0xD000000;
    // place the stack physical frame, 1mb aligned and after the page table
    let stack_pointer_phy : usize = ((l1table_unsafe as usize) +  (4*mem::L1TABLE_ENTRIES)  + 2*(1 << 20)) & MB_MASK;
    const STACK_POINTER_END : usize = 0xD000000 +  (1 << 20) - 1;

    // This code here only uses the most basic rust.
    // That's because we want to make sure no rust library functions are called as they reside in unmapped memory
    // (where all the code lives)

    // Zero page table:

    // can't use iterator loop as the code is not mapped yet :(
    let mut i = 0;
    while i < mem::L1TABLE_ENTRIES {
        // can't use offset cause it is not mapped yet :(
        let cur_entry : *mut u32 = ((l1table_unsafe as usize) + 4*i) as *mut u32;
        unsafe{*cur_entry = 0;}
        i += 1;
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
            let cur_entry : *mut u32 = ((l1table_unsafe as usize) + 4*offset) as *mut u32;
            *cur_entry   = (0b10 | 0xc | (0b11 << 10 ) | (stub_begin & MB_MASK)) as u32;
        }
        {
            // TODO: test if kernel is larger than 1mb
            // TODO make sure that when allocation 1mb address the addresses are 1mb aligned
            // TODO this will not work cause kernel is not 1mb aligned in physical memory
            let offset = (kernel_start_virt >> 20) as usize;
            let cur_entry : *mut u32 = ((l1table_unsafe as usize) + 4*offset) as *mut u32;
            *cur_entry   = (0b10 | 0xc | (0b11 << 10 ) | (kernel_start_phy & MB_MASK)) as u32;
        }
        {
            let offset = (STACK_POINTER_BEGIN >> 20) as usize;
            let cur_entry : *mut u32 = ((l1table_unsafe as usize) + 4*offset) as *mut u32;
            *cur_entry   = (0b10 | 0xc | (0b11 << 10 ) | (stack_pointer_phy & MB_MASK)) as u32;
        }
    }
    
    // write barrier probably not needed but just in case..
    mem::memory_write_barrier();
    mem::invalidate_caches();
    mem::invalidate_tlb();
    mem::disable_access_checks();
    mem::set_ttb0(l1table_unsafe as *const());
    mem::set_ttb1(l1table_unsafe as *const());
    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0433a/CIHHACFF.html
    mem::set_ttbcr(0);
    // enable_mmu also turns on caches
    mem::enable_mmu();
    
    // now switch stack and call arm main:
    unsafe {
      asm!("mov sp, $0
            mov r0, $2
            mov r1, $3
            mov r2, $4
            b $1 "
            :: 
            "r"(STACK_POINTER_END) ,
            "i"(::arch::arm::arm_main as extern "C" fn(_,_,_) -> !),
            "r"(kernel_start_phy) ,
            "r"(kernel_start_virt) ,
            "r"(kernel_end_virt)
            : "sp","r0","r1","r2" : "volatile"
      )
    }

    unsafe {
        ::core::intrinsics::unreachable();
    }
}

// interface for page table
// gets framer
// then has findnextfreepage
// map(number of pages) -> virt address
// if number of available pages == 1 add another l1 entry
