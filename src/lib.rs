#![feature(lang_items)]
#![no_std]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(step_by)]

#![feature(alloc, collections)]

// TODO: delete
#![feature(drop_types_in_const)]
#![feature(const_fn)]
#![feature(fnbox)]

#[macro_use]
extern crate collections;
extern crate alloc;
extern crate spin;
extern crate rlibc;
extern crate kernel_alloc;
extern crate volatile;

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
pub mod io;

mod drivers;


use collections::boxed::Box;
use collections::Vec;
use alloc::rc::Rc;
use alloc::arc::Arc;

fn init_heap(mapper: &mut ::mem::MemoryMapper, frame_allocator: &mut ::mem::FrameAllocator) {
    const HEAP_BASE: ::mem::VirtualAddress = mem::VirtualAddress(0xf000_0000);
    const HEAP_SIZE: mem::MemorySize = mem::MemorySize::MegaBytes(40);
    let pa = frame_allocator.allocate(mem::to_pages(HEAP_SIZE).ok().unwrap()).unwrap();
    mapper.map(frame_allocator, pa, HEAP_BASE, HEAP_SIZE).unwrap();
    kernel_alloc::init_heap(HEAP_BASE.0,
                            mem::to_bytes(HEAP_SIZE),
                            platform::get_interrupts,
                            platform::set_interrupts);

}

pub fn rust_main<M, F>(mut mapper: M, mut frame_allocator: F)
    where M: mem::MemoryMapper + 'static,
          F: mem::FrameAllocator + 'static
{
    init_heap(&mut mapper, &mut frame_allocator);

    // we can box stuff!
    // init scheduler

    let mut cpus : Vec<cpu::CPU> = Vec::new();
    for i in 0..platform::get_num_cpus() {
        cpus.push(cpu::CPU::new(i));
    }

    let farc = Rc::new(frame_allocator);

    unsafe{
        platform::set_memory_services(platform::MemoryServices{
            mem_manager: Box::new(
                self::mem::DefaultMemoryManagaer::new(
                    Box::new(mapper),
                    farc.clone()
                )
            ), 
            frame_alloc: farc.clone(),
        });
    }
    unsafe{
        platform::set_platform_services(platform::PlatformServices {
            scheduler: sched::Sched::new(),
            arch_services: platform::ArchPlatformServices::new(),
            cpus : cpus,
        });
    }

    // set current thread
    let mut curth = thread::Thread::new_cur_thread(sched::MAIN_THREAD_ID);
    curth.cpu_affinity = Some(platform::get_current_cpu_id());
    curth.priority = 0;
    platform::get_platform_services().get_current_cpu().set_running_thread(Box::new(curth));


    // TODO add the sched interrupt back, to be explicit    
    unsafe { platform::get_mut_platform_services().arch_services.init_platform() };

    // scheduler is ready ! we can use sync objects!

    // platform services is fully initialized, so we can start processing IPIs and the such..
    platform::set_system_ready();
    // enable interrupts!
    platform::set_interrupts(true);


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

    platform::get_platform_services()
        .get_scheduler()
        .spawn(move || {
            main_thread();
        });


    platform::get_platform_services().get_scheduler().yield_thread();
    loop {
        platform::wait_for_interrupts();
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


fn main_thread() {

    // sema
    let sema = Arc::new(sync::Semaphore::new(1));

    // init our thread:
    {
        let sema = sema.clone();

        platform::get_platform_services()
            .get_scheduler()
            .spawn(move || {
                loop {
                    platform::write_to_console("t1 started");
                    platform::write_to_console("t1 acquire");
                    sema.acquire();
                    platform::write_to_console("t1 acquireD");

                    platform::get_platform_services().get_scheduler().sleep(1000);
                    platform::write_to_console("t1 release");
                    sema.release();
                    platform::write_to_console("t1 sem releaseD");
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
                    platform::write_to_console("t2 acquire");
                    sema.acquire();
                    platform::write_to_console("t2 acquireD");

                    platform::write_to_console("t2 sleep 1000");
                    platform::get_platform_services().get_scheduler().sleep(1000);
                    platform::write_to_console("t2 slept");

                    platform::write_to_console("t2 release");
                    sema.release();
                    platform::write_to_console("t2 sem releaseD");
                }
            });
    }

{

        platform::get_platform_services()
        .get_scheduler()
        .spawn(move || {
                let spi = &platform::get_platform_services()
                    .arch_services.driver_manager.spi[0];
                        
                    spi.confiure(device::spi::Configuration{
                                    clock_polarity : None,
                                    clock_phase : None,
                                    speed : Some(device::spi::Hz(800_000)),});
                loop {

                    let leds = [drivers::LED{red:0xff,green : 0, blue: 0},drivers::LED{red:0,green : 0xff, blue: 0},drivers::LED{red:0,green : 0, blue: 0xff}];
                    let buf = drivers::drive_leds(&leds);
                    spi.start_transfer(device::spi::Transfer::new(buf, 1, move|_|{
                        // this is called from interrupt context, so can't do much here...
                        let mut i = 1;
                        i += 1;

                    }));

                        platform::get_platform_services().get_scheduler().sleep(1000);
                }
        });
}
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
