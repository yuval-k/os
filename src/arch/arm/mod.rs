pub mod integrator;
pub mod vector;
pub mod mem;
pub mod cpu;
pub mod thread;

use kernel_alloc;
use collections::boxed::Box;
use device::serial::SerialMMIO;

// TODO remove integrator::serial hack
use self::integrator::serial;
use ::mem::MemoryMapper;

pub fn build_mode_stacks<T : ::mem::FrameAllocator>(mapper : &mut ::mem::MemoryMapper, mut frameAllocator : &mut T) {

    const stacks_base : ::mem::VirtualAddress = ::mem::VirtualAddress(0xb000_0000);
    
    let modes = [cpu::IRQ_MODE, cpu::ABRT_MODE, cpu::UNDEF_MODE, cpu::SYS_MODE];

    const numPages : usize = 1;

    for (i, m) in modes.iter().enumerate() {
        // TODO allocate pages one by one from frame allocator, as
        // we don't need them contiguous 
        let pa = frameAllocator.allocate(numPages).unwrap();
        let stackStart = stacks_base.uoffset(mem::PAGE_SIZE);
        let stackEnd   = stacks_base.uoffset(mem::PAGE_SIZE * (numPages + 1)); // one page size.
        mapper.map(frameAllocator, pa, stackStart, ::mem::MemorySize::PageSizes(numPages));
        cpu::set_stack_for_mode(*m, stackEnd);
    }
}


#[no_mangle]
pub fn arm_main<T : ::mem::FrameAllocator>(mut mapper : self::mem::PageTable, mut frameAllocator : T) -> !{
    // init intr and build mode stacks
   // TODO: add check if done, and do if not  build_mode_stacks(& mut mapper, &mut frameAllocator);
    // heap should work now!


  // DONE. install_interrupt_handlers();
  // DONE: init_timer
  // DONE init_heap()
  // DONE init_scheduler() + threads;
  // TODO init_SMP()
  // TODO create semaphore

/*
    TODO: to support user space, we can use the MPU:
    memoryProtection.setRegion(kernel_start_virt, kernel_start_virt+WHATEVER, NORMAL)
    memoryProtection.map( mmio, whatever, PAGE_SIZE, DEVICE)
*/

    // undefined instruction to test
 //   unsafe{asm!(".word 0xffffffff" :: :: "volatile");}
    let initplat = |mm : &mut self::mem::PageTable, fa : &mut T| {
        
        let board_services = self::integrator::init_integrator(mm as &mut MemoryMapper);

        // init board
        PlatformServices{
            board_services: board_services
        }
    };
    ::rust_main(mapper, frameAllocator, initplat);

    loop {}
}

pub struct PlatformServices {
    board_services : self::integrator::PlatformServices
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