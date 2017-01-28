
pub mod board;


pub mod vector;
pub mod mem;
pub mod cpu;
pub mod thread;
pub mod pic;

pub use self::board::write_to_console;
pub use self::board::ticks_in_second;
pub use self::board::send_ipi;

pub fn get_num_cpus() -> usize {
    board::NUM_CPUS
}

pub fn build_mode_stacks(mapper: &mut ::mem::MemoryMapper,
                         frame_allocator: &mut ::mem::FrameAllocator) {

    const STACK_BASE: ::mem::VirtualAddress = ::mem::VirtualAddress(0xb000_0000);
    const NUM_PAGES: usize = 1;

    let modes = [cpu::IRQ_MODE, cpu::ABRT_MODE, cpu::UNDEF_MODE, cpu::SYS_MODE];

    for (i, m) in modes.iter().enumerate() {
        // TODO allocate pages one by one from frame allocator, as
        // we don't need them contiguous
        let pa = frame_allocator.allocate(NUM_PAGES).unwrap();
        let stack_start = STACK_BASE.uoffset(i << mem::PAGE_SHIFT);
        let stack_end = stack_start.uoffset(NUM_PAGES << mem::PAGE_SHIFT); // one page size.
        mapper.map(frame_allocator,
                 pa,
                 stack_start,
                 ::mem::MemorySize::PageSizes(NUM_PAGES))
            .unwrap();
        cpu::set_stack_for_mode(*m, stack_end);
    }
}

fn init_vectors(mapper: &mut ::mem::MemoryMapper,
                mut frame_allocator: &mut ::mem::FrameAllocator) {
    mapper.map(frame_allocator,
             ::mem::PhysicalAddress(0),
             vector::VECTORS_ADDR,
             ::mem::MemorySize::PageSizes(1))
        .unwrap();
    vector::init_interrupts();
    build_mode_stacks(mapper, frame_allocator);
}

pub fn arm_main<F>(mut mapper: self::mem::PageTable,
                                          mut frame_allocator: F) -> !
          where F : ::mem::FrameAllocator + 'static
                                           {
    // init intr and build mode stacks
    // TODO: add check if done, and do if not  build_mode_stacks(& mut mapper, &mut frame_allocator);

    // init and map vector tables - we don't supposed to have to do this now, but it makes debugging easier..
    init_vectors(&mut mapper, &mut frame_allocator);

    // DONE. install_interrupt_handlers();
    // DONE: init_timer
    // DONE init_heap()
    // DONE init_scheduler() + threads;
    // TODO init_SMP()
    // TODO create semaphore

    // TODO: to support user space, we can use the MPU:
    // memoryProtection.setRegion(kernel_start_virt, kernel_start_virt+WHATEVER, NORMAL)
    // memoryProtection.map( mmio, whatever, PAGE_SIZE, DEVICE)
    //

    // undefined instruction to test
    //   unsafe{asm!(".word 0xffffffff" :: :: "volatile");}
    let initplat = || {

        let board_services = self::board::init_board();

        // init board
        PlatformServices { board_services: board_services }
    };
    ::rust_main(mapper, frame_allocator, initplat);

    loop {}
}

pub struct PlatformServices {
    board_services: self::board::PlatformServices,
}


#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}


#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> () {
    loop {}
}



#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr1() -> () {
    loop {}
}
