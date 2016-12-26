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
pub mod sync;
pub mod thread;
pub mod platform;
pub mod cpu;


use collections::boxed::Box;
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
    where M: mem::MemoryMapper + 'static,
          F: mem::FrameAllocator + 'static,
          I: Fn() -> platform::ArchPlatformServices
{
    init_heap(&mut mapper, &mut frame_allocator);

    // we can box stuff!
    // init scheduler
    let farc = Rc::new(frame_allocator);
    let p_s = platform::PlatformServices {
            scheduler: sched::Sched::new(),
            mem_manager: Box::new(
                self::mem::DefaultMemoryManagaer::new(
                    Box::new(mapper),
                    farc.clone()
                )
            ), 
            frame_alloc: farc.clone(),
            arch_services: None,
            cpus : vec![::cpu::CPU::new(platform::get_current_cpu_id())]
        };
    unsafe{
        platform::set_platform_services(p_s);
    }

    // set current thread
    platform::get_platform_services().get_current_cpu().set_running_thread(Box::new(thread::Thread::new_cur_thread(sched::MAIN_THREAD_ID)));

    platform::get_mut_platform_services().scheduler.add_idle_thread_for_cpu();

    // TODO add the sched interrupt back, to be explicit
    let arch_plat_services = init_platform();

    unsafe{
        platform::get_mut_platform_services().arch_services = Some(arch_plat_services);
    }
    // scheduler is ready ! we can use sync objects!

    // enable interrupts!
    platform::set_interrupts(true);

    // sema
    let sema = Arc::new(sync::Semaphore::new(1));

    // init our thread:
    {
        let sema = sema.clone();

        platform::get_platform_services()
            .get_scheduler()
            .spawn(move || {
                loop {
                    sema.acquire();
                    platform::get_platform_services().get_scheduler().sleep(1000);
                    sema.release();
                    platform::write_to_console("t1 sem acquired");
                }
            });
    }

    // init our thread:
    {
        let sema = sema.clone();
        
        platform::get_platform_services()
            .get_scheduler()
            .spawn(move || {
                loop{
                    platform::write_to_console("t2 acquire semaphore");
                    sema.acquire();

                    platform::write_to_console("t2 sleep");
                    platform::get_platform_services().get_scheduler().sleep(1000);

                    platform::write_to_console("t2 releasing semaphore");
                    sema.release();
                }
            });
    }

    // let stack2: ::mem::VirtualAddress = ::mem::VirtualAddress(0xDF00_0000);
    // let pa = frame_allocator.allocate(1).unwrap();
    // mapper.map(&mut frame_allocator,
    // pa,
    // stack2,
    // mem::MemorySize::PageSizes(1))
    // .unwrap();
    // platform::get_platform_services()
    // .get_scheduler()
    // .spawn(stack2.uoffset(platform::PAGE_SIZE), t1);
    //
    
    // to do:
    // create idle thread with lowest priority, that just does wait_for_interurpts
    // create isr thread with highest priority that responds to interrupts. (need semaphore for that..)


    loop {
        platform::get_platform_services().get_scheduler().block();
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
#[no_mangle]
extern "C" fn rust_begin_unwind(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    use collections::String;
    use core::fmt::Write;

    platform::write_to_console("PANIC!");
    let mut w = String::new();
    write!(&mut w, "Location: {}:{}; {}", file, line, fmt);
    platform::write_to_console(&w);
    
    loop {}
}


fn t1() {
    loop {

        platform::write_to_console("t1");
    }
}