pub mod mailbox;
pub mod serial;
pub mod stub;
pub mod intr;
pub mod timer;

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
pub const NUM_CPUS : usize = 4;


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
const MMIO_PSTART: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x3f000000);
const MMIO_SIZE: usize = (16<<20);
const MMIO_PEND: ::mem::PhysicalAddress = ::mem::PhysicalAddress(MMIO_PSTART.0 + MMIO_SIZE); //16mb periferals + 16mv arm local
const MMIO_VSTART: ::mem::VirtualAddress = ::mem::VirtualAddress(0x1000_0000);

const ARM_LOCAL_PSTART: ::mem::PhysicalAddress = ::mem::PhysicalAddress(0x4000_0000);
const ARM_LOCAL_PEND: ::mem::PhysicalAddress = ::mem::PhysicalAddress(ARM_LOCAL_PSTART.0 +  (1<<12)); //4kb
const ARM_LOCAL_VSTART: ::mem::VirtualAddress = ::mem::VirtualAddress(MMIO_VSTART.0 + MMIO_SIZE);



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

const GPIO_BASE : ::mem::VirtualAddress = ::mem::VirtualAddress(MMIO_VSTART.0 + 0x200000);

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
    write_to_console("Debugger enabled!");
    
    while !debug_release() {
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
turn_led_on();
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

    // TODO support sending IPIs to other CPUs when page mapping changes so they can flush tlbs.
    let page_table = mem::init_page_table(::mem::VirtualAddress(l1table_id),
                                              ::mem::VirtualAddress(l2table_space_id),
                                              &ml,
                                              &mut frame_allocator);

    // map all the gpio
    page_table.map_device(&mut frame_allocator,
                    ARM_LOCAL_PSTART,
                    ARM_LOCAL_VSTART,
                    ARM_LOCAL_PEND - ARM_LOCAL_PSTART)
        .unwrap();
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
    if ! platform::is_system_ready() {
        return;
    }
    // only do if we are initialized.
    let mailboxes = & ::platform::get_platform_services().arch_services.as_ref().unwrap().board_services.mailboxes;
    mailboxes.mailboxes[id].set_high(mailbox::MailboxIndex::MailboxZero, 1 <<  (ipi as usize));
}


fn clear_ipi(ipi : ::cpu::IPI) {
    let id = ::platform::get_current_cpu_id();
    let mailboxes = & ::platform::get_platform_services().arch_services.as_ref().unwrap().board_services.mailboxes;
    mailboxes.mailboxes[id].set_low(mailbox::MailboxIndex::MailboxZero, 1 << (ipi as usize));
}
pub struct PlatformServices {
//    pic : Box<pic::PIC>
    mailboxes : mailbox::LocalMailbox,
    timers : [Rc<timer::GlobalTimer>; 4 ],
}


pub struct CpuServices {
//    pic : Box<pic::PIC>
    mailboxes : mailbox::LocalMailbox
}

extern {
    fn _secondary_start () -> !;
}

