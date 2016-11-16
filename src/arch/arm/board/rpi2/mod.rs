pub mod serial;
pub mod stub;

use core::ops;
use core::sync::atomic;
use super::super::mem;
use super::super::vector;

use collections::boxed::Box;
use alloc::rc::Rc;

use mem::MemoryMapper;
use platform;

use device::serial::SerialMMIO;

pub const ticks_in_second : usize = 20;
pub const NUM_CPUS : usize = 4;


static mut current_stack : usize = 0;
static cpus_awake: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

static need_stub : atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

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

    // TODO: add stub to skip ranges
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

    // TODO: check how many other CPUs we have,
    // setup a stack of each of them.


    // TODO: make frame allocator and page table accessible to other CPUs
    // other cpus will use provisonal l1 page table to access kernel. 
    // so don't release stub just yet.

    // TODO: by here we shouls assume scheduler is active.

    // the other CPUs still need the stub..
    need_stub.store(NUM_CPUS-1, atomic::Ordering::SeqCst);

    // for 1 .. (cpu-1):
    //    set stack for CPU
    //    do memory barrier()
    //    wake other CPU(i)
    //    wait for CPU
    for i in 1 .. NUM_CPUS {
        unsafe{current_stack = 0x100_0000*i  };
        ::arch::arm::cpu::memory_write_barrier();
        // wake up CPU
        // TODO: WAKE UP CPU
        // wait for cpu
        while cpus_awake.load(atomic::Ordering::SeqCst) != i {}
    }
          



    // TODO: start and wait for other CPUs
    // TODO: once other cpus started, and signaled that they swiched to use page_table and waiting somewhere in kernel virtmem, continue
    // TODO: remove stub from skip ranges

    // TODO: scheduler should be somewhat available here..

    PlatformServices{
    //    pic: pic_
    }
}


#[no_mangle]
pub extern "C" fn rpi_multi_main() -> ! {
    // we got to here, that means that the stack 
    // is no longer the temp stack, and we can continue and init other CPUs

    // notify..
    cpus_awake.fetch_add(1, atomic::Ordering::SeqCst);

    // init real page table
    // TODO: !!

    if 1 == need_stub.fetch_sub(1, atomic::Ordering::SeqCst) {
        // if previous value was one, it means that we are the last one that needed the stub
        // and we can release it now
        // TODO: release stub
    }

    loop{}
}
