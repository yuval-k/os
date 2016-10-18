#![feature(lang_items)]
#![no_std]
#![feature(asm)]
#![feature(naked_functions)] 
#![feature(core_intrinsics)]
#![feature(step_by)]

#![feature(alloc, collections)]

// TODO: delete
#![feature(drop_types_in_const)]

#[macro_use]
extern crate collections;

extern crate alloc;

extern crate rlibc;
extern crate kernel_alloc;

#[macro_use]
extern crate bitflags;

pub mod device;
pub mod arch;
pub mod mem;
pub mod sched;
pub mod platform;


use collections::boxed::Box;
use alloc::rc::Rc;
use core::cell::UnsafeCell;

fn init_heap(mapper : &mut ::mem::MemoryMapper, frameAllocator : &mut ::mem::FrameAllocator) {
    const heap_base : ::mem::VirtualAddress = mem::VirtualAddress(0xf000_0000);
    const heap_size : mem::MemorySize = mem::MemorySize::MegaBytes(4); // 4mb heap
    let pa = frameAllocator.allocate(mem::toPages(heap_size).ok().unwrap()).unwrap();
    mapper.map(frameAllocator, pa, heap_base, heap_size);
    kernel_alloc::init_heap(heap_base.0, mem::toBytes(heap_size), platform::get_interrupts, platform::set_interrupts);

}

pub struct PlatformServices {
    pub scheduler : Rc<UnsafeCell<self::sched::Sched>>,
    pub arch_services : platform::ArchPlatformServices
}


pub fn rust_main<M,F,I>(mut mapper :  M, mut frame_allocator : F, init_platform: I) 
where M : mem::MemoryMapper,
      F : mem::FrameAllocator,
      I: Fn(&mut M, &mut F, Rc<UnsafeCell<platform::InterruptSource>>) -> platform::ArchPlatformServices {
    init_heap(&mut mapper, &mut frame_allocator);

    // init scheduler
    let sched = Rc::new(UnsafeCell::new(sched::Sched::new()));

    let arch_platform_services = init_platform(&mut mapper, &mut frame_allocator, sched.clone());

    // enable interrupts!
    platform::set_interrupts(true);
    

    let mut platform_services = PlatformServices {
        scheduler : sched,
        arch_services : arch_platform_services
    };

    // time to enable interrupts
    platform::set_interrupts(true);

    // init our thread:

    const STACK2 : ::mem::VirtualAddress = ::mem::VirtualAddress(0xDD00_0000);
    let pa = frame_allocator.allocate(1).unwrap();
    mapper.map(&mut frame_allocator, pa, STACK2, mem::MemorySize::PageSizes(1));

    // TODO wrap in safe methods.    
    unsafe { (&mut *platform_services.scheduler.get()).spawn_thread(STACK2.uoffset(platform::PAGE_SIZE), mem::VirtualAddress(t1 as usize), 0); }

    // to do: 
    // create idle thread with lowest priority, that just does wait_for_interurpts
    // create isr thread with highest priority that responds to interrupts. (need semaphore for that..)


    loop {
        unsafe{ (&mut *platform_services.scheduler.get()).yield_thread(); }
    }
    

    // turn on identity map for a lot of bytes
 //   tun_on_identity_map()
 //   build_virtual_table() // we need phy2virt; we need frame alocator with ranges;
 //   flush_mem_and_switch_table()
    // turn on virtual memory and map kernel

    // fix page table and jump to virtual main.

/*
    let mut w : &mut devserial::SerialMMIO = &mut serial::Writer::new();
    w.write_byte('Y' as u8);
    w.write_byte('u' as u8);
    w.write_byte('v' as u8);
    w.write_byte('a' as u8);
    w.write_byte('l' as u8);
*/
}


#[lang = "eh_personality"]
extern fn eh_personality() {
}

#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    loop{}
}


fn t1() {
loop{
}
}