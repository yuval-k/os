pub mod integrator;
pub mod vector;
pub mod mem;
pub mod cpu;

use kernel_alloc;
use ::sched;
use collections::boxed::Box;
use device::serial::SerialMMIO;

#[no_mangle]
pub extern "C" fn arm_main<T : ::mem::FrameAllocator>(mapper : &mut ::mem::MemoryMapper, mut frameAllocator : T) -> !{

    const heap_base : ::mem::VirtualAddress = ::mem::VirtualAddress(0xf000_0000);
    const heap_size : usize = 1 << 22; // 4mb heap
    let pa = frameAllocator.allocate(heap_size >> mem::PAGE_SHIFT).unwrap();
    mapper.map(&mut frameAllocator, pa, heap_base, heap_size);
    kernel_alloc::init_heap(heap_base.0, heap_size, cpu::get_interrupts, cpu::set_interrupts);

    // heap should work now!
    let t : sched::Thread = sched::Thread{
        ctx : vector::Context {
            r0:0,r1:0,r2:0,r3:0,r4:0,r5:0,r6:0,r7:0,r8:0,r9:0,r10:0,r11:0,r12:0,sp:0,lr:0,pc:0,cpsr:0
        }
    };
    let sched : sched::Sched = sched::Sched::new(Box::new(t));

  // DONE. install_interrupt_handlers();
  // DONE: init_timer
  // DONE init_heap()
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