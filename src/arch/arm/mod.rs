
pub mod board;


pub mod vector;
pub mod mem;
pub mod cpu;
pub mod thread;
pub mod pic;
pub mod pl011;

pub use self::board::write_to_console;
pub use self::board::ticks_in_second;

#[cfg(feature = "multicpu")]
pub use self::board::send_ipi;

pub use ::platform;
use core::ops;
use alloc::rc::Rc;
use collections::boxed::Box;
use collections::Vec;

#[cfg(feature = "multicpu")]
pub fn get_num_cpus() -> usize {
    board::NUM_CPUS
}
#[cfg(not(feature = "multicpu"))]
pub fn get_num_cpus() -> usize {
    1
}

pub fn build_mode_stacks() {

    let modes = [cpu::IRQ_MODE, cpu::ABRT_MODE, cpu::UNDEF_MODE, cpu::SYS_MODE];

    for m in modes.iter() {
        cpu::set_stack_for_mode(*m,  ::thread::Thread::allocate_stack());
    }
}

fn init_vectors() {
    platform::get_memory_services().mem_manager.map(
             ::mem::PhysicalAddress(0),
             vector::VECTORS_ADDR,
             ::mem::MemorySize::PageSizes(1))
        .unwrap();
    vector::init_interrupts();
    build_mode_stacks();
}

pub fn arm_main(
    ml : self::mem::MemLayout, 
    skip_frames : &[ops::Range<::mem::PhysicalAddress>],
    initial_l1 : ::mem::VirtualAddress,
    initial_l2 : ::mem::VirtualAddress,
    mem_size : usize) -> ! {

    
    let mut frame_allocator =
        mem::LameFrameAllocator::new(&skip_frames, mem_size);

    let page_table = mem::init_page_table(initial_l1,
                                        initial_l2,
                                        &ml,
                                        &mut frame_allocator);
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
    ::rust_main(page_table, frame_allocator);

    loop {}
}

pub trait Driver{
    fn attach(&mut self, d : DriverHandle);
}



#[derive(Clone, Copy)]
enum DriverHandleEnum {
    Regular(usize),
    Interruptable(usize),
}

#[derive(Clone, Copy)]
pub struct DriverHandle(DriverHandleEnum);

pub trait InterruptableDriver : Driver+platform::Interruptable {}

pub struct DriverManager{
    drivers : Vec<Box<Driver>>,
    interruptable : Vec<Box<InterruptableDriver>>

}

impl DriverManager {

    fn new() -> DriverManager {
        DriverManager{
            drivers : vec![],
            interruptable : vec![],
        }
    }

    fn attach_all(&mut self) {
        for (i,d) in self.drivers.iter_mut().enumerate() {
            d.attach(DriverHandle(DriverHandleEnum::Regular(i)));
        }
        for (i,d) in self.interruptable.iter_mut().enumerate() {
            d.attach(DriverHandle(DriverHandleEnum::Interruptable(i)));
        }
    }

    pub fn add_driver<T : Driver +'static>(&mut self, d : T) -> DriverHandle {
      //  add_interruptable(self, d)
      let dh = DriverHandle(DriverHandleEnum::Regular(self.drivers.len()));
      self.drivers.push(Box::new(d));
      dh
    }
    pub fn add_driver_interruptable<T : InterruptableDriver + 'static>(&mut self, d : T) -> DriverHandle {
      //  add_interruptable(self, d)
      let dh = DriverHandle(DriverHandleEnum::Interruptable(self.interruptable.len()));
      self.interruptable.push(Box::new(d));
      dh
    }

    fn driver_interrupted(&self, dh : DriverHandle) {
        if let DriverHandleEnum::Interruptable(idx) = dh.0 {
            self.interruptable[idx].interrupted();
        } else {
            panic!("driver not interruptable!")
        }
    }
}

/*
impl DriverManager {
    fn get_interruptable(dh) -> &Interruptable {

    }

    fn get_fs_node(dh) -> &Interruptable {

    }

}
*/

pub struct PlatformServices {
    board_services: self::board::PlatformServices,
    pub driver_manager : DriverManager,
    interrupt_service : self::pic::PIC,
}

impl PlatformServices {
    pub fn new() -> Self {
        PlatformServices {
            board_services:  self::board::PlatformServices::new(),
            driver_manager : DriverManager::new(),
            interrupt_service : pic::PIC::new(),
        }
    }

    pub fn init_platform(&mut self) {
         init_vectors();

        self.board_services.init_board();

        // we should have all drivers initialized
        self.driver_manager.attach_all();

        let interrupts = InterHandler::new();
        self::vector::get_vec_table().set_irq_callback(Box::new(interrupts));

        self.interrupt_service.enable_registered();
    }
}


struct InterHandler {
}


impl InterHandler {
    fn new() -> Self {
        InterHandler{

        }
    }
}

impl platform::Interruptable for InterHandler {
    fn interrupted(&self) {
        platform::get_platform_services().arch_services.interrupt_service.interrupted()
    }
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
