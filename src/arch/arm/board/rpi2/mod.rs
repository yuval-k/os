pub mod mailbox;
pub mod serial;
pub mod stub;

use core;
use core::sync::atomic;
use core::intrinsics::{volatile_load, volatile_store};

use super::super::mem;
use rlibc;

use mem::MemoryMapper;
use mem::PVMapper;

use device::serial::SerialMMIO;

pub const ticks_in_second : usize = 20;
pub const NUM_CPUS : usize = 4;


static mut current_stack : usize = 0;
static cpus_awake: atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

static need_stub : atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

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
    static stub_l1pagetable : *const Ptr;
    static stub_l2pagetable : *const Ptr;
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

pub fn write_to_console(s: &str) {
    serial::Writer::new(unsafe { serial_base }).writeln(s);
}

pub struct PlatformServices {
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

    // the other CPUs still need the stub..
    need_stub.store(NUM_CPUS-1, atomic::Ordering::SeqCst);

    // for 1 .. (cpu-1):
    //    set stack for CPU
    //    do memory barrier()
    //    wake other CPU(i)
    //    wait for CPU

    let mem_manager = &::platform::get_platform_services().mem_manager;
    let fa = &::platform::get_platform_services().frame_alloc;

    let base = mem_manager.p2v(ARM_LOCAL_PSTART).unwrap();
        
    let mailboxes = mailbox::LocalMailbox::new();

    for i in 1 .. NUM_CPUS {
        // TODO: allocate stack instead of making up random values..
        
        let pa = fa.allocate(1).unwrap();
        // TODO - de uglyfy
        let stk = ::mem::VirtualAddress(0x100_0000 + 0x1000*i);
        mem_manager.map(pa, stk, ::mem::MemorySize::PageSizes(1)).unwrap();

        unsafe{current_stack = stk.0  + ::platform::PAGE_SIZE};
        ::arch::arm::cpu::memory_write_barrier();

        // wake up CPU
        // TODO: WAKE UP CPU
        // write start address to CPU N mailbox 3
        mailboxes.mailboxes[i].set_high(3, _secondary_start as *const u32 as u32);

        // wait for cpu to start
        loop {
            // other cpu hatched and cleared his mailbox
            // what the other cpu does, is "documented" in qemu's write_smpboot: https://github.com/qemu/qemu/blob/4771d756f46219762477aaeaaef9bd215e3d5c60/hw/arm/raspi.c#L35)
            let cpunmbox3 = mailboxes.mailboxes[i].read(3);
            if cpunmbox3 == 0 {
                break;
            }
        }

        // wait for cpu to use the new stack
        while cpus_awake.load(atomic::Ordering::SeqCst) != i {}
    }

    // TODO: start and wait for other CPUs
    // TODO: once other cpus started, and signaled that they swiched to use page_table and waiting somewhere in kernel virtmem, continue
    // TODO: remove stub from skip ranges

    // TODO: scheduler should be somewhat available here..

    PlatformServices{
    //    pic: pic_
        mailboxes : mailboxes
    }
}

pub fn send_ipi(id : usize, ipi : ::cpu::IPI) {
    let mailboxes = & ::platform::get_platform_services().arch_services.as_ref().unwrap().board_services.mailboxes;
    mailboxes.mailboxes[id].set_high(0, 1 <<  (ipi as usize));
}


fn clear_ipi(ipi : ::cpu::IPI) {
    let id = ::platform::get_platform_services().get_current_cpu().id();
    let mailboxes = & ::platform::get_platform_services().arch_services.as_ref().unwrap().board_services.mailboxes;
    mailboxes.mailboxes[id].set_low(0, 1 << (ipi as usize));
}


#[no_mangle]
pub extern "C" fn rpi_multi_main() -> ! {
    // we got to here, that means that the stack 
    // is no longer the temp stack, and we can continue and init other CPUs

    // notify..
    cpus_awake.fetch_add(1, atomic::Ordering::SeqCst);

    // TODO: !!
    // init real page table

    if 1 == need_stub.fetch_sub(1, atomic::Ordering::SeqCst) {
        // if previous value was one, it means that we are the last one that needed the stub
        // and we can release it now
        // TODO: release stub
    }

    // TODO init timer

    // TODO method for all CPUs:
    // unmask mailbox interrupts (dedicate one mailbox to page table changes?)
    // ??

    loop{}
}
