#![feature(allocator)]

#![allocator]
#![no_std]



extern crate spin;
extern crate linked_list_allocator;

// thanks phil!
// http://os.phil-opp.com/kernel-heap.html

use core::cell::UnsafeCell;
use core::ops::{Drop, Deref, DerefMut};


struct InterruptGuard<T: ?Sized> {

    get_int : fn() -> bool,
    set_int : fn(bool),
    data: UnsafeCell<T>,
}



struct InterruptGuardHelper<'a, T: ?Sized + 'a> {

    to_state : bool,
    set_int  : fn(bool),
    data: &'a mut T,

}

unsafe impl<T: ?Sized + Send> Sync for InterruptGuard<T> {}
unsafe impl<T: ?Sized + Send> Send for InterruptGuard<T> {}

impl<T> InterruptGuard<T>
{

    pub fn new<'a>(user_data: T,  get_int : fn() -> bool, set_int : fn(bool) ) -> InterruptGuard<T>
    {
        InterruptGuard
        {
            get_int : get_int,
            set_int : set_int,
            data: UnsafeCell::new(user_data),
        }
    }

    pub fn no_interrupts(&self) -> InterruptGuardHelper<T>
    {
        let old_state = (self.get_int)();
        (self.set_int)(false);

        InterruptGuardHelper
        {
            to_state: old_state,
            set_int: self.set_int,
            data: unsafe { &mut *self.data.get() },
        }
    }
}


impl<'a, T: ?Sized> Deref for InterruptGuardHelper<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T: ?Sized> DerefMut for InterruptGuardHelper<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T { &mut *self.data }
}

impl<'a, T: ?Sized> Drop for InterruptGuardHelper<'a, T>
{
    fn drop(&mut self)
    {
        (self.set_int)(self.to_state);
    }
}

pub fn init_heap(start : usize, size : usize, get_int : fn() -> bool, set_int : fn(bool) ) {
     unsafe {
         HEAP = Some(
         InterruptGuard::new(
             spin::Mutex::new(linked_list_allocator::Heap::new(start, size))
             , get_int, set_int));
     }
}

#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe {
        let g = HEAP.as_mut().unwrap().no_interrupts();
        return g.lock().allocate_first_fit(size, align).expect("out of memory");
    }
}

static mut HEAP: Option<InterruptGuard<spin::Mutex<linked_list_allocator::Heap>>> = None;

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, size: usize, align: usize) {
    unsafe {    
        let g = HEAP.as_mut().unwrap().no_interrupts();
        g.lock().deallocate(ptr, size, align);
    }
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, size: usize,
    _new_size: usize, _align: usize) -> usize
{
    size
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, size: usize, new_size: usize,
                                align: usize) -> *mut u8 {
    use core::{ptr, cmp};

    // from: https://github.com/rust-lang/rust/blob/
    //     c66d2380a810c9a2b3dbb4f93a830b101ee49cc2/
    //     src/liballoc_system/lib.rs#L98-L101

    let new_ptr = __rust_allocate(new_size, align);
    unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new_size)) };
    __rust_deallocate(ptr, size, align);
    new_ptr
}