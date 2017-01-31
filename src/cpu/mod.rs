use collections::boxed::Box;
use core::cell::RefCell;
use core::mem;
use core::cell::Cell;

pub struct CPU {
    // no need to lock this, as it should only be modified
    // from the same CPU and with no interrupts
    running_thread : RefCell<Option<Box<::thread::Thread>>>,
    id : usize,
    pub should_resched : Cell<bool>,
//    pub arch_services : RefCell<ArchCPUServices>,
}

#[derive(Clone, Copy)]
pub enum IPI {
    MEM_CHANGED,
    SCHED_CHANGED,
}

impl CPU {
    pub fn new(id : usize) -> Self {
        CPU {
            running_thread: RefCell::new(None),
            id : id,
            should_resched : Cell::new(false),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
    
    pub fn send_ipi_to_others(&self, ipi : IPI) {
                    
        let cpus = & ::platform::get_platform_services().cpus;
        for cpu in cpus.iter().filter(|x| x.id != self.id){
            cpu.interrupt(ipi);
        }
    }

    pub fn interrupt(&self, ipi : IPI) {
        ::platform::send_ipi(self.id, ipi);
    }

    pub fn interrupted(&self, ipi : IPI) {
        match ipi {
            MEM_CHANGED => ::platform::invalidate_tlb(),
            // SCHED_CHANGED => if on idle thread - yeild();
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