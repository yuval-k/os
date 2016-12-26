use collections::boxed::Box;
use core::cell::RefCell;
use core::mem;

pub struct CPU {
    // no need to lock this, as it should only be modified
    // from the same CPU and with no interrupts
    running_thread : RefCell<Option<Box<::thread::Thread>>>,
    id : usize,
}

impl CPU {
    pub fn new(id : usize) -> Self {
        CPU {
            running_thread: RefCell::new(None),
            id : id
        }
    }

    pub fn take_running_thread(&self) -> Box<::thread::Thread> {
        if self.id != ::platform::get_current_cpu_id() {
            panic!("can't take thread from diff cpu");
        }

        let mut m_t = self.running_thread.borrow_mut();
        let mut retval = None;
        mem::swap(&mut *m_t, &mut retval);

        retval.unwrap()
    }

    
    pub fn set_running_thread(&self, t:Box<::thread::Thread>) {
        if self.id != ::platform::get_current_cpu_id() {
            panic!("can't set thread from diff cpu");
        }
        let mut m_t = self.running_thread.borrow_mut();
        mem::replace(&mut *m_t, Some(t));
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_running_thread(&self) -> &RefCell<Option<Box<::thread::Thread>>> {
        &self.running_thread
    }
}
// TODO: implement IPI interface, and use it in the mem map.