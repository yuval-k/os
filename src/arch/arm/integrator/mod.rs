pub mod serial;
pub mod pic;
pub mod timer;
pub mod stub;

use core::ops;
use super::mem;
use super::vector;
use super::cpu;
use super::thread;

use collections::boxed::Box;

use mem::MemoryMapper;

use device::serial::SerialMMIO;

fn up(a : usize) -> ::mem::PhysicalAddress {::mem::PhysicalAddress((a + mem::PAGE_MASK) & (!mem::PAGE_MASK))}
fn down(a : usize) -> ::mem::PhysicalAddress { ::mem::PhysicalAddress((a ) & (!mem::PAGE_MASK))}

const MMIO_PSTART : ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1000_0000);
const MMIO_PEND : ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x1F00_0000);
const MMIO_VSTART : ::mem::VirtualAddress = ::mem::VirtualAddress(0x1000_0000);

#[no_mangle]
pub extern "C" fn integrator_main(
    sp_end_virt : usize, sp_end_phy : usize, 
    kernel_start_phy : usize, kernel_start_virt : usize, kernel_end_virt : usize,
    l1table_id : usize, l2table_space_id : usize) -> !{

    let ml = mem::MemLayout {
        kernel_start_phy : ::mem::PhysicalAddress(kernel_start_phy),
        kernel_start_virt : ::mem::VirtualAddress(kernel_start_virt),
        kernel_end_virt : ::mem::VirtualAddress(kernel_end_virt),
        stack_phy  : ::mem::PhysicalAddress(sp_end_phy - mem::PAGE_SIZE), // sp points to begining of stack..
        stack_virt :  ::mem::VirtualAddress(sp_end_virt- mem::PAGE_SIZE),
        };

    let skipRanges = [down(kernel_start_virt)..up(kernel_end_virt), down(ml.stack_virt.0) .. up(sp_end_virt), down(l1table_id) .. up(l2table_space_id + 4*mem::L2TABLE_ENTRIES) ];
    // can't use short syntax: https://github.com/rust-lang/rust/pull/21846#issuecomment-110526401
    let mut freedRanges : [Option<ops::Range<::mem::PhysicalAddress>>;10] = [None,None,None,None,None,None,None,None,None,None];
    
    let mut frameAllocator = mem::LameFrameAllocator::new(&skipRanges, &mut freedRanges, 1<<27);

    let mut pageTable = mem::init_page_table(::mem::VirtualAddress(l1table_id), ::mem::VirtualAddress(l2table_space_id), &ml , &mut frameAllocator);

    // map all the gpio
    pageTable.map_device(&mut frameAllocator, MMIO_PSTART, MMIO_VSTART, MMIO_PEND - MMIO_PSTART);

    let mut w = &mut serial::Writer::new(pageTable.p2v(serial::SERIAL_BASE_PADDR).unwrap());
    w.writeln("Welcome home!");

    ::arch::arm::arm_main(pageTable, frameAllocator);

    loop {}
}

pub struct PlatformServices {
//    pic : Box<pic::PIC>
}

// This function should be called when we have a heap and a scheduler.
pub fn init_integrator(mapper : &mut ::mem::MemoryMapper) -> PlatformServices{

    let mut pic_ = Box::new(pic::PIC::new(mapper.p2v(pic::PIC_BASE_PADDR).unwrap()));

    // start a timer
    let mut tmr = Box::new(timer::Timer::new(0, mapper.p2v(timer::TIMERS_BASE).unwrap()));

    tmr.start_timer(true);

    pic_.add_timer_callback(tmr);

    // TODO not move the pic to the vector table.
    vector::get_vec_table().set_irq_callback(pic_);

    PlatformServices{
    //    pic: pic_
    }
}
