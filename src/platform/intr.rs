
use core::cell::UnsafeCell;
use core::ops::{Drop, Deref, DerefMut};

use super::get_interrupts;
use super::set_interrupts;


pub fn no_interrupts() -> InterruptGuardOneShot {
    let old_state = get_interrupts();
    set_interrupts(false);

    InterruptGuardOneShot { to_state: old_state }
}

pub struct InterruptGuardOneShot {
    to_state: bool,
}

impl Drop for InterruptGuardOneShot {
    fn drop(&mut self) {
        set_interrupts(self.to_state);
    }
}



pub struct InterruptGuard<T: ?Sized> {
    data: UnsafeCell<T>,
}

pub struct InterruptGuardHelper<'a, T: ?Sized + 'a> {
    to_state: bool,
    data: &'a mut T,
}

unsafe impl<T: ?Sized + Send> Sync for InterruptGuard<T> {}
unsafe impl<T: ?Sized + Send> Send for InterruptGuard<T> {}

impl<T> InterruptGuard<T> {
    pub fn new<'a>(user_data: T) -> InterruptGuard<T> {
        InterruptGuard { data: UnsafeCell::new(user_data) }
    }

    pub fn no_interrupts(&self) -> InterruptGuardHelper<T> {
        let old_state = get_interrupts();
        set_interrupts(false);

        InterruptGuardHelper {
            to_state: old_state,
            data: unsafe { &mut *self.data.get() },
        }
    }
}


impl<'a, T: ?Sized> Deref for InterruptGuardHelper<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for InterruptGuardHelper<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for InterruptGuardHelper<'a, T> {
    fn drop(&mut self) {
        set_interrupts(self.to_state);
    }
}
