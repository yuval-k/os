use collections::boxed::Box;

pub struct CPU {
    pub running_thread : Option<Box<::thread::Thread>>,
    pub id : usize,
}

impl CPU {
    pub fn new(id : usize) -> Self {
        CPU {
            running_thread: None,
            id : id
        }
    }
}