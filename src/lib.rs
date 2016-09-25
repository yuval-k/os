#![feature(lang_items)]
#![no_std]

#![feature(core_intrinsics)]

pub mod device;
pub mod arch;

use arch::arm::integrator::serial;
use device::serial as devserial;



#[no_mangle]
pub extern fn rust_main() {
    // turn on virtual memory and map kernel

    // fix page table and jump to virtual main.


    let mut w : &mut devserial::SerialMMIO = &mut serial::Writer::new();
    w.write_byte('Y' as u8);
    w.write_byte('u' as u8);
    w.write_byte('v' as u8);
    w.write_byte('a' as u8);
    w.write_byte('l' as u8);
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! {loop{}}


#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> ()
{
    loop {}
}