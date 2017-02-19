use volatile;

const GPIO_ADDR : ::mem::VirtualAddress = super::super::GPIO_BASE.uoffset(0);


pub enum FunctionSelect{
    Input     = 0b000,
    Output    = 0b001,
    Function0 = 0b100,
    Function1 = 0b101,
    Function2 = 0b110,
    Function3 = 0b111,
    Function4 = 0b011,
    Function5 = 0b010,
}

fn to_func_select( fs : u32) ->  FunctionSelect {
    match fs {
        0b000 => FunctionSelect::Input,
        0b001 => FunctionSelect::Output,
        0b100 => FunctionSelect::Function0,
        0b101 => FunctionSelect::Function1,
        0b110 => FunctionSelect::Function2,
        0b111 => FunctionSelect::Function3,
        0b011 => FunctionSelect::Function4,
        0b010 => FunctionSelect::Function5,
        _ => panic!("Invalid pin!"),
    }
}

bitflags! {
    #[repr(C,packed)] pub flags PullType: u32 {
        const OFF   = 0b00,
        const PULL_DOWN   = 0b01,
        const PULL_UP   = 0b10,
        const RESERVED   = 0b11,
    }
}

#[repr(C,packed)]
pub struct GPIO  {
   pub func_sel : [volatile::Volatile<u32>; 6],
   pub reserved0 : volatile::Volatile<u32>,

   pub set : [volatile::WriteOnly<u32>; 2],
   pub reserved1 : volatile::Volatile<u32>,

   pub clear : [volatile::WriteOnly<u32>;2],
   pub reserved2 : volatile::Volatile<u32>,

   pub level : [volatile::ReadOnly<u32>; 2],
   pub reserved3 : volatile::Volatile<u32>,

   pub event_detect_status : [volatile::Volatile<u32>; 2],
   pub reserved4 : volatile::Volatile<u32>,

   pub rising_edge_detect_enable : [volatile::Volatile<u32>; 2],
   pub reserved5 : volatile::Volatile<u32>,

   pub falling_edge_detect_enable : [volatile::Volatile<u32>; 2],
   pub reserved6 : volatile::Volatile<u32>,

   pub high_detect_enable : [volatile::Volatile<u32>; 2],
   pub reserved7 : volatile::Volatile<u32>,

   pub low_detect_enable : [volatile::Volatile<u32>; 2],
   pub reserved8 : volatile::Volatile<u32>,

   pub async_rising_edge_detect_enable : [volatile::Volatile<u32>; 2],
   pub reserved9 : volatile::Volatile<u32>,

   pub async_falling_edge_detect_enable : [volatile::Volatile<u32>; 2],
   pub reserved10 : volatile::Volatile<u32>,


   pub pull_up_down_enable : volatile::Volatile<PullType>,
   pub pull_up_down_enable_clock : [volatile::Volatile<u32>; 2],
   pub reserved11 : volatile::Volatile<u32>,
   pub test : volatile::Volatile<u32>,

}


macro_rules! read_bit {
    ( $selv:ident, $v:ident, $b:expr) => {
        match $b {
            0...53 => (($selv.$v[$b >> 5].read() & (1 << ($b & 0b1_1111))) != 0),
            _ => panic!("Invalid pin!"),
        }
    };
}

macro_rules! set_bit {
    ( $selv:ident, $v:ident, $b:expr) => {
        match $b {
            0...53 => {$selv.$v[$b >> 5].write(1 << ($b & 0b1_1111));},
            _ => panic!("Invalid pin!"),
        }
    };
}



impl GPIO {
    
    pub unsafe fn  new() -> &'static mut Self {
        &mut *(GPIO_ADDR.0 as *mut GPIO)
    }

    pub fn set_function(&mut self, pin : usize, func :  FunctionSelect) {
        match pin {
            0...53   => self.func_sel[pin / 10].update(|fs| {*fs = Self::update_fs(pin, *fs, func)}),
            _ => panic!("Invalid pin!"),
        }
    }
    pub fn get_function(&mut self, pin : usize) -> FunctionSelect{
        match pin {
            0...53   => to_func_select(self.func_sel[pin / 10].read() >> (3*(pin%10) & 0b111)),
            _ => panic!("Invalid pin!"),
        }
    }

    
    fn update_fs(pin : usize, mut fs : u32, func :  FunctionSelect) -> u32{
        let pin  = pin % 10;
        // clear old
        fs = fs & !(0b111 << (3*pin));
        fs = fs | ((func as u32) << (3*pin));
        fs
    }


    pub fn read_level(&mut self, pin : usize) -> bool {
        read_bit!(self, level, pin)
    }
    pub fn output_set(&mut self, pin : usize) {
        set_bit!(self, set, pin)
    }
    pub fn output_clear(&mut self, pin : usize) {
        set_bit!(self, clear, pin)
    }

    pub fn event_detect_status_read(&mut self, pin : usize) -> bool {
        read_bit!(self, event_detect_status, pin)
    }

    pub fn event_detect_status_clear(&mut self, pin : usize) {
        set_bit!(self, event_detect_status, pin)
    }



    pub fn set_pullup_pulldown(&mut self, pin : usize, p : PullType) {
        // 1. Write direction
        self.pull_up_down_enable.write(p);
        // 2. wait
        wait150();
        // 3. write clock
        set_bit!(self, pull_up_down_enable_clock, pin);
        // 4. wait
        wait150();
        // 5. disable pud -- not really sure what to do here as there is no disable value in the data sheet..
        // 6. disable clock
        self.pull_up_down_enable_clock[0].write(0);
        self.pull_up_down_enable_clock[1].write(0);
        
    }
}

fn wait150() {
    for _ in 0..150 {
        // nop
        unsafe{asm!("mov r0,r0")};
    }
}