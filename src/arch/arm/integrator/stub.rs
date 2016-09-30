#![link_section=".stub"]

use ::arch::arm::mem;
// enum LinkerPtr{}

extern "C" {
    fn stub_begin_glue() -> *const usize;
    fn stub_end_glue() -> *const usize;
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

    // use SimplePageTable to map big entries:
    // have room in assembly for l1,l2 page tables.
    // 
    // - add stub address
    // map_page(physical_stub_address,physical_stub_address, len)
    // - map address zero to zero
    //  map_page(0,0, len)
    // - map kernel physical address to virtual address
    //  map_page(physical_kernel_address,virtual_kernel_address, len)
    // - first avail phys address to stack address
    //  map_page(physical_right_after_kernel_end, stack virtual, 4kb)
    //

    // the kernel can do user kernel mode bullshit, we gonna go all in kernel mode and TTB1?
    // turn on mmu !
    
    // switch stack and go to rust main! in assembly.. :

    // jump to rust main - rust main is virtual!!! no looking back! TODO set stack to right value
    // rust main should take the page information (stub, kernel, stack) and remove the stub
    // call arm_main() that sets up diff stacks for diff modes
    // arm_main will call rust_main()
    
    let stub_begin :  usize = unsafe{ stub_begin_glue() as usize};
    let stub_end :  usize = unsafe{ stub_end_glue() as usize};
    let kernel_start_phy :  usize = unsafe{ kernel_start_phy_glue() as usize};
    let kernel_start_virt :  usize = unsafe{ kernel_start_virt_glue() as usize};
    let kernel_end_virt :  usize = unsafe{ kernel_end_virt_glue() as usize};
    
    const MB_MASK : usize = !((1 << 20)-1);

    //VIRTAL TABLE TIME! 
    // find next available physical frame
    let l1tableUnsafe : *const u32 = unsafe{ l1pagetable_glue() as *mut _};
    const l1tableEntries : usize = 4096; //4096 entries of 1MB each (=4gb address space). each entry is 4 bytes.

    // 1mb aligned stack pointer. 0xD000000 can be more random
    const stack_pointer_begin : usize = 0xD000000;
    // place the stack physical frame, 1mb aligned and after the page table
    let stack_pointer_phy : usize = ((l1tableUnsafe as usize) +  (4*l1tableEntries)  + 2*(1 << 20)) & MB_MASK;
    const stack_pointer_end : usize = 0xD000000 +  (1 << 20) - 1;

    // This code doesn't use only most basic rust.
    // That's because we want to make sure no rust library functions are called as they reside in unmapped memory
    // (where all the code lives)

    // Zero page table:

    // can't use iterator loop as the code is not mapped yet :(
    let mut i = 0;
    while i < l1tableEntries {
        // can't use offset cause it is not mapped yet :(
        let curEntry : *mut u32 = ((l1tableUnsafe as usize) + 4*i) as *mut u32;
        unsafe{*curEntry = 0;}
        i += 1;
    }

    // VERY CURDE PAGE TABLE
    // map the stub, kernel and stack in a very basic way
    // let the kernel create a normal page table once it is in virtual mode and can use
    // more language constructs

    // http://stackoverflow.com/questions/16383007/what-is-the-right-way-to-update-mmu-translation-table
    // see here: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0333h/I1029222.html
    // map a super section to the stub.
    // map a super section to the kernel.
    // map a super section for the stack
    // give control to the kernel and let it sort this shit out in virtual mode.
    unsafe {
        // TODO: test if kernel is larger than 1mb
        {
            // TODO make sure not more space is needed here...
            let offset = (stub_begin >> 20) as usize;
            let curEntry : *mut u32 = ((l1tableUnsafe as usize) + 4*offset) as *mut u32;
            *curEntry   = (0b10 | 0xc | (0b11 << 10 ) | (stub_begin & MB_MASK)) as u32;
        }
        {
            // TODO make sure that when allocation 1mb address the addresses are 1mb aligned
            // TODO this will not work cause kernel is not 1mb aligned in physical memory
            let offset = (kernel_start_virt >> 20) as usize;
            let curEntry : *mut u32 = ((l1tableUnsafe as usize) + 4*offset) as *mut u32;
            *curEntry   = (0b10 | 0xc | (0b11 << 10 ) | (kernel_start_phy & MB_MASK)) as u32;
        }
        {
            let offset = (stack_pointer_begin >> 20) as usize;
            let curEntry : *mut u32 = ((l1tableUnsafe as usize) + 4*offset) as *mut u32;
            *curEntry   = (0b10 | 0xc | (0b11 << 10 ) | (stack_pointer_phy & MB_MASK)) as u32;
        }
    }
    ::arch::arm::mem::memory_write_barrier();
    ::arch::arm::mem::disable_access_checks();
    ::arch::arm::mem::set_ttb0(l1tableUnsafe as *const());
    ::arch::arm::mem::set_ttb1(l1tableUnsafe as *const());
    // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.ddi0433a/CIHHACFF.html
    ::arch::arm::mem::set_ttbcr(0);
    // TODO turn on caches
    ::arch::arm::mem::enable_mmu();
    
    // now switch stack and call arm main:
    unsafe {
      asm!("mov sp, $0
            b $1 "
            :: "r"(stack_pointer_end) ,"i"(::arch::arm::arm_main as extern "C" fn() -> !) : "sp" : "volatile"
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
