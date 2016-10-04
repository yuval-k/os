pub mod integrator;
pub mod vector;
pub mod mem;
pub mod cpu;


#[no_mangle]
pub extern "C" fn arm_main(mapper : &mut ::mem::MemoryMapper, frameAllocator : & mut ::mem::FrameAllocator) -> !{

    // map vector tables
    mapper.map(frameAllocator, ::mem::PhysicalAddress(0), vector::VECTORS_ADDR, 1);
    vector::build_vector_table();
  // TODO build_mode_stacks();
  // TODO install_interrupt_handlers();
  // TODO init_heap();

/*
    TODO: to support user space, we can use the MPU:
    memoryProtection.setRegion(kernel_start_virt, kernel_start_virt+WHATEVER, NORMAL)
    memoryProtection.map( mmio, whatever, PAGE_SIZE, DEVICE)
*/

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