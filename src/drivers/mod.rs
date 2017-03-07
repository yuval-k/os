
use collections::Vec;

pub struct LED {
pub     red : u8,
pub     blue : u8,
pub     green : u8,
}

pub fn drive_leds(leds : &[LED]) -> Vec<u8> {
    let mut transaction_buf : Vec<u8> = Vec::new();

    let brightness : u8 = 0xFF;

    // 4 zero start
    transaction_buf.push(0);
    transaction_buf.push(0);
    transaction_buf.push(0);
    transaction_buf.push(0);
 
    // leds
    for led in leds.iter() {
        transaction_buf.push(brightness);
        transaction_buf.push(led.red);
        transaction_buf.push(led.blue);
        transaction_buf.push(led.green);
    }

    // end frame, n/2 zero bits to push n bits
    for i in 0..((leds.len()+15) / 16) {
        transaction_buf.push(0);    
    }

    transaction_buf
}