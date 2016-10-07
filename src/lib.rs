#![feature(lang_items)]
#![no_std]
#![feature(asm)]
#![feature(naked_functions)] 
#![feature(core_intrinsics)]
#![feature(step_by)]

#![feature(alloc, collections)]

#[macro_use]
extern crate collections;

extern crate rlibc;
extern crate kernel_alloc;

#[macro_use]
extern crate bitflags;

pub mod device;
pub mod arch;
pub mod mem;

use arch::arm::integrator::serial;
use device::serial as devserial;

use ::arch::arm::vector;


pub fn rust_main() {
    vector::build_vector_table();
    
    // turn on identity map for a lot of bytes
 //   tun_on_identity_map()
 //   build_virtual_table() // we need phy2virt; we need frame alocator with ranges;
 //   flush_mem_and_switch_table()
    // turn on virtual memory and map kernel

    // fix page table and jump to virtual main.


    let mut w : &mut devserial::SerialMMIO = &mut serial::Writer::new();
    w.write_byte('Y' as u8);
    w.write_byte('u' as u8);
    w.write_byte('v' as u8);
    w.write_byte('a' as u8);
    w.write_byte('l' as u8);
}

#[lang = "eh_personality"]
extern fn eh_personality() {
}

#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    loop{}
}

