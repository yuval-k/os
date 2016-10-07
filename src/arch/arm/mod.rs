pub mod integrator;
pub mod vector;
pub mod mem;
pub mod cpu;

#[no_mangle]
pub extern "C" fn arm_main<T : ::mem::FrameAllocator>(mapper : &mut ::mem::MemoryMapper, mut frameAllocator : T) -> !{

  // DONE. install_interrupt_handlers();
  // TODO: init_timer
  // TODO init_scheduler() + threads;
  // TODO create semaphore
  // TODO init_heap()

/*
    TODO: to support user space, we can use the MPU:
    memoryProtection.setRegion(kernel_start_virt, kernel_start_virt+WHATEVER, NORMAL)
    memoryProtection.map( mmio, whatever, PAGE_SIZE, DEVICE)
*/
    // undefined instruction to test
 //   unsafe{asm!(".word 0xffffffff" :: :: "volatile");}
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