// This function should be called when we have a heap and a scheduler.
// TODO make sure we have a scheduler..
pub fn init_board() -> PlatformServices {
    // TODO: init mailbox

    // TODO: check how many other CPUs we have,
    // setup a stack of each of them.


    // TODO: make frame allocator and page table accessible to other CPUs
    // other cpus will use provisonal l1 page table to access kernel. 
    // so don't release stub just yet.

    // TODO: by here we shouls assume scheduler is active.

    // for 1 .. (cpu-1):
    //    set stack for CPU
    //    do memory barrier()
    //    wake other CPU(i)
    //    wait for CPU

    let mem_manager = &::platform::get_platform_services().mem_manager;
    let fa = &::platform::get_platform_services().frame_alloc;

    let base = mem_manager.p2v(ARM_LOCAL_PSTART).unwrap();
        
    let mailboxes = mailbox::LocalMailbox::new();
   // let mut pic = Box::new(pic::PIC::new());
    let mut pic : pic::PIC< Box<pic::InterruptSource> , Rc<platform::Interruptable> > = pic::PIC::new();
  
// part of cpu struct?

    let ipi_handler = Rc::new(IPIHandler{});

    // create global timer objects
    let timers = [Rc::new(timer::GlobalTimer::new()), Rc::new(timer::GlobalTimer::new()), Rc::new(timer::GlobalTimer::new()), Rc::new(timer::GlobalTimer::new())];
    for i in 0 .. NUM_CPUS {
        let corepic = Box::new(intr::CorePIC::new_for_cpu(i));
        corepic.enable(intr::Interrupts::Mailbox0 as usize);
        corepic.enable(intr::Interrupts::CNTVIRQ as usize);
        corepic.disable(intr::Interrupts::GPU as usize);
        corepic.disable(intr::Interrupts::PMU as usize);
        corepic.disable(intr::Interrupts::AXI as usize);
        corepic.disable(intr::Interrupts::LocalTimer as usize);

        let handle = pic.add_source( corepic ); //
        pic.register_callback_on_intr(handle, intr::Interrupts::Mailbox0 as usize, ipi_handler.clone());
        pic.register_callback_on_intr(handle, intr::Interrupts::CNTVIRQ as usize,timers[i].clone()); // dont init timer here, but from the cpecific cpu
        // TODO: add to per cpu struct
    }
    drop(ipi_handler);
    
    // make these available to other cpus ^
/* 
    let mut pic = Box::new(pic::PIC::new());
    for all cpus:
        let handle = pic.add_source(interrupt_source[cpuid]);
        pic.register_callback_on_intr(handle, intr::Interrupts::MSGBOX1 as usize, ipi_handler);
        pic.register_callback_on_intr(handle, intr::Interrupts::TIMER3 as usize, gtmr);

// in each cpu:
    interrupts[cpuid].enable(msgbox1);
    interrupts[cpuid].enable(gtmr);
    gtmr.start_timer();

*/

    unsafe{current_page_table = super::super::cpu::get_ttb0();}


    for i in 1 .. NUM_CPUS {
        
        let stk = ::thread::Thread::allocate_stack();

        unsafe{current_stack = stk.0;}

        ::arch::arm::cpu::memory_write_barrier();

        // wake up CPU
        // TODO: WAKE UP CPU
        // write start address to CPU N mailbox 3
        mailboxes.mailboxes[i].set_high(mailbox::MailboxIndex::MailboxThree, _secondary_start as *const u32 as u32);

        // wait for cpu to start
        loop {
            // other cpu hatched and cleared his mailbox
            // what the other cpu does, is "documented" in qemu's write_smpboot: https://github.com/qemu/qemu/blob/4771d756f46219762477aaeaaef9bd215e3d5c60/hw/arm/raspi.c#L35)
            let cpunmbox3 = mailboxes.mailboxes[i].read(mailbox::MailboxIndex::MailboxThree);
            if cpunmbox3 == 0 {
                break;
            }
        }

        // wait for cpu to use the new stack and page table
        while CPUS_AWAKE.load(atomic::Ordering::SeqCst) != i {}
    }
    // stub now not in use by anyone! -  now can deallocate 
    let s_begin = &_stub_begin as *const*const Ptr as usize;
    let s_end = &_stub_end as *const*const Ptr as usize;
    ::platform::get_platform_services().frame_alloc.deallocate(down(s_begin), ::mem::to_pages(up(s_end)-down(s_begin)).expect("misaligned pages!"));
    // TODO: once other cpus started, and signaled that they swiched to use page_table and waiting somewhere in kernel virtmem, continue
    // TODO: remove stub from skip ranges

    let interrupts = InterHandler{pic:pic};

    super::super::vector::get_vec_table().set_irq_callback(Box::new(interrupts));

    timers[0].start_timer();
    // TODO: scheduler should be somewhat available here..
    // TODO: setup gtmr
    // TODO: setup mailbox interrupts to cpus ipi handler
    PlatformServices{
        mailboxes : mailboxes,
        timers : timers,
    }
}

