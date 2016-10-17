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
use core::cell::RefCell;

fn init_heap(mapper : &mut ::mem::MemoryMapper, frameAllocator : &mut ::mem::FrameAllocator) {
    const heap_base : ::mem::VirtualAddress = mem::VirtualAddress(0xf000_0000);
    const heap_size : mem::MemorySize = mem::MemorySize::MegaBytes(4); // 4mb heap
    let pa = frameAllocator.allocate(mem::toPages(heap_size).ok().unwrap()).unwrap();
    mapper.map(frameAllocator, pa, heap_base, heap_size);
    kernel_alloc::init_heap(heap_base.0, mem::toBytes(heap_size), platform::get_interrupts, platform::set_interrupts);

}

pub struct PlatformServices {
    pub scheduler : Rc<RefCell<self::sched::Sched>>,
    pub arch_services : platform::ArchPlatformServices
}


pub fn rust_main<M,F,I>(mut mapper :  M, mut frame_allocator : F, init_platform: I) 
where M : mem::MemoryMapper,
      F : mem::FrameAllocator,
      I: Fn(&mut M, &mut F, Rc<RefCell<platform::InterruptSource>>) -> platform::ArchPlatformServices {
    init_heap(&mut mapper, &mut frame_allocator);

    let t : sched::Thread = sched::Thread{
        ctx : platform::Context {
            // TODO make this cross platform
            r0:0,r1:0,r2:0,r3:0,r4:0,r5:0,r6:0,r7:0,r8:0,r9:0,r10:0,r11:0,r12:0,sp:0,lr:0,pc:0,cpsr:0
        }
    };
    // init scheduler
    let sched = Rc::new(RefCell::new(sched::Sched::new(Box::new(t))));

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


    let t1 : sched::Thread = sched::Thread{
    ctx : platform::Context {
        r0:0,r1:0,r2:0,r3:0,r4:0,r5:0,r6:0,r7:0,r8:0,r9:0,r10:0,r11:0,r12:0,sp:STACK2.uoffset(platform::PAGE_SIZE).0 as u32,lr:0,pc:t1 as u32,cpsr: arch::arm::cpu::get_cpsr()
    }
    };

    // TODO wrap in safe methods.
    
    platform_services.scheduler.borrow_mut().spawn_thread(t1);

    loop {
     //   platform_services.scheduler.borrow_mut().yield_thread();
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