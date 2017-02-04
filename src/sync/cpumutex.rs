use core::sync::atomic;
use core::default::Default;
use core::ops::{Drop, Deref, DerefMut};
use core::marker::Sync;
use core::cell::{UnsafeCell,Cell};

pub struct CpuMutex<T: ?Sized>{
    owner : atomic::AtomicIsize,
    recursion : Cell<usize>,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for CpuMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for CpuMutex<T> {}

impl<T> CpuMutex<T> {
    pub const fn new(user_data: T) -> Self {
        CpuMutex{
            owner : atomic::AtomicIsize::new(-1),
            recursion : Cell::new(0),
            data: UnsafeCell::new(user_data),
        }
    }
}
pub struct CpuMutexGuard<'a, T: ?Sized + 'a> {
    mutex : &'a CpuMutex<T>,
    data: &'a mut T,

}

impl<T: ?Sized> CpuMutex<T> {
    pub fn lock(&self) -> CpuMutexGuard<T> {
       self.obtain_lock();
        CpuMutexGuard
        {
            mutex: &self,
            data: unsafe { &mut *self.data.get() },
        }
    }

    fn obtain_lock(&self) {
        let curcpu = ::platform::get_current_cpu_id() as isize;
        if self.owner.load(atomic::Ordering::Acquire) == curcpu {
            self.recursion.set(self.recursion.get() + 1);
            return
        }

        while self.owner.compare_and_swap(-1, curcpu, atomic::Ordering::AcqRel) != -1  {
            // Wait until the lock looks unlocked before retrying
            while self.owner.load(atomic::Ordering::Acquire) != -1 {
                // TODO: add arm yield?
            }
        }
        self.recursion.set(1);
    }


    fn release_lock(&self) {
        let curcpu = ::platform::get_current_cpu_id() as isize;
        let lockedcpu = self.owner.load(atomic::Ordering::Acquire);
        if lockedcpu != curcpu {
            // this is a bug!
            panic!("cpu release lock owner mismatch!")
        }
        self.recursion.set(self.recursion.get() - 1);
            if self.recursion.get() > 0 {
                return
        }
        self.owner.store(-1, atomic::Ordering::Release)
        
    }

}


impl<T: ?Sized + Default> Default for CpuMutex<T> {
    fn default() -> CpuMutex<T> {
        CpuMutex::new(Default::default())
    }
}

impl<'a, T: ?Sized> Deref for CpuMutexGuard<'a, T>
{
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T { &*self.data }
}

impl<'a, T: ?Sized> DerefMut for CpuMutexGuard<'a, T>
{
    fn deref_mut<'b>(&'b mut self) -> &'b mut T { &mut *self.data }
}

impl<'a, T: ?Sized> Drop for CpuMutexGuard<'a, T> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self)
    {
        self.mutex.release_lock();
    }
}
