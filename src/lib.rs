#![feature(lang_items)]
#![no_std]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(step_by)]

#![feature(alloc, collections)]

// TODO: delete
#![feature(drop_types_in_const)]

#![feature(fnbox)]

#[macro_use]
extern crate collections;
extern crate alloc;
extern crate spin;
extern crate rlibc;
extern crate kernel_alloc;

#[macro_use]
extern crate bitflags;

pub mod device;
pub mod arch;
pub mod mem;
pub mod sched;
pub mod platform;


use alloc::rc::Rc;
use alloc::arc::Arc;

fn init_heap(mapper: &mut ::mem::MemoryMapper, frame_allocator: &mut ::mem::FrameAllocator) {
    const HEAP_BASE: ::mem::VirtualAddress = mem::VirtualAddress(0xf000_0000);
    const HEAP_SIZE: mem::MemorySize = mem::MemorySize::MegaBytes(4); // 4mb heap
    let pa = frame_allocator.allocate(mem::to_pages(HEAP_SIZE).ok().unwrap()).unwrap();
    mapper.map(frame_allocator, pa, HEAP_BASE, HEAP_SIZE).unwrap();
    kernel_alloc::init_heap(HEAP_BASE.0,
                            mem::to_bytes(HEAP_SIZE),
                            platform::get_interrupts,
                            platform::set_interrupts);

}

pub fn rust_main<M, F, I>(mut mapper: M, mut frame_allocator: F, init_platform: I)
    where M: mem::MemoryMapper,
          F: mem::FrameAllocator,
          I: Fn(&mut M, &mut F, Rc<platform::InterruptSource>) -> platform::ArchPlatformServices
{
    init_heap(&mut mapper, &mut frame_allocator);

    // init scheduler
    let sched = Rc::new(sched::Sched::new());
    let arch_platform_services = init_platform(&mut mapper, &mut frame_allocator, sched.clone());

    platform::set_platform_services(platform::PlatformServices {
        scheduler: sched,
        arch_services: arch_platform_services,
    });

    // enable interrupts!
    platform::set_interrupts(true);


    // time to enable interrupts
    platform::set_interrupts(true);


    // sema
    let sema = Arc::new(sched::sema::Semaphore::new(0));

    // init our thread:
    {
        let sema = sema.clone();
        let stack2: ::mem::VirtualAddress = ::mem::VirtualAddress(0xDD00_0000);
        let pa = frame_allocator.allocate(1).unwrap();
        mapper.map(&mut frame_allocator,
                 pa,
                 stack2,
                 mem::MemorySize::PageSizes(1))
            .unwrap();
        platform::get_platform_services()
            .get_scheduler()
            .spawn(stack2.uoffset(platform::PAGE_SIZE), move || {
                loop {
                    sema.acquire();
                    sema.release();
                    platform::write_to_console("t1 sem acquired");
                }
            });
    }

    // init our thread:
    {
        let sema = sema.clone();
        let stack2: ::mem::VirtualAddress = ::mem::VirtualAddress(0xDE00_0000);
        let pa = frame_allocator.allocate(1).unwrap();
        mapper.map(&mut frame_allocator,
                 pa,
                 stack2,
                 mem::MemorySize::PageSizes(1))
            .unwrap();
        platform::get_platform_services()
            .get_scheduler()
            .spawn(stack2.uoffset(platform::PAGE_SIZE), move || {
                loop{
                    platform::write_to_console("t2 releasing semaphore");
                    sema.acquire();
                    sema.release();
                }
            });
    }

/*
    let stack2: ::mem::VirtualAddress = ::mem::VirtualAddress(0xDF00_0000);
    let pa = frame_allocator.allocate(1).unwrap();
    mapper.map(&mut frame_allocator,
             pa,
             stack2,
             mem::MemorySize::PageSizes(1))
        .unwrap();
    platform::get_platform_services()
        .get_scheduler()
        .spawn(stack2.uoffset(platform::PAGE_SIZE), t1);
*/
    // to do:
    // create idle thread with lowest priority, that just does wait_for_interurpts
    // create isr thread with highest priority that responds to interrupts. (need semaphore for that..)


    loop {
        platform::get_platform_services().get_scheduler().yield_thread();
    }


    // turn on identity map for a lot of bytes
    //   tun_on_identity_map()
    //   build_virtual_table() // we need phy2virt; we need frame alocator with ranges;
    //   flush_mem_and_switch_table()
    // turn on virtual memory and map kernel

    // fix page table and jump to virtual main.

    // let mut w : &mut devserial::SerialMMIO = &mut serial::Writer::new();
    // w.write_byte('Y' as u8);
    // w.write_byte('u' as u8);
    // w.write_byte('v' as u8);
    // w.write_byte('a' as u8);
    // w.write_byte('l' as u8);
    //
}


#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {

    loop {
        platform::write_to_console("crash");
    }
}


fn t1() {
    loop {

        platform::write_to_console("t1");
    }
}