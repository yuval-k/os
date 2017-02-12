use volatile;
use io;

bitflags! {
    #[repr(C,packed)] pub flags DataFlags: u32 {
        const OVERRUN_ERR           = 1 << 11,
        const BREAK_ERR             = 1 << 10,
        const PARITY_ERR            = 1 << 9,
        const FRAMING_ERR           = 1 << 9,
    }
}


bitflags! {
    #[repr(C,packed)] pub flags FlagsFlags: u32 {
        const RING_INDICATOR           = 1 << 8,
        const TRANSMIT_FIFO_EMPTY      = 1 << 7,
        const RECEIVE_FIFO_FULL        = 1 << 6,
        const TRANSMIT_FIFO_FULL       = 1 << 5,
        const RECEIVE_FIFO_EMPTY       = 1 << 4,
        const BUSY                     = 1 << 3,
        const DATA_CARRIER_DETECT      = 1 << 2,
        const DATA_SET_READY           = 1 << 1,
        const CLEAR_TO_SEND            = 1 << 0,
    }
}

bitflags! {
    #[repr(C,packed)] pub flags LineControlFlags: u32 {
        const STICK_PARITY_SELECT      = 1 << 7,
        const WLEN_8                   = 0b11 << 5,
        const WLEN_7                   = 0b10 << 5,
        const WLEN_6                   = 0b01 << 5,
        const WLEN_5                   = 0b00 << 5,
        const ENABLE_FIFO              = 1 << 4,
        const TWO_STOP_BITS_SELECT     = 1 << 3,
        const EVEN_PARITY_SELECT       = 1 << 2,
        const PARITY_ENABLE            = 1 << 1,
        const SEND_BREAK               = 1 << 0,
    }
}


bitflags! {
    #[repr(C,packed)] pub flags ControlFlags: u32 {
        const CTS_ENABLE          = 1 << 15,
        const RST_ENABLE          = 1 << 14,
        const OUT2                = 1 << 13,
        const OUT1                = 1 << 12,
        const REQUEST_TO_SEND     = 1 << 11,
        const DATA_TRANSMIT_READY = 1 << 10,
        const RECEIVE_ENABLE      = 1 << 9,
        const TRANSMIT_ENABLE     = 1 << 8,
        const LOOPBACK_ENABLE     = 1 << 7,
        const RESERVED1           = 1 << 6,
        const RESERVED2           = 1 << 5,
        const RESERVED3           = 1 << 4,
        const RESERVED4           = 1 << 3,
        const SIR_LOW_POWER       = 1 << 2,
        const SIR_ENABLE          = 1 << 1,
        const UART_ENABLE         = 1 << 0,
    }
}

// see here: http://infocenter.arm.com/help/topic/com.arm.doc.ddi0183f/DDI0183.pdf section 3.2
#[repr(C,packed)]
pub struct PL011 {
    data: volatile::Volatile<u32>,
    receive_status_error_clear: volatile::Volatile<u32>,
    reserved1: volatile::Volatile<u32>,
    reserved2: volatile::Volatile<u32>,
    reserved3: volatile::Volatile<u32>,
    reserved4: volatile::Volatile<u32>,
    flags: volatile::ReadOnly<FlagsFlags>,
    reserved5: volatile::Volatile<u32>,
    low_power: volatile::Volatile<u32>,
    integer_baud_rate: volatile::Volatile<u32>,
    fractional_baud_rate: volatile::Volatile<u32>,
    line_control: volatile::Volatile<LineControlFlags>,
    control: volatile::Volatile<ControlFlags>,
    interrupt_fifo_level_select: volatile::Volatile<u32>,
    interrupt_mask_set_clear: volatile::Volatile<u32>,
    raw_interrupt_status: volatile::ReadOnly<u32>,
    masked_interrupt_status: volatile::ReadOnly<u32>,
    interrupt_clear: volatile::WriteOnly<u32>,
    dma_control: volatile::Volatile<u32>,
}


impl PL011 {
    pub unsafe fn new(v : ::mem::VirtualAddress) -> &'static mut Self {
         &mut *(v.0 as *mut PL011)
    }
    
    pub unsafe fn init(&mut self) {
        self.integer_baud_rate.write(1);
        self.fractional_baud_rate.write(40);
        // update, as according to spec there are bits that should not be 
        // modified
        self.line_control.update(|line_control| {
            *line_control |= ENABLE_FIFO | WLEN_8;
        });
        self.control.write(UART_ENABLE | TRANSMIT_ENABLE | RECEIVE_ENABLE);
        
    }
   
}


impl io::WriteFifo for PL011 {
    fn can_write(&self) -> bool {
        ! self.flags.read().contains(TRANSMIT_FIFO_FULL)
    }

    fn write_one(&mut self, b : u8) {
        self.data.write(b as u32)
    }
}

impl io::ReadFifo for PL011 {

    fn can_read(&self) -> bool {
        ! self.flags.read().contains(RECEIVE_FIFO_EMPTY)
    }
    fn read_one(&mut self) -> u8 {
        (self.data.read() & 0xFF) as u8
    }
}
