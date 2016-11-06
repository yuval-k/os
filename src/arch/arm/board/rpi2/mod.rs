pub mod serial;
pub mod stub;

use core::ops;
use super::super::mem;
use super::super::vector;

use collections::boxed::Box;
use alloc::rc::Rc;

use mem::MemoryMapper;
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
const MMIO_PSTART: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x3f000000);
const MMIO_PEND: ::mem::PhysicalAddress = ::mem::PhysicalAddress(MMIO_PSTART.0 + (16<<20)); //16mb
const MMIO_VSTART: ::mem::VirtualAddress = ::mem::VirtualAddress(0x1000_0000);

#[no_mangle]
pub extern "C" fn rpi_main(sp_end_virt: usize,
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
        stack_phy: ::mem::PhysicalAddress(sp_end_phy - mem::PAGE_SIZE), /* sp points to begining of stack.. */
        stack_virt: ::mem::VirtualAddress(sp_end_virt - mem::PAGE_SIZE),
    };

    let kernel_size = kernel_end_virt - kernel_start_virt;

    // todo: add stub to skip ranges
    let skip_ranges = [down(kernel_start_phy)..up(kernel_start_phy + kernel_size),
                       down(ml.stack_phy.0)..up(sp_end_phy),
                       down(l1table_id)..up(l2table_space_id + 4 * mem::L2TABLE_ENTRIES)];
    // can't use short syntax: https://github.com/rust-lang/rust/pull/21846#issuecomment-110526401
    let mut freed_ranges: [Option<ops::Range<::mem::PhysicalAddress>>; 10] =
        [None, None, None, None, None, None, None, None, None, None];



    let mut frame_allocator =
        mem::LameFrameAllocator::new(&skip_ranges, &mut freed_ranges, 1 << 27);


    let mut page_table = mem::init_page_table(::mem::VirtualAddress(l1table_id),
                                              ::mem::VirtualAddress(l2table_space_id),
                                              &ml,
                                              &mut frame_allocator);
     

    // map all the gpio
    page_table.map_device(&mut frame_allocator,
                    MMIO_PSTART,
                    MMIO_VSTART,
                    MMIO_PEND - MMIO_PSTART)
        .unwrap();


    unsafe { serial_base = page_table.p2v(serial::SERIAL_BASE_PADDR).unwrap() }

    write_to_console("Welcome home!");

    ::arch::arm::arm_main(page_table, frame_allocator);

    loop {}
}

static mut serial_base: ::mem::VirtualAddress = ::mem::VirtualAddress(0);

pub fn write_to_console(s: &str) {
    serial::Writer::new(unsafe { serial_base }).writeln(s);
}

pub struct PlatformServices {
//    pic : Box<pic::PIC>
}

// This function should be called when we have a heap and a scheduler.
pub fn init_board(mapper: &mut ::mem::MemoryMapper,
                       sched_intr: Rc<platform::InterruptSource>)
                       -> PlatformServices {

    // make frame allocator and 
    // page table accessible to other CPUs

    // wake up other CPUs!

    // todo: start and wait for other CPUs
    // todo: once other cpus started, swicthed to use page_table and waiting somewhere in kernel virtmem, continue
    // todo: remove stub from skip ranges

    // todo: scheduler should be somewhat available here..

    PlatformServices{
    //    pic: pic_
    }
}
