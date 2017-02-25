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

pub const NUM_CPUS : usize = 1;

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
pub fn init_board(pic : &mut pic::PIC< Box<pic::InterruptSource> , Rc<platform::Interruptable> >) -> PlatformServices {

    unsafe { serial_base = platform::get_platform_services().mem_manager.p2v(serial::SERIAL_BASE_PADDR).unwrap() }

    write_to_console("Welcome home!");

    let mapper = &::platform::get_platform_services().mem_manager;

    let mut interrupt_source = Box::new(intr::PIC::new(mapper.p2v(intr::PIC_BASE_PADDR).unwrap()));


    // start a timer
    let mut tmr =
        Box::new(timer::Timer::new(1, mapper.p2v(timer::TIMERS_BASE).unwrap(), Box::new(move||{::platform::get_platform_services().clock()})));

    // timer 1 is 1mhz
    let counter = 1_000_000 / (ticks_in_second as u32);
    tmr.start_timer(counter, true);


    interrupt_source.enable(intr::Interrupts::TIMERINT1 as usize);

    let mut pic : Box<pic::PIC< Box<pic::InterruptSource>, Box<platform::Interruptable>  > > = Box::new(pic::PIC::new());
    let handle = pic.add_source(interrupt_source);
    pic.register_callback_on_intr(handle, intr::Interrupts::TIMERINT1 as usize, tmr);

    // TODO not move the pic to the vector table.
    // as we will need to call it from other places to
    // disable interrupts - perhaps add it to local cpu as well?
    vector::get_vec_table().set_irq_callback(pic);

    PlatformServices{
      //  pic: pic_
    }
}