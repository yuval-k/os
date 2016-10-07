pub mod integrator;
pub mod vector;
pub mod mem;
pub mod cpu;

use kernel_alloc;

#[no_mangle]
pub extern "C" fn arm_main<T : ::mem::FrameAllocator>(mapper : &mut ::mem::MemoryMapper, mut frameAllocator : T) -> !{

    const heap_base : ::mem::VirtualAddress = ::mem::VirtualAddress(0xf000_0000);
    // allocate 5 pages
    let pa = frameAllocator.allocate(1<<22).unwrap();

    // 4k per stack; so need 5*4kb memory = five pages
    mapper.map(&mut frameAllocator, pa, heap_base, (1<<22)*mem::PAGE_SIZE);


    kernel_alloc::init_heap(heap_base.0, (1<<22)*mem::PAGE_SIZE, cpu::get_interrupts, cpu::set_interrupts);
  // DONE. install_interrupt_handlers();
  // TODO: init_timer
  // TODO init_heap()
  // TODO init_scheduler() + threads;
  // TODO init_SMP()
  // TODO create semaphore

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