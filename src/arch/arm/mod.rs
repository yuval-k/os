pub mod integrator;
pub mod vector;
pub mod mem;
pub mod cpu;

fn build_mode_stacks<T : ::mem::FrameAllocator>(mapper : &mut ::mem::MemoryMapper, mut frameAllocator : &mut T) {

    const stacks_base : ::mem::VirtualAddress = ::mem::VirtualAddress(0xb000_0000);
    // allocate 5 pages
    let pa = frameAllocator.allocate(5).unwrap();

    // 4k per stack; so need 5*4kb memory = five pages
    mapper.map(frameAllocator, pa, stacks_base, 5*mem::PAGE_SIZE);
    
    cpu::set_stack_for_modes(stacks_base);
}

#[no_mangle]
pub extern "C" fn arm_main<T : ::mem::FrameAllocator>(mapper : &mut ::mem::MemoryMapper, mut frameAllocator : T) -> !{

    build_mode_stacks(mapper, &mut frameAllocator);
    // map vector tables
    mapper.map(&mut frameAllocator, ::mem::PhysicalAddress(0), vector::VECTORS_ADDR, 1);
    vector::build_vector_table();
  // TODO install_interrupt_handlers();
  // TODO init_heap();
  // TODO init_scheduler();
  // TODO create semaphore

/*
    TODO: to support user space, we can use the MPU:
    memoryProtection.setRegion(kernel_start_virt, kernel_start_virt+WHATEVER, NORMAL)
    memoryProtection.map( mmio, whatever, PAGE_SIZE, DEVICE)
*/
    // undefined instruction to test
    unsafe{asm!(".word 0xffffffff"
          :: :: "volatile"
          );}
    ::rust_main();

    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}


#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> ()
{
    loop {}
}

#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr1() -> ()
{
    loop {}
}