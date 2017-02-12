pub mod serial;
pub mod stub;
pub mod intr;
pub mod spi;

use core;
use core::sync::atomic;
use core::intrinsics::{volatile_load, volatile_store};
use collections::boxed::Box;
use alloc::rc::Rc;

use super::super::mem;
use super::super::pic;
use ::platform;
use ::thread;
use rlibc;

use mem::MemoryMapper;
use mem::PVMapper;

use device::serial::SerialMMIO;
use arch::arm::pic::InterruptSource;

pub const ticks_in_second : usize = 20;

static mut current_stack : usize = 0;
static mut current_page_table: *const () = 0 as  *const ();
static CPUS_AWAKE: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

fn up(a: usize) -> ::mem::PhysicalAddress {
    ::mem::PhysicalAddress((a + mem::PAGE_MASK) & (!mem::PAGE_MASK))
}
fn down(a: usize) -> ::mem::PhysicalAddress {
    ::mem::PhysicalAddress((a) & (!mem::PAGE_MASK))
}

// see:
// http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0159b/Bbaficij.html
const MMIO_PSTART: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x20000000);
const MMIO_SIZE: usize = (16<<20);
const MMIO_PEND: ::mem::PhysicalAddress = ::mem::PhysicalAddress(MMIO_PSTART.0 + MMIO_SIZE); //16mb periferals + 16mv arm local
const MMIO_VSTART: ::mem::VirtualAddress = ::mem::VirtualAddress(0x1000_0000);


pub enum Ptr {}

extern "C" {
    static _stub_begin : *const Ptr;
    static _stub_end : *const Ptr;
    static _kernel_start_phy : *const Ptr;
    static _kernel_start_virt : *const Ptr;
    static _kernel_end_virt : *const Ptr;
    static __bss_start : *const Ptr;
    static __bss_end : *const Ptr;
    
}

const GPIO_BASE : ::mem::VirtualAddress = ::mem::VirtualAddress(MMIO_VSTART.0 + 0x20_0000);

// thanks http://sysprogs.com/VisualKernel/tutorials/raspberry/jtagsetup/
fn set_gpio_alt(gpio : u32, func : u32 ) {
    let register_index : usize = gpio as usize / 10;
    let bit = (gpio % 10) * 3;

    let ptr = (GPIO_BASE.0 + core::mem::size_of::<u32>()*register_index) as *mut u32;

    let old_value = unsafe{volatile_load(ptr)};
    let mask : u32 = 0b111 << bit;
    unsafe{volatile_store(ptr, (old_value & (!mask)) | ((func << bit) & mask))};
}

fn orr(v : ::mem::VirtualAddress, vl : u32) {

    let ptr = (v.0) as *mut u32;

    let old_value = unsafe{volatile_load(ptr)};
    unsafe{volatile_store(ptr, old_value | vl)};

}
// http://www.valvers.com/open-software/raspberry-pi/step01-bare-metal-programming-in-cpt1/
fn turn_led_on() {

    let LED_GPFSEL   : usize =   4;
    let LED_GPFBIT   : usize =   21;
    let LED_GPSET    : usize =   8;
    let LED_GPCLR    : usize =   10;
    let LED_GPIO_BIT : usize =   15;

    orr(GPIO_BASE.uoffset(4*LED_GPFSEL), 1 << LED_GPFBIT);
    orr(GPIO_BASE.uoffset(4*LED_GPSET), 1 << LED_GPIO_BIT);
}


fn debug_release() -> bool {
    // deubgger attached will change this to true..
    return false;
}

fn enable_debugger() {
    const GPIO_ALT_FUNCTION_4 :u32 = 0b011;
    const GPIO_ALT_FUNCTION_5 :u32 = 0b010;
    set_gpio_alt(22, GPIO_ALT_FUNCTION_4);
	set_gpio_alt(4,  GPIO_ALT_FUNCTION_5);
	set_gpio_alt(27, GPIO_ALT_FUNCTION_4);
	set_gpio_alt(25, GPIO_ALT_FUNCTION_4);
	set_gpio_alt(23, GPIO_ALT_FUNCTION_4);
	set_gpio_alt(24, GPIO_ALT_FUNCTION_4);
    loop {
    }
}

