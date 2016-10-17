pub mod serial;

pub trait Device {
    fn new() -> Self;
    fn attach(&mut self);
}