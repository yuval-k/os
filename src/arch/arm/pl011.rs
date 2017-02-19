use volatile;
use io;


enum FifoLevels {
    Level18 = 0b000,
    Level14 = 0b001,
    Level12 = 0b010,
    Level34 = 0b011,
    Level78 = 0b100,
}

bitflags! {
    #[repr(C,packed)] pub flags FifoLevelFlags: u32 {
        const TXIFLSEL_LEVEL1_8 = (FifoLevels::Level18 as u32) << 0,
        const TXIFLSEL_LEVEL1_4 = (FifoLevels::Level14 as u32) << 0,
        const TXIFLSEL_LEVEL1_2 = (FifoLevels::Level12 as u32) << 0,
        const TXIFLSEL_LEVEL3_4 = (FifoLevels::Level34 as u32) << 0,
        const TXIFLSEL_LEVEL7_8 = (FifoLevels::Level78 as u32) << 0,
        
        const RXIFLSEL_LEVEL1_8 = (FifoLevels::Level18 as u32) << 3,
        const RXIFLSEL_LEVEL1_4 = (FifoLevels::Level14 as u32) << 3,
        const RXIFLSEL_LEVEL1_2 = (FifoLevels::Level12 as u32) << 3,
        const RXIFLSEL_LEVEL3_4 = (FifoLevels::Level34 as u32) << 3,
        const RXIFLSEL_LEVEL7_8 = (FifoLevels::Level78 as u32) << 3,
    }
}

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
bitflags! {
    #[repr(C,packed)] pub flags InterruptFlags: u32 {
        const UARTOEINTR    = 1 << 10,
        const UARTBEINTR    = 1 << 9,
        const UARTPEINTR    = 1 << 8,
        const UARTFEINTR    = 1 << 7,
        const UARTRTINTR    = 1 << 6,
        const UARTTXINTR    = 1 << 5,
        const UARTRXINTR    = 1 << 4,
        const UARTDSRINTR   = 1 << 3,
        const UARTDCDINTR   = 1 << 2,
        const UARTCTSINTR   = 1 << 1,
        const UARTRIINTR    = 1 << 0,
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
    interrupt_fifo_level_select: volatile::Volatile<FifoLevelFlags>,
    interrupt_mask_set_clear: volatile::Volatile<InterruptFlags>,
    raw_interrupt_status: volatile::ReadOnly<InterruptFlags>,
    masked_interrupt_status: volatile::ReadOnly<InterruptFlags>,
    interrupt_clear: volatile::WriteOnly<InterruptFlags>,
    dma_control: volatile::Volatile<u32>,
}


impl PL011 {
    pub unsafe fn new(v: ::mem::VirtualAddress) -> &'static mut Self {
        let p = &mut *(v.0 as *mut PL011);

        // disable all
        p.control.write(ControlFlags::empty());

        p.integer_baud_rate.write(1);
        p.fractional_baud_rate.write(40);
        // update, as according to spec there are bits that should not be
        // modified
        p.line_control.update(|line_control| { *line_control |= ENABLE_FIFO | WLEN_8; });

        // clear all interrupts.. we are just starting!
        p.interrupt_clear.write(InterruptFlags::all());

        p.control.write(UART_ENABLE | TRANSMIT_ENABLE | RECEIVE_ENABLE);


        p
    }
}

// TODO implement interrupt handler (template over something that can borrow a slice??)


impl io::WriteFifo for PL011 {
    fn can_write(&self) -> bool {
        !self.flags.read().contains(TRANSMIT_FIFO_FULL)
    }

    fn write_one(&mut self, b: u8) {
        self.data.write(b as u32)
    }
}

impl io::ReadFifo for PL011 {
    fn can_read(&self) -> bool {
        !self.flags.read().contains(RECEIVE_FIFO_EMPTY)
    }
    fn read_one(&mut self) -> u8 {
        (self.data.read() & 0xFF) as u8
    }
}