#[no_mangle]
pub extern "C" fn rpi_main(sp_end_virt: usize,
                                  sp_end_phy: usize,
                                  kernel_start_phy: usize,
                                  kernel_start_virt: usize,
                                  kernel_end_virt: usize,
                                  l1table_id: usize,
                                  l2table_space_id: usize)
                                  -> ! {

    // first thing - zero out the bss
    let bss_start =  &__bss_start as *const*const Ptr as *mut u8;
    let bss_end = &__bss_end as *const*const Ptr as *mut u8;

    unsafe { rlibc::memset(bss_start, 0, (bss_end as usize) - (bss_start as usize))};

    let ml = mem::MemLayout {
        kernel_start_phy: ::mem::PhysicalAddress(kernel_start_phy),
        kernel_start_virt: ::mem::VirtualAddress(kernel_start_virt),
        kernel_end_virt: ::mem::VirtualAddress(kernel_end_virt),
        // TODO: make stack size a constant and not hard coded
        stack_phy: ::mem::PhysicalAddress(sp_end_phy - 2*mem::PAGE_SIZE), /* sp points to begining of stack.. */
        stack_virt: ::mem::VirtualAddress(sp_end_virt - 2*mem::PAGE_SIZE),
    };

    let kernel_size = kernel_end_virt - kernel_start_virt;

    let s_begin = &_stub_begin as *const*const Ptr as usize;
    let s_end = &_stub_end as *const*const Ptr as usize;

    // TODO: add stub to skip ranges
    let skip_ranges = [down(kernel_start_phy)..up(kernel_start_phy + kernel_size),
                       down(ml.stack_phy.0)..up(sp_end_phy),
                       down(s_begin)..up(s_end)];

    let mut frame_allocator =
        mem::LameFrameAllocator::new(&skip_ranges, 1 << 27);

    let page_table = mem::init_page_table(::mem::VirtualAddress(l1table_id),
                                              ::mem::VirtualAddress(l2table_space_id),
                                              &ml,
                                              &mut frame_allocator);

    page_table.map_device(&mut frame_allocator,
                    MMIO_PSTART,
                    MMIO_VSTART,
                    MMIO_PEND - MMIO_PSTART)
        .unwrap();
    unsafe { serial_base = page_table.p2v(serial::SERIAL_BASE_PADDR).unwrap() }

    // gpio mapped, we can enable JTAG pins!
  //  enable_debugger();

    write_to_console("Welcome home!");

    ::arch::arm::arm_main(page_table, frame_allocator);
}

static mut serial_base: ::mem::VirtualAddress = ::mem::VirtualAddress(0);

static serial_writer : ::sync::CpuMutex<()> = ::sync::CpuMutex::<()>::new(());
 
pub fn write_to_console(s: &str) {
    let lock = serial_writer.lock();

    serial::Writer::new(unsafe { serial_base }).writeln(s);
}

pub fn send_ipi(id : usize, ipi : ::cpu::IPI) {
}


fn clear_ipi(ipi : ::cpu::IPI) {
}
pub struct PlatformServices {
//    pic : Box<pic::PIC>
}



// This function should be called when we have a heap and a scheduler.
// TODO make sure we have a scheduler..
pub fn init_board() -> PlatformServices {
    
    
    let mut pic : pic::PIC< Box<pic::InterruptSource> , Rc<platform::Interruptable> > = pic::PIC::new();

    register_interrupts(&mut pic);
  

    let interrupts = InterHandler{pic:pic};
    super::super::vector::get_vec_table().set_irq_callback(Box::new(interrupts));

    PlatformServices{
    }
}

pub fn register_interrupts(pic : &mut pic::PIC< Box<pic::InterruptSource> , Rc<platform::Interruptable> > ) {
  let handle = pic.add_source(Box::new(self::intr::PICDev::new()));

  // TODO add timer

//    pic.register_callback_on_intr(handle, intr::Interrupts::SPI, );

}

struct InterHandler {
    pic : pic::PIC<Box<pic::InterruptSource> , Rc<platform::Interruptable> >
}

impl platform::Interruptable for InterHandler {
    fn interrupted(&self, ctx: &mut platform::Context) {
        self.pic.interrupted(ctx)
    }
}
