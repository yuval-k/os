
pub trait SerialMMIO {
fn write_byte_async(&mut self, ch : u8);
fn is_done(&self) -> bool;
fn write_byte(&mut self, ch : u8) {
    while !self.is_done() {}
    self.write_byte_async(ch);
}
}