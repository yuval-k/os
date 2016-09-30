pub mod integrator;
pub mod vector;
pub mod mem;


#[no_mangle]
pub extern "C" fn arm_main() -> !{

    ::rust_main();

    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}


#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() -> ()
{
    loop {}
}