pub mod serial;
pub mod intr;
pub mod timer;
pub mod stub;

use core::ops;
use super::super::mem;
use super::super::vector;
use super::super::pic;

use collections::boxed::Box;
use alloc::rc::Rc;
use arch::arm::pic::InterruptSource;

use mem::FrameAllocator;
use mem::MemoryMapper;
use mem::PVMapper;
use platform;

use device::serial::SerialMMIO;

pub const ticks_in_second : usize = 20;

fn up(a: usize) -> ::mem::PhysicalAddress {
    ::mem::PhysicalAddress((a + mem::PAGE_MASK) & (!mem::PAGE_MASK))
}
fn down(a: usize) -> ::mem::PhysicalAddress {
    ::mem::PhysicalAddress((a) & (!mem::PAGE_MASK))
}

// see:
// http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0159b/Bbaficij.html
const MMIO_PSTART: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1000_0000);
const MMIO_PEND: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1F00_0000);
const MMIO_VSTART: ::mem::VirtualAddress = ::mem::VirtualAddress(0x1000_0000);

#[no_mangle]
pub extern "C" fn integrator_main(sp_end_virt: usize,
                                  sp_end_phy: usize,
                                  kernel_start_phy: usize,
                                  kernel_start_virt: usize,
                                  kernel_end_virt: usize,
                                  l1table_id: usize,
                                  l2table_space_id: usize)
                                  -> ! {

    let ml = mem::MemLayout {
        kernel_start_phy: ::mem::PhysicalAddress(kernel_start_phy),
        kernel_start_virt: ::mem::VirtualAddress(kernel_start_virt),
        kernel_end_virt: ::mem::VirtualAddress(kernel_end_virt),
        stack_phy: ::mem::PhysicalAddress(sp_end_phy - 2*mem::PAGE_SIZE), /* sp points to begining of stack.. */
        stack_virt: ::mem::VirtualAddress(sp_end_virt - 2*mem::PAGE_SIZE),
    };

    let kernel_size = kernel_end_virt - kernel_start_virt;

    let pagetable_start = down(l1table_id);
    let pagetable_end = up(l2table_space_id + 4 * mem::L2TABLE_ENTRIES);

    let skip_ranges = [down(kernel_start_phy)..up(kernel_start_phy + kernel_size),
                       down(ml.stack_phy.0)..up(sp_end_phy),
                       pagetable_start..pagetable_end];


    ::arch::arm::arm_main(ml, &skip_ranges,
        ::mem::VirtualAddress(l1table_id),
        ::mem::VirtualAddress(l2table_space_id), 1 << 27);

    loop {}
}

static mut serial_base: ::mem::VirtualAddress = ::mem::VirtualAddress(0);

pub fn write_to_console(s: &str) {
    serial::Writer::new(unsafe { serial_base }).writeln(s);
}

pub struct PlatformServices {
  //  pic : Box<pic::PIC>
}

pub struct CpuServices {
}

pub fn send_ipi(id : usize, ipi : ::cpu::IPI) {
    panic!("no ipi support in integrator.")
}


// This function will be called when we have a heap and a scheduler.

impl PlatformServices{

pub fn new() -> Self {
    platform::get_memory_services().mem_manager.map_device(
                    MMIO_PSTART,
                    MMIO_VSTART,
                    MMIO_PEND - MMIO_PSTART)
        .unwrap();
        PlatformServices{

        }
}

pub fn init_board(&mut self) -> PlatformServices {

    unsafe { serial_base = platform::get_memory_services().mem_manager.p2v(serial::SERIAL_BASE_PADDR).unwrap() }

    write_to_console("Welcome home!");

    let mapper = &::platform::get_memory_services().mem_manager;

    let interrupt_source = intr::PIC::new(mapper.p2v(intr::PIC_BASE_PADDR).unwrap());
    &platform::get_platform_services().arch_services.interrupt_service.add_source(interrupt_source);

    // start a timer
    let mut tmr = timer::Timer::new(1, mapper.p2v(timer::TIMERS_BASE).unwrap(), Box::new(move||{::platform::get_platform_services().clock()}));

    // timer 1 is 1mhz
    let counter = 1_000_000 / (ticks_in_second as u32);
    tmr.start_timer(counter, true);
    
    let dm = unsafe{&mut platform::get_mut_platform_services().arch_services.driver_manager};

    dm.add_driver_interruptable(tmr);

    PlatformServices {
      //  pic: pic_
    }
}

}