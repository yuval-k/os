use core::result;

pub enum Error {
    FifoFull,
    FifoEmpty,
}

pub type Result<T> = result::Result<T, Error>;

pub trait ReadFifo  {
    fn can_read(&self) -> bool;
    fn read_one(&mut self) -> u8;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.len() == 0 {
            return Ok(0)
        }

        if ! self.can_read() {
            return Err(Error::FifoEmpty)
        }

        for (i, e) in buf.iter_mut().enumerate() {
            *e = self.read_one();            
            if ! self.can_read() {
                return Ok(i+1)
            }
        }
        return Ok(buf.len())

    }
}

pub trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize>;
}
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
}

impl<T: WriteFifo> Write for T {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.try_write(buf)
  }
}

pub trait WriteFifo : Write {
    fn can_write(&self) -> bool;
    fn write_one(&mut self, b : u8);
    
    fn writeln(&mut self, s: &str) -> Result<usize> {
        let size = self.write(s.as_bytes())?; 
        let b = ['\n' as u8];
        self.write(&b)?;
        Ok(size+1)
    }

    fn try_write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.len() == 0 {
            return Ok(0)
        }

        if ! self.can_write() {
            return Err(Error::FifoFull)
        }

        for (i, e) in buf.iter().enumerate() {
            self.write_one(*e);
            if ! self.can_write() {
                return Ok(i+1)
            }
        }
        return Ok(buf.len())
    }
}