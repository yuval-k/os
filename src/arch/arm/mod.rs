pub mod integrator;
pub mod vector;
pub mod mem;
pub mod cpu;

use kernel_alloc;
use device::serial::SerialMMIO;

#[no_mangle]
pub extern "C" fn arm_main<T : ::mem::FrameAllocator>(mapper : &mut ::mem::MemoryMapper, mut frameAllocator : T) -> !{

    const heap_base : ::mem::VirtualAddress = ::mem::VirtualAddress(0xf000_0000);
    const heap_size :usize = 1 << 22; // 4mb heap
    let pa = frameAllocator.allocate(heap_size >> mem::PAGE_SHIFT).unwrap();
    mapper.map(&mut frameAllocator, pa, heap_base, heap_size);
    kernel_alloc::init_heap(heap_base.0, heap_size, cpu::get_interrupts, cpu::set_interrupts);

    // heap should work now!

    let v = vec!["!","2","1","3"];

    // TODO: better abstraction for drivers..
    let mut w = &mut ::arch::arm::integrator::serial::Writer::new();


    for i in v {
            w.writeln(i);
    }

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