struct InterHandler {
    pic : pic::PIC<Box<pic::InterruptSource> , Rc<platform::Interruptable> >
}

impl platform::Interruptable for InterHandler {
    fn interrupted(&self, ctx: &mut platform::Context) {
        self.pic.interrupted(ctx)
    }
}

struct IPIHandler {
}

impl platform::Interruptable for IPIHandler {
    fn interrupted(&self, ctx: &mut platform::Context) {
    
        let id = ::platform::get_current_cpu_id();
        let mailboxes = & ::platform::get_platform_services().arch_services.as_ref().unwrap().board_services.mailboxes;
        let mut ipis = mailboxes.mailboxes[id].read(mailbox::MailboxIndex::MailboxZero);
        let mut cur_ipi = 0u32;
        while ipis != 0 {
            if   (ipis & 1) != 0  {
                let cur_ipi_enum = int_to_ipi(cur_ipi);
                clear_ipi(cur_ipi_enum);
                // send IPI!    
                ::platform::get_platform_services().get_current_cpu().interrupted(cur_ipi_enum);
            }

            cur_ipi += 1;
            ipis = ipis >> 1;
        }
    }
}

fn int_to_ipi(i : u32) -> ::cpu::IPI{
    match i {
        0 => ::cpu::IPI::MEM_CHANGED,
        1 => ::cpu::IPI::SCHED_CHANGED,
        _ => panic!("unknown IPIs")
    
    }
}

#[no_mangle]
pub extern "C" fn rpi_multi_main() -> ! {
    // we got to here, that means that the stack 
    // is no longer the temp stack.

    // notify..
    CPUS_AWAKE.fetch_add(1, atomic::Ordering::SeqCst);

    // move this to arm_mp_start and call that fuction that will init innterrupt vectors stacks as well
    // enable interrupts
    super::super::build_mode_stacks();

    // TODO TODO BUG: init interrupt stacks
    while ! platform::is_system_ready() {
    }

    let timers = & ::platform::get_platform_services().arch_services.as_ref().unwrap().board_services.timers;
    timers[::platform::get_current_cpu_id()].start_timer();

    // make set current thread the idle loop in the current cpu
    let tid = platform::ThreadId(::sched::MAIN_THREAD_ID.0 + platform::get_current_cpu_id());
    let mut curth = thread::Thread::new_cur_thread(tid);
    curth.cpu_affinity = Some(platform::get_current_cpu_id());
    curth.priority = 0;
    platform::get_platform_services().get_current_cpu().set_running_thread(Box::new(curth));





    platform::set_interrupts(true);
    platform::get_platform_services().get_scheduler().yield_thread();
    loop {
        platform::wait_for_interrupts();
    }

}

#[naked] #[no_mangle]
pub extern "C" fn rpi_multi_pre_main() -> ! {

    // just set the page table and off we go!  and we can continue and init other CPUs

//TODO: make sure this uses no stack! - current stack will only be valid after the new mapping,
// and old stack will be invalid after the mapping.. so..
// this is a bit hacky as i am hoping that no stack will be used (there's no reason for it, anyway..)

    // init real page table - just use the same page table for all cpus..
    unsafe{ super::super::cpu::set_ttb0(current_page_table); }
    // isb to make sure instructino completed
    super::super::cpu::instruction_synchronization_barrier();
    // flush tlb
    super::super::cpu::invalidate_tlb();
    // invalidate cache - as safety incase some code changed
    super::super::cpu::invalidate_caches(); 

    // change to main stack
    unsafe {
        asm!("mov sp, $1
            b $0 "
            :: 
            "i"(rpi_multi_main as extern "C" fn() -> !),
            "r"(current_stack)
            : "sp" : "volatile"
      )
    }
    unsafe {
        ::core::intrinsics::unreachable();
    }
